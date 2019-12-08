use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct CompilationError<'src> {
  pub(crate) token: Token<'src>,
  pub(crate) kind: CompilationErrorKind<'src>,
}

impl Error for CompilationError<'_> {}

impl Display for CompilationError<'_> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use CompilationErrorKind::*;
    let message = Color::fmt(f).message();

    write!(f, "{}", message.prefix())?;

    match self.kind {
      Collision {
        first,
        name,
        namespace,
        kind,
        first_kind,
      } => {
        // TODO:
        // - integration tests for these
        // - remove other error variants
        if kind == first_kind {
          writeln!(
            f,
            "{} `{}` first {} on line {} is {} on line {}",
            Capitalized(kind),
            name,
            if namespace == Namespace::Setting {
              "set"
            } else {
              "defined"
            },
            first.ordinal(),
            if namespace == Namespace::Setting {
              "set again"
            } else {
              "redefined"
            },
            self.token.line.ordinal(),
          )?;
        } else {
          writeln!(
            f,
            "{} `{}` conflicts with {} first defined on line {}",
            Capitalized(kind),
            name,
            first_kind,
            first.ordinal(),
          )?;
        }
      }
      CircularRecipeDependency { recipe, ref circle } => {
        if circle.len() == 2 {
          writeln!(f, "Recipe `{}` depends on itself", recipe)?;
        } else {
          writeln!(
            f,
            "Recipe `{}` has circular dependency `{}`",
            recipe,
            circle.join(" -> ")
          )?;
        }
      }
      CircularVariableDependency {
        variable,
        ref circle,
      } => {
        if circle.len() == 2 {
          writeln!(f, "Variable `{}` is defined in terms of itself", variable)?;
        } else {
          writeln!(
            f,
            "Variable `{}` depends on its own value: `{}`",
            variable,
            circle.join(" -> ")
          )?;
        }
      }
      ForbiddenModule { module } => {
        writeln!(
          f,
          "Found module `{}` without `module-experiment` setting enabled",
          module,
        )?;
        writeln!(
          f,
          "Use `set module-experiment := true` to enable modules. Modules are experimental and may change, break, or be removed at any time.",
        )?;
      }
      InvalidEscapeSequence { character } => {
        let representation = match character {
          '`' => r"\`".to_string(),
          '\\' => r"\".to_string(),
          '\'' => r"'".to_string(),
          '"' => r#"""#.to_string(),
          _ => character.escape_default().collect(),
        };
        writeln!(f, "`\\{}` is not a valid escape sequence", representation)?;
      }
      DuplicateParameter { recipe, parameter } => {
        writeln!(
          f,
          "Recipe `{}` has duplicate parameter `{}`",
          recipe, parameter
        )?;
      }
      UnexpectedName { expected, found } => {
        writeln!(
          f,
          "Expected identifier `{}` but found `{}`",
          expected, found,
        )?;
      }
      UnexpectedToken {
        ref expected,
        found,
      } => {
        writeln!(f, "Expected {}, but found {}", List::or(expected), found)?;
      }
      DependencyArgumentCountMismatch {
        dependency,
        found,
        min,
        max,
      } => {
        write!(
          f,
          "Dependency `{}` got {} {} but takes ",
          dependency,
          found,
          Count("argument", found),
        )?;

        if min == max {
          let expected = min;
          writeln!(f, "{} {}", expected, Count("argument", expected))?;
        } else if found < min {
          writeln!(f, "at least {} {}", min, Count("argument", min))?;
        } else {
          writeln!(f, "at most {} {}", max, Count("argument", max))?;
        }
      }
      ParameterShadowsVariable { parameter } => {
        writeln!(
          f,
          "Parameter `{}` shadows variable of the same name",
          parameter
        )?;
      }
      RequiredParameterFollowsDefaultParameter { parameter } => {
        writeln!(
          f,
          "Non-default parameter `{}` follows default parameter",
          parameter
        )?;
      }
      ParameterFollowsVariadicParameter { parameter } => {
        writeln!(f, "Parameter `{}` follows variadic parameter", parameter)?;
      }
      MixedLeadingWhitespace { whitespace } => {
        writeln!(
          f,
          "Found a mix of tabs and spaces in leading whitespace: `{}`\n\
           Leading whitespace may consist of tabs or spaces, but not both",
          ShowWhitespace(whitespace)
        )?;
      }
      ExtraLeadingWhitespace => {
        writeln!(f, "Recipe line has extra leading whitespace")?;
      }
      FunctionArgumentCountMismatch {
        function,
        found,
        expected,
      } => {
        writeln!(
          f,
          "Function `{}` called with {} {} but takes {}",
          function,
          found,
          Count("argument", found),
          expected
        )?;
      }
      InconsistentLeadingWhitespace { expected, found } => {
        writeln!(
          f,
          "Recipe line has inconsistent leading whitespace. \
           Recipe started with `{}` but found line with `{}`",
          ShowWhitespace(expected),
          ShowWhitespace(found)
        )?;
      }
      UnknownAliasTarget { alias, target } => {
        writeln!(f, "Alias `{}` has an unknown target `{}`", alias, target)?;
      }
      UnknownDependency { recipe, unknown } => {
        writeln!(
          f,
          "Recipe `{}` has unknown dependency `{}`",
          recipe, unknown
        )?;
      }
      UndefinedVariable { variable } => {
        writeln!(f, "Variable `{}` not defined", variable)?;
      }
      UnknownFunction { function } => {
        writeln!(f, "Call to unknown function `{}`", function)?;
      }
      UnknownSetting { setting } => {
        writeln!(f, "Unknown setting `{}`", setting)?;
      }
      UnknownStartOfToken => {
        writeln!(f, "Unknown start of token:")?;
      }
      UnpairedCarriageReturn => {
        writeln!(f, "Unpaired carriage return")?;
      }
      UnterminatedInterpolation => {
        writeln!(f, "Unterminated interpolation")?;
      }
      UnterminatedString => {
        writeln!(f, "Unterminated string")?;
      }
      UnterminatedBacktick => {
        writeln!(f, "Unterminated backtick")?;
      }
      Internal { ref message } => {
        writeln!(
          f,
          "Internal error, this may indicate a bug in just: {}\n\
           consider filing an issue: https://github.com/casey/just/issues/new",
          message
        )?;
      }
    }

    write!(f, "{}", message.suffix())?;

    self.token.write_context(f, Color::fmt(f).error())
  }
}
