use crate::common::*;

pub(crate) struct Group<'src: 'run, 'run, 'slice> {
  pub(crate) recipe: &'run Recipe<'src>,
  pub(crate) arguments: &'slice [&'run str],
}
