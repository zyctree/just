use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Module<'src> {
  pub(crate) aliases: Table<'src, Alias<'src>>,
  pub(crate) assignments: Table<'src, Assignment<'src>>,
  pub(crate) recipes: Table<'src, Rc<Recipe<'src>>>,
  pub(crate) settings: Settings<'src>,
  pub(crate) submodules: Table<'src, Submodule<'src>>,
}

impl<'src: 'run, 'run> Module<'src> {
  pub(crate) fn groups<'slice>(
    &'run self,
    mut arguments: &'slice [&'run str],
  ) -> RunResult<'run, Vec<Group<'src, 'run, 'slice>>> {
    let mut groups = Vec::new();
    while let Some((head, tail)) = arguments.split_first() {
      let (group, rest) = self.group(head, tail)?;
      groups.push(group);
      arguments = rest;
    }
    Ok(groups)
  }

  fn group<'slice>(
    &'run self,
    name: &'run str,
    arguments: &'slice [&'run str],
  ) -> RunResult<'run, (Group<'src, 'run, 'slice>, &'slice [&'run str])> {
    // TODO: deal with submodules

    if let Some(recipe) = self.get_recipe(name) {
      let argument_range = recipe.argument_range();
      let argument_count = cmp::min(arguments.len(), recipe.max_arguments());
      if !argument_range.range_contains(&argument_count) {
        return Err(RuntimeError::ArgumentCountMismatch {
          recipe: recipe.name(),
          parameters: recipe.parameters.iter().collect(),
          found: arguments.len(),
          min: recipe.min_arguments(),
          max: recipe.max_arguments(),
        });
      }
      Ok((
        Group {
          arguments: &arguments[0..argument_count],
          recipe,
        },
        &arguments[argument_count..],
      ))
    } else if let Some(submodule) = self.submodules.get(name) {
      if let Some((head, tail)) = arguments.split_first() {
        submodule.items.group(head, tail)
      } else {
        unimplemented!()
      }
    } else {
      Err(RuntimeError::UnknownRecipe {
        recipe: name,
        suggestion: self.suggest(name),
      })
    }
  }

  fn suggest(&'run self, name: &str) -> Option<&'src str> {
    let mut suggestions = self
      .recipes
      .keys()
      .cloned()
      .map(|suggestion| (edit_distance(suggestion, name), suggestion))
      .collect::<Vec<(usize, &str)>>();
    suggestions.sort();
    if let Some(&(distance, suggestion)) = suggestions.first() {
      if distance < 3 {
        return Some(suggestion);
      }
    }
    None
  }

  fn get_recipe(&'run self, name: &str) -> Option<&'run Recipe<'src>> {
    if let Some(recipe) = self.recipes.get(name) {
      Some(recipe)
    } else if let Some(alias) = self.aliases.get(name) {
      Some(alias.target.as_ref())
    } else {
      None
    }
  }
}
