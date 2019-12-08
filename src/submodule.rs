use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Submodule<'src, I = Module<'src>> {
  pub(crate) name: Name<'src>,
  pub(crate) items: I,
}

impl<'src, I> Keyed<'src> for Submodule<'src, I> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
