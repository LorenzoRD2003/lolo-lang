// Responsabilidad: Definir todos los IDs nominales de la IR.

use crate::common::IncrementalId;

/// ID para indexar variables locales. (cuando haya funciones, tendremos diferentes contextos locales)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LocalId(pub(crate) usize);

impl IncrementalId for LocalId {
  fn from_usize(value: usize) -> Self {
    Self(value)
  }
}

/// ID para indexar valores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueId(pub(crate) usize);

impl IncrementalId for ValueId {
  fn from_usize(value: usize) -> Self {
    Self(value)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockId(pub(crate) usize);

impl IncrementalId for BlockId {
  fn from_usize(value: usize) -> Self {
    Self(value)
  }
}
