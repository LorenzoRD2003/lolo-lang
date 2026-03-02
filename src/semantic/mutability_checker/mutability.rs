#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability {
  Mutable,
  Immutable,
}

impl Mutability {
  pub fn is_mutable(&self) -> bool {
    match self {
      Mutability::Mutable => true,
      Mutability::Immutable => false,
    }
  }
}