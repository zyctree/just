use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Setting<'src> {
  Shell(Shell<'src>),
  ModuleExperiment,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Shell<'src> {
  pub(crate) command: StringLiteral<'src>,
  pub(crate) arguments: Vec<StringLiteral<'src>>,
}
