// Responsabilidad: Definir todos los IDs nominales de la IR.

use std::fmt::Display;

use crate::common::IncrementalId;

/// ID para indexar valores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueId(pub(crate) usize);

impl IncrementalId for ValueId {
  fn from_usize(value: usize) -> Self {
    Self(value)
  }
}

impl Display for ValueId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "%v{}", self.0)
  }
}

/// ID para indexar instrucciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InstId(pub(crate) usize);

impl IncrementalId for InstId {
  fn from_usize(value: usize) -> Self {
    Self(value)
  }
}

/// ID para indexar bloques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BlockId(pub(crate) usize);

impl IncrementalId for BlockId {
  fn from_usize(value: usize) -> Self {
    Self(value)
  }
}

impl Display for BlockId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "bb{}", self.0)
  }
}
