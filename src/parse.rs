use crate::common::*;

/// The top-level type produced by the parser.
///
/// Not all successful parses result in valid justfiles, so additional
/// consistency checks and name resolution are performed by the `Analyzer`,
/// which produces a `Justfile` from a `Parse`.
#[derive(Debug)]
pub(crate) struct Parse<'src> {
  /// Top-level suite of items
  pub(crate) suite: Suite<'src>,
  /// Warnings encountered during parsing
  pub(crate) warnings: Vec<Warning<'src>>,
}
