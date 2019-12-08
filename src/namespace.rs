use crate::common::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Namespace {
  Recipe,
  Assignment,
  Setting,
}

impl Display for Namespace {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use Namespace::*;
    let text = match self {
      Recipe => "recipe",
      Assignment => "assignment",
      Setting => "setting",
    };
    write!(f, "{}", text)
  }
}
