use crate::common::*;

/// A single top-level item
#[derive(Debug, PartialEq)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Name<'src>>),
  Assignment(Assignment<'src>),
  Submodule(UnresolvedSubmodule<'src>),
  Recipe(UnresolvedRecipe<'src>),
  Set(Set<'src>),
}

impl<'src> Item<'src> {
  pub(crate) fn name(&self) -> Name<'src> {
    use Item::*;
    match self {
      Alias(alias) => alias.name,
      Assignment(assignment) => assignment.name,
      Submodule(submodule) => submodule.name,
      Recipe(recipe) => recipe.name,
      Set(set) => set.name,
    }
  }

  pub(crate) fn namespace(&self) -> Namespace {
    use Item::*;
    match self {
      Alias(_) | Submodule(_) | Recipe(_) => Namespace::Recipe,
      Assignment(_) => Namespace::Assignment,
      Set(_) => Namespace::Setting,
    }
  }

  pub(crate) fn kind(&self) -> &'static str {
    use Item::*;
    match self {
      Alias(_) => "alias",
      Assignment(_) => "variable",
      Recipe(_) => "recipe",
      Set(_) => "setting",
      Submodule(_) => "submodule",
    }
  }
}

impl<'src> Keyed<'src> for Item<'src> {
  fn key(&self) -> &'src str {
    self.name().lexeme()
  }
}
