use crate::common::*;

pub(crate) struct Capitalized<T: Display>(pub(crate) T);

impl<T: Display> Display for Capitalized<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let text = self.0.to_string();
    if let Some(first) = text.chars().next() {
      write!(f, "{}{}", first.to_uppercase(), &text[first.len_utf8()..])
    } else {
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty() {
    assert_eq!("", Capitalized("").to_string());
  }

  #[test]
  fn single() {
    assert_eq!("X", Capitalized("x").to_string());
  }

  #[test]
  fn multiple() {
    assert_eq!("Xyz", Capitalized("xyz").to_string());
  }
}
