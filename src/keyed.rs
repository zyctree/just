use crate::common::*;

pub(crate) trait Keyed<'key> {
  fn key(&self) -> &'key str;
}

impl<'key, T: Keyed<'key>> Keyed<'key> for Rc<T> {
  fn key(&self) -> &'key str {
    self.as_ref().key()
  }
}

impl<'key, T: Keyed<'key>> Keyed<'key> for &T {
  fn key(&self) -> &'key str {
    <T as Keyed>::key(self)
  }
}
