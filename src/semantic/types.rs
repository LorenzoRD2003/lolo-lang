// Los tipos los manejo con un enum simple porque por ahora todo va a tener tipos estaticos
// y los usuarios no van a poder definir tipos.

use std::fmt::Display;

use crate::ast::expr::ConstValue;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
  Int32,
  Bool,
  Unknown,
  /// `Unit` significa "no hay valor significativo". Es como `void` en C, `()` en Rust, etc.
  Unit,
  /// `DefaultType` es para cuando hay un error a la hora de asignar un tipo a una expresion.
  DefaultErrorType,
}

impl Type {
  pub fn to_string(&self) -> &str {
    match &self {
      Self::Int32 => "Int32",
      Self::Bool => "Bool",
      Self::Unknown => "Unknown",
      Self::Unit => "()",
      Self::DefaultErrorType => "DefaultErrorType",
    }
  }

  pub fn is_number(&self) -> bool {
    match &self {
      Self::Int32 => true,
      _ => false,
    }
  }

  pub fn is_boolean(&self) -> bool {
    match &self {
      Self::Bool => true,
      _ => false,
    }
  }
}

impl Display for Type {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl From<ConstValue> for Type {
  fn from(value: ConstValue) -> Self {
    match value {
      ConstValue::Int32(_) => Self::Int32,
      ConstValue::Bool(_) => Self::Bool,
    }
  }
}
