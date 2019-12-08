use crate::common::*;

use CompilationErrorKind::*;

// TODO:
// - test duplicate submodule error message

pub(crate) struct Analyzer;

impl<'src> Analyzer {
  pub(crate) fn analyze(parse: Parse<'src>) -> CompilationResult<'src, Justfile> {
    let analyzer = Analyzer::new();

    analyzer.justfile(parse)
  }

  pub(crate) fn new() -> Analyzer {
    Analyzer
  }

  pub(crate) fn justfile(mut self, parse: Parse<'src>) -> CompilationResult<'src, Justfile<'src>> {
    let root = self.analyze_suite(parse.suite)?;

    Ok(Justfile {
      warnings: parse.warnings,
      root,
    })
  }

  pub(crate) fn analyze_suite(
    &mut self,
    suite: Suite<'src>,
  ) -> CompilationResult<'src, Module<'src>> {
    {
      let mut assignment_table: Table<&Item> = Table::new();
      let mut recipe_table: Table<&Item> = Table::new();
      let mut setting_table: Table<&Item> = Table::new();

      for item in &suite.items {
        let namespace = item.namespace();

        let table = match namespace {
          Namespace::Assignment => &mut assignment_table,
          Namespace::Recipe => &mut recipe_table,
          Namespace::Setting => &mut setting_table,
        };

        if let Some(first) = table.get(item.key()) {
          return Err(item.name().error(Collision {
            name: item.name().lexeme(),
            kind: item.kind(),
            first: first.name().line,
            first_kind: first.kind(),
            namespace,
          }));
        }

        table.insert(item);
      }
    }

    let mut items = Items::new();

    for item in suite.items {
      match item {
        Item::Alias(alias) => {
          items.aliases.insert(alias);
        }
        Item::Assignment(assignment) => {
          items.assignments.insert(assignment);
        }
        Item::Submodule(submodule) => {
          items.submodules.insert(submodule);
        }
        Item::Recipe(recipe) => {
          self.analyze_recipe(&recipe)?;
          items.recipes.insert(recipe);
        }
        Item::Set(set) => {
          items.sets.insert(set);
        }
      }
    }

    self.resolve_items(items)
  }

  fn resolve_items(&mut self, mut items: Items<'src>) -> CompilationResult<'src, Module<'src>> {
    let assignments = items.assignments;

    AssignmentResolver::resolve_assignments(&assignments)?;

    let recipes = RecipeResolver::resolve_recipes(items.recipes, &assignments)?;

    for recipe in recipes.values() {
      for parameter in &recipe.parameters {
        if assignments.contains_key(parameter.name.lexeme()) {
          return Err(parameter.name.token().error(ParameterShadowsVariable {
            parameter: parameter.name.lexeme(),
          }));
        }
      }
    }

    let mut aliases = Table::new();
    while let Some(alias) = items.aliases.pop() {
      aliases.insert(Self::resolve_alias(&recipes, alias)?);
    }

    let mut settings = Settings::new();

    for (_, set) in items.sets.into_iter() {
      match set.value {
        Setting::Shell(shell) => {
          assert!(settings.shell.is_none());
          settings.shell = Some(shell);
        }
        Setting::ModuleExperiment => {
          assert!(!settings.module_experiment);
          settings.module_experiment = true;
        }
      }
    }

    let mut submodules = Table::new();
    while let Some(submodule) = items.submodules.pop() {
      if !settings.module_experiment {
        return Err(submodule.name.error(CompilationErrorKind::ForbiddenModule {
          module: submodule.name.lexeme(),
        }));
      }
      submodules.insert(self.resolve_submodule(submodule)?);
    }

    Ok(Module {
      aliases,
      assignments,
      recipes,
      settings,
      submodules,
    })
  }

  fn analyze_recipe(&self, recipe: &UnresolvedRecipe<'src>) -> CompilationResult<'src, ()> {
    let mut parameters = BTreeSet::new();
    let mut passed_default = false;

    for parameter in &recipe.parameters {
      if parameters.contains(parameter.name.lexeme()) {
        return Err(parameter.name.token().error(DuplicateParameter {
          recipe: recipe.name.lexeme(),
          parameter: parameter.name.lexeme(),
        }));
      }
      parameters.insert(parameter.name.lexeme());

      if parameter.default.is_some() {
        passed_default = true;
      } else if passed_default {
        return Err(
          parameter
            .name
            .token()
            .error(RequiredParameterFollowsDefaultParameter {
              parameter: parameter.name.lexeme(),
            }),
        );
      }
    }

    let mut continued = false;
    for line in &recipe.body {
      if !recipe.shebang && !continued {
        if let Some(Fragment::Text { token }) = line.fragments.first() {
          let text = token.lexeme();

          if text.starts_with(' ') || text.starts_with('\t') {
            return Err(token.error(ExtraLeadingWhitespace));
          }
        }
      }

      continued = line.is_continuation();
    }

    Ok(())
  }

  fn resolve_alias(
    recipes: &Table<'src, Rc<Recipe<'src>>>,
    alias: Alias<'src, Name<'src>>,
  ) -> CompilationResult<'src, Alias<'src>> {
    let token = alias.name.token();

    // Make sure the target recipe exists
    match recipes.get(alias.target.lexeme()) {
      Some(target) => Ok(alias.resolve(target.clone())),
      None => Err(token.error(UnknownAliasTarget {
        alias: alias.name.lexeme(),
        target: alias.target.lexeme(),
      })),
    }
  }

  fn resolve_submodule(
    &mut self,
    submodule: UnresolvedSubmodule<'src>,
  ) -> CompilationResult<'src, Submodule<'src>> {
    Ok(Submodule {
      name: submodule.name,
      items: self.analyze_suite(submodule.items)?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  analysis_error! {
    name: duplicate_alias,
    input: "alias foo = bar\nalias foo = baz",
    offset: 22,
    line: 1,
    column: 6,
    width: 3,
    kind: Collision {
      first: 0,
      first_kind: "alias",
      kind: "alias",
      name: "foo",
      namespace: Namespace::Recipe,
    },
  }

  analysis_error! {
    name: unknown_alias_target,
    input: "alias foo = bar\n",
    offset: 6,
    line: 0,
    column: 6,
    width: 3,
    kind: UnknownAliasTarget {alias: "foo", target: "bar"},
  }

  analysis_error! {
    name: alias_shadows_recipe_before,
    input: "bar: \n  echo bar\nalias foo = bar\nfoo:\n  echo foo",
    offset: 33,
    line: 3,
    column: 0,
    width: 3,
    kind: Collision {
      first: 2,
      first_kind: "alias",
      kind: "recipe",
      name: "foo",
      namespace: Namespace::Recipe,
    },
  }

  analysis_error! {
    name: alias_shadows_recipe_after,
    input: "foo:\n  echo foo\nalias foo = bar\nbar:\n  echo bar",
    offset: 22,
    line: 2,
    column: 6,
    width: 3,
    kind: Collision {
      first: 0,
      first_kind: "recipe",
      kind: "alias",
      name: "foo",
      namespace: Namespace::Recipe,
    },
  }

  analysis_error! {
    name:   required_after_default,
    input:  "hello arg='foo' bar:",
    offset:  16,
    line:   0,
    column: 16,
    width:  3,
    kind:   RequiredParameterFollowsDefaultParameter{parameter: "bar"},
  }

  analysis_error! {
    name:   duplicate_parameter,
    input:  "a b b:",
    offset:  4,
    line:   0,
    column: 4,
    width:  1,
    kind:   DuplicateParameter{recipe: "a", parameter: "b"},
  }

  analysis_error! {
    name:   duplicate_variadic_parameter,
    input:  "a b +b:",
    offset: 5,
    line:   0,
    column: 5,
    width:  1,
    kind:   DuplicateParameter{recipe: "a", parameter: "b"},
  }

  analysis_error! {
    name:   parameter_shadows_varible,
    input:  "foo = \"h\"\na foo:",
    offset:  12,
    line:   1,
    column: 2,
    width:  3,
    kind:   ParameterShadowsVariable{parameter: "foo"},
  }

  analysis_error! {
    name:   duplicate_recipe,
    input:  "a:\nb:\na:",
    offset:  6,
    line:   2,
    column: 0,
    width:  1,
    kind: Collision {
      first: 0,
      first_kind: "recipe",
      kind: "recipe",
      name: "a",
      namespace: Namespace::Recipe,
    },
  }

  analysis_error! {
    name:   duplicate_assignment,
    input:  "a = \"0\"\na = \"0\"",
    offset:  8,
    line:    1,
    column:  0,
    width:   1,
    kind: Collision {
      first: 0,
      first_kind: "variable",
      kind: "variable",
      name: "a",
      namespace: Namespace::Assignment,
    },
  }

  analysis_error! {
    name:   duplicate_setting,
    input:  "
      set module-experiment := true
      set module-experiment := true
    ",
    offset:  34,
    line:    1,
    column:  4,
    width:   17,
    kind: Collision {
      first: 0,
      first_kind: "setting",
      kind: "setting",
      name: "module-experiment",
      namespace: Namespace::Setting,
    },
  }

  analysis_error! {
    name:   extra_whitespace,
    input:  "a:\n blah\n  blarg",
    offset:  10,
    line:   2,
    column: 1,
    width:  6,
    kind:   ExtraLeadingWhitespace,
  }
}
