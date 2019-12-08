use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Items<'src> {
  pub(crate) aliases: Table<'src, Alias<'src, Name<'src>>>,
  pub(crate) assignments: Table<'src, Assignment<'src>>,
  pub(crate) recipes: Table<'src, UnresolvedRecipe<'src>>,
  pub(crate) sets: Table<'src, Set<'src>>,
  pub(crate) submodules: Table<'src, UnresolvedSubmodule<'src>>,
}

impl<'src> Items<'src> {
  pub(crate) fn new() -> Items<'src> {
    Items {
      recipes: Table::new(),
      assignments: Table::new(),
      aliases: Table::new(),
      sets: Table::new(),
      submodules: Table::new(),
    }
  }
}
