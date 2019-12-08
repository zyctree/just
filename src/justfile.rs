use crate::common::*;

// TODO:
// - check that eols are actually expected
//   - recipe
//   - alias
//   - assignment
//   - export
//   - setting
//   - module
// - check that no functional change without module experiment
// - add tests for current functionality
//
// - make issue with todo list
// - consider switching to {} for modules
// - make evaluate work with submodules
// - make list work with submodules
// - think about other subcommands
// - make setting inheritance work
// - make first recipe default work with submodules
// - make variable reference in submodules work
// - make aliases work with submodules
//
// modules are almost totally independant:
// - they cannot access recipes or assignments outside of that module
// - they *can* access settings. Settings inherit from parent module, and can be overridden
// - a module and a recipe cannot have the same name
//
// - allow reference values in parent modules
// - allow depending on recipes in parent module
// - allow depending on recipes in child modules
// - allow depending on recipes is adjacent modules (root::foo::bar, super::foo::bar)
//
// foo::
//
// 	bar:
// 		echo bar
//
// 	baz:
// 		echo baz
//
//  foo::
//
//  - inline modules
//
//  mod foo ;
//
// foo:: "bar"
//
//  later:
//  - out-of-line modules w/path to file (add source-loader, load-mid parse, or resolve)
//  - out-of-line modules w/path to directory
//  - out of line modules w/no suite
//
//  - finish modules / shitcan modules
//  - start working on torrent creator

#[derive(Debug, PartialEq)]
pub(crate) struct Justfile<'src> {
  pub(crate) root: Module<'src>,
  pub(crate) warnings: Vec<Warning<'src>>,
}

impl<'src: 'run, 'run> Justfile<'src> {
  pub(crate) fn first(&self) -> Option<&Recipe> {
    self
      .root
      .recipes
      .values()
      .map(Rc::as_ref)
      .min_by_key(|recipe| recipe.line_number())
  }

  pub(crate) fn count(&self) -> usize {
    self.root.recipes.len()
  }

  pub(crate) fn suggest(&self, name: &str) -> Option<&'src str> {
    let mut suggestions = self
      .root
      .recipes
      .keys()
      .map(|suggestion| (edit_distance(suggestion, name), suggestion))
      .collect::<Vec<_>>();
    suggestions.sort();
    if let Some(&(distance, suggestion)) = suggestions.first() {
      if distance < 3 {
        return Some(suggestion);
      }
    }
    None
  }

  pub(crate) fn run(
    &'run self,
    config: &'run Config,
    working_directory: &'run Path,
    overrides: &'run BTreeMap<String, String>,
    arguments: &'run [String],
  ) -> RunResult<'run, ()> {
    let dotenv = load_dotenv()?;

    let scope = {
      let mut scope = Scope::new();
      let mut unknown_overrides = Vec::new();

      for (name, value) in overrides {
        if let Some(assignment) = self.root.assignments.get(name) {
          scope.bind(assignment.export, assignment.name, value.clone());
        } else {
          unknown_overrides.push(name.as_ref());
        }
      }

      if !unknown_overrides.is_empty() {
        return Err(RuntimeError::UnknownOverrides {
          overrides: unknown_overrides,
        });
      }

      Evaluator::evaluate_assignments(
        &self.root.assignments,
        config,
        &dotenv,
        scope,
        &self.root.settings,
        working_directory,
      )?
    };

    if let Subcommand::Evaluate { .. } = config.subcommand {
      let mut width = 0;

      for name in scope.names() {
        width = cmp::max(name.len(), width);
      }

      for binding in scope.bindings() {
        println!(
          "{0:1$} := \"{2}\"",
          binding.name.lexeme(),
          width,
          binding.value
        );
      }

      return Ok(());
    }

    let argvec: Vec<&str> = if !arguments.is_empty() {
      arguments.iter().map(|argument| argument.as_str()).collect()
    } else if let Some(recipe) = self.first() {
      let min_arguments = recipe.min_arguments();
      if min_arguments > 0 {
        return Err(RuntimeError::DefaultRecipeRequiresArguments {
          recipe: recipe.name.lexeme(),
          min_arguments,
        });
      }
      vec![recipe.name()]
    } else {
      return Err(RuntimeError::NoRecipes);
    };

    let arguments = argvec.as_slice();

    let unknown_overrides = overrides
      .keys()
      .filter(|name| !self.root.assignments.contains_key(name.as_str()))
      .map(|name| name.as_str())
      .collect::<Vec<&str>>();

    if !unknown_overrides.is_empty() {
      return Err(RuntimeError::UnknownOverrides {
        overrides: unknown_overrides,
      });
    }
    let groups = self.root.groups(arguments)?;

    let context = RecipeContext {
      settings: &self.root.settings,
      config,
      scope,
      working_directory,
    };

    let mut ran = BTreeSet::new();
    for Group { recipe, arguments } in groups {
      Self::run_recipe(&context, recipe, arguments, &dotenv, &mut ran)?
    }

    Ok(())
  }

  pub(crate) fn get_alias(&self, name: &str) -> Option<&Alias> {
    self.root.aliases.get(name)
  }

  pub(crate) fn get_recipe(&self, name: &str) -> Option<&Recipe<'src>> {
    if let Some(recipe) = self.root.recipes.get(name) {
      Some(recipe)
    } else if let Some(alias) = self.root.aliases.get(name) {
      Some(alias.target.as_ref())
    } else {
      None
    }
  }

  fn run_recipe(
    context: &'run RecipeContext<'src, 'run>,
    recipe: &Recipe<'src>,
    arguments: &[&'run str],
    dotenv: &BTreeMap<String, String>,
    ran: &mut BTreeSet<Vec<String>>,
  ) -> RunResult<'src, ()> {
    let scope = Evaluator::evaluate_parameters(
      context.config,
      dotenv,
      &recipe.parameters,
      arguments,
      &context.scope,
      context.settings,
      context.working_directory,
    )?;

    let mut evaluator = Evaluator::recipe_evaluator(
      context.config,
      dotenv,
      &scope,
      context.settings,
      context.working_directory,
    );

    for Dependency { recipe, arguments } in &recipe.dependencies {
      let mut invocation = vec![recipe.name().to_owned()];

      for argument in arguments {
        invocation.push(evaluator.evaluate_expression(argument)?);
      }

      if !ran.contains(&invocation) {
        let arguments = invocation
          .iter()
          .skip(1)
          .map(String::as_ref)
          .collect::<Vec<&str>>();
        Self::run_recipe(context, recipe, &arguments, dotenv, ran)?;
      }
    }

    recipe.run(context, dotenv, scope)?;

    let mut invocation = Vec::new();
    invocation.push(recipe.name().to_owned());
    for argument in arguments.iter().cloned() {
      invocation.push(argument.to_owned());
    }

    ran.insert(invocation);
    Ok(())
  }
}

impl<'src> Display for Justfile<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let mut items = self.root.recipes.len() + self.root.assignments.len() + self.root.aliases.len();
    for (name, assignment) in &self.root.assignments {
      if assignment.export {
        write!(f, "export ")?;
      }
      write!(f, "{} := {}", name, assignment.value)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for alias in self.root.aliases.values() {
      write!(f, "{}", alias)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for recipe in self.root.recipes.values() {
      write!(f, "{}", recipe)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use testing::compile;
  use RuntimeError::*;

  run_error! {
    name: unknown_recipe,
    src: "a:\nb:\nc:",
    args: ["a", "x", "y", "z"],
    error: UnknownRecipe {
      recipe,
      suggestion,
    },
    check: {
      assert_eq!(recipe, "x");
      assert_eq!(suggestion, Some("a"));
    }
  }

  // this test exists to make sure that shebang recipes
  // run correctly. although this script is still
  // executed by a shell its behavior depends on the value of a
  // variable and continuing even though a command fails,
  // whereas in plain recipes variables are not available
  // in subsequent lines and execution stops when a line
  // fails
  run_error! {
    name: run_shebang,
    src: "
      a:
        #!/usr/bin/env sh
        code=200
          x() { return $code; }
            x
              x
    ",
    args: ["a"],
    error: Code {
      recipe,
      line_number,
      code,
    },
    check: {
      assert_eq!(recipe, "a");
      assert_eq!(code, 200);
      assert_eq!(line_number, None);
    }
  }

  run_error! {
    name: code_error,
    src: "
      fail:
        @exit 100
    ",
    args: ["fail"],
    error: Code {
      recipe,
      line_number,
      code,
    },
    check: {
      assert_eq!(recipe, "fail");
      assert_eq!(code, 100);
      assert_eq!(line_number, Some(2));
    }
  }

  run_error! {
    name: run_args,
    src: r#"
      a return code:
        @x() { {{return}} {{code + "0"}}; }; x
    "#,
    args: ["a", "return", "15"],
    error: Code {
      recipe,
      line_number,
      code,
    },
    check: {
      assert_eq!(recipe, "a");
      assert_eq!(code, 150);
      assert_eq!(line_number, Some(2));
    }
  }

  run_error! {
    name: missing_some_arguments,
    src: "a b c d:",
    args: ["a", "b", "c"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_some_arguments_variadic,
    src: "a b c +d:",
    args: ["a", "B", "C"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, usize::MAX - 1);
    }
  }

  run_error! {
    name: missing_all_arguments,
    src: "a b c d:\n echo {{b}}{{c}}{{d}}",
    args: ["a"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 0);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_some_defaults,
    src: "a b c d='hello':",
    args: ["a", "b"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 1);
      assert_eq!(min, 2);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: missing_all_defaults,
    src: "a b c='r' d='h':",
    args: ["a"],
    error: ArgumentCountMismatch {
      recipe,
      parameters,
      found,
      min,
      max,
    },
    check: {
      let param_names = parameters
        .iter()
        .map(|p| p.name.lexeme())
        .collect::<Vec<&str>>();
      assert_eq!(recipe, "a");
      assert_eq!(param_names, ["b", "c", "d"]);
      assert_eq!(found, 0);
      assert_eq!(min, 1);
      assert_eq!(max, 3);
    }
  }

  run_error! {
    name: unknown_overrides,
    src: "
      a:
       echo {{`f() { return 100; }; f`}}
    ",
    args: ["foo=bar", "baz=bob", "a"],
    error: UnknownOverrides { overrides },
    check: {
      assert_eq!(overrides, &["baz", "foo"]);
    }
  }

  run_error! {
    name: export_failure,
    src: r#"
      export foo = "a"
      baz = "c"
      export bar = "b"
      export abc = foo + bar + baz

      wut:
        echo $foo $bar $baz
    "#,
    args: ["--quiet", "wut"],
    error: Code {
      code: _,
      line_number,
      recipe,
    },
    check: {
      assert_eq!(recipe, "wut");
      assert_eq!(line_number, Some(7));
    }
  }

  macro_rules! test {
    ($name:ident, $input:expr, $expected:expr $(,)*) => {
      #[test]
      fn $name() {
        test($input, $expected);
      }
    };
  }

  fn test(input: &str, expected: &str) {
    let justfile = compile(input);
    let actual = format!("{:#}", justfile);
    assert_eq!(actual, expected);
    println!("Re-parsing...");
    let reparsed = compile(&actual);
    let redumped = format!("{:#}", reparsed);
    assert_eq!(redumped, actual);
  }

  test! {
    parse_empty,
    "

# hello


    ",
    "",
  }

  test! {
    parse_string_default,
    r#"

foo a="b\t":


  "#,
    r#"foo a="b\t":"#,
  }

  test! {
  parse_multiple,
    r#"
a:
b:
"#,
    r#"a:

b:"#,
  }

  test! {
    parse_variadic,
    r#"

foo +a:


  "#,
    r#"foo +a:"#,
  }

  test! {
    parse_variadic_string_default,
    r#"

foo +a="Hello":


  "#,
    r#"foo +a="Hello":"#,
  }

  test! {
    parse_raw_string_default,
    r#"

foo a='b\t':


  "#,
    r#"foo a='b\t':"#,
  }

  test! {
    parse_export,
    r#"
export a := "hello"

  "#,
    r#"export a := "hello""#,
  }

  test! {
  parse_alias_after_target,
    r#"
foo:
  echo a
alias f := foo
"#,
r#"alias f := foo

foo:
    echo a"#
  }

  test! {
  parse_alias_before_target,
    r#"
alias f := foo
foo:
  echo a
"#,
r#"alias f := foo

foo:
    echo a"#
  }

  test! {
  parse_alias_with_comment,
    r#"
alias f := foo #comment
foo:
  echo a
"#,
r#"alias f := foo

foo:
    echo a"#
  }

  test! {
  parse_complex,
    "
x:
y:
z:
foo := \"xx\"
bar := foo
goodbye := \"y\"
hello a b    c   : x y    z #hello
  #! blah
  #blarg
  {{ foo + bar}}abc{{ goodbye\t  + \"x\" }}xyz
  1
  2
  3
",
    "bar := foo

foo := \"xx\"

goodbye := \"y\"

hello a b c: x y z
    #! blah
    #blarg
    {{foo + bar}}abc{{goodbye + \"x\"}}xyz
    1
    2
    3

x:

y:

z:"
  }

  test! {
  parse_shebang,
    "
practicum := 'hello'
install:
\t#!/bin/sh
\tif [[ -f {{practicum}} ]]; then
\t\treturn
\tfi
",
    "practicum := 'hello'

install:
    #!/bin/sh
    if [[ -f {{practicum}} ]]; then
    \treturn
    fi",
  }

  test! {
    parse_simple_shebang,
    "a:\n #!\n  print(1)",
    "a:\n    #!\n     print(1)",
  }

  test! {
  parse_assignments,
    r#"a := "0"
c := a + b + a + b
b := "1"
"#,
    r#"a := "0"

b := "1"

c := a + b + a + b"#,
  }

  test! {
  parse_assignment_backticks,
    "a := `echo hello`
c := a + b + a + b
b := `echo goodbye`",
    "a := `echo hello`

b := `echo goodbye`

c := a + b + a + b",
  }

  test! {
  parse_interpolation_backticks,
    r#"a:
  echo {{  `echo hello` + "blarg"   }} {{   `echo bob`   }}"#,
    r#"a:
    echo {{`echo hello` + "blarg"}} {{`echo bob`}}"#,
  }

  test! {
    eof_test,
    "x:\ny:\nz:\na b c: x y z",
    "a b c: x y z\n\nx:\n\ny:\n\nz:",
  }

  test! {
    string_quote_escape,
    r#"a := "hello\"""#,
    r#"a := "hello\"""#,
  }

  test! {
    string_escapes,
    r#"a := "\n\t\r\"\\""#,
    r#"a := "\n\t\r\"\\""#,
  }

  test! {
  parameters,
    "a b c:
  {{b}} {{c}}",
    "a b c:
    {{b}} {{c}}",
  }

  test! {
  unary_functions,
    "
x := arch()

a:
  {{os()}} {{os_family()}}",
    "x := arch()

a:
    {{os()}} {{os_family()}}",
  }

  test! {
  env_functions,
    r#"
x := env_var('foo',)

a:
  {{env_var_or_default('foo' + 'bar', 'baz',)}} {{env_var(env_var("baz"))}}"#,
    r#"x := env_var('foo')

a:
    {{env_var_or_default('foo' + 'bar', 'baz')}} {{env_var(env_var("baz"))}}"#,
  }

  test! {
    parameter_default_string,
    r#"
f x="abc":
"#,
    r#"f x="abc":"#,
  }

  test! {
    parameter_default_raw_string,
    r#"
f x='abc':
"#,
    r#"f x='abc':"#,
  }

  test! {
    parameter_default_backtick,
    r#"
f x=`echo hello`:
"#,
    r#"f x=`echo hello`:"#,
  }

  test! {
    parameter_default_concatination_string,
    r#"
f x=(`echo hello` + "foo"):
"#,
    r#"f x=(`echo hello` + "foo"):"#,
  }

  test! {
    parameter_default_concatination_variable,
    r#"
x := "10"
f y=(`echo hello` + x) +z="foo":
"#,
    r#"x := "10"

f y=(`echo hello` + x) +z="foo":"#,
  }

  test! {
    parameter_default_multiple,
    r#"
x := "10"
f y=(`echo hello` + x) +z=("foo" + "bar"):
"#,
    r#"x := "10"

f y=(`echo hello` + x) +z=("foo" + "bar"):"#,
  }

  test! {
    concatination_in_group,
    "x := ('0' + '1')",
    "x := ('0' + '1')",
  }

  test! {
    string_in_group,
    "x := ('0'   )",
    "x := ('0')",
  }

  #[rustfmt::skip]
  test! {
    escaped_dos_newlines,
    "@spam:\r
\t{ \\\r
\t\tfiglet test; \\\r
\t\tcargo build --color always 2>&1; \\\r
\t\tcargo test  --color always -- --color always 2>&1; \\\r
\t} | less\r
",
"@spam:
    { \\
    \tfiglet test; \\
    \tcargo build --color always 2>&1; \\
    \tcargo test  --color always -- --color always 2>&1; \\
    } | less",
  }
}
