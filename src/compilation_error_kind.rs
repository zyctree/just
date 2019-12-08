use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) enum CompilationErrorKind<'src> {
  CircularRecipeDependency {
    recipe: &'src str,
    circle: Vec<&'src str>,
  },
  CircularVariableDependency {
    variable: &'src str,
    circle: Vec<&'src str>,
  },
  Collision {
    first: usize,
    first_kind: &'static str,
    kind: &'static str,
    name: &'src str,
    namespace: Namespace,
  },
  DependencyArgumentCountMismatch {
    dependency: &'src str,
    found: usize,
    min: usize,
    max: usize,
  },
  DuplicateParameter {
    recipe: &'src str,
    parameter: &'src str,
  },
  ExtraLeadingWhitespace,
  ForbiddenModule {
    module: &'src str,
  },
  FunctionArgumentCountMismatch {
    function: &'src str,
    found: usize,
    expected: usize,
  },
  InconsistentLeadingWhitespace {
    expected: &'src str,
    found: &'src str,
  },
  Internal {
    message: String,
  },
  InvalidEscapeSequence {
    character: char,
  },
  MixedLeadingWhitespace {
    whitespace: &'src str,
  },
  ParameterFollowsVariadicParameter {
    parameter: &'src str,
  },
  ParameterShadowsVariable {
    parameter: &'src str,
  },
  RequiredParameterFollowsDefaultParameter {
    parameter: &'src str,
  },
  UndefinedVariable {
    variable: &'src str,
  },
  UnexpectedName {
    expected: &'static str,
    found: &'src str,
  },
  UnexpectedToken {
    expected: Vec<TokenKind>,
    found: TokenKind,
  },
  UnknownAliasTarget {
    alias: &'src str,
    target: &'src str,
  },
  UnknownDependency {
    recipe: &'src str,
    unknown: &'src str,
  },
  UnknownFunction {
    function: &'src str,
  },
  UnknownSetting {
    setting: &'src str,
  },
  UnknownStartOfToken,
  UnpairedCarriageReturn,
  UnterminatedBacktick,
  UnterminatedInterpolation,
  UnterminatedString,
}
