use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Suite<'src> {
  pub(crate) items: Vec<Item<'src>>,
}

impl<'src> Suite<'src> {
  pub(crate) fn new() -> Suite<'src> {
    Suite { items: Vec::new() }
  }
}
