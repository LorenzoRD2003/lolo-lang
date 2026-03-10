// Los tipos los manejo con un enum simple porque por ahora todo va a tener tipos estaticos
// y los usuarios no van a poder definir tipos.

use std::fmt::Display;

use crate::ast::ConstValue;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SemanticType {
  Int32,
  Bool,
  /// `Unit` significa "no hay valor significativo". Es como `void` en C, `()` en Rust, etc.
  Unit,
  /// `DefaultErrorType` es para cuando hay un error a la hora de asignar un tipo a una expresion.
  #[allow(clippy::enum_variant_names)]
  DefaultErrorType,
}

impl SemanticType {
  fn as_string(&self) -> &str {
    match &self {
      Self::Int32 => "Int32",
      Self::Bool => "Bool",
      Self::Unit => "()",
      Self::DefaultErrorType => "DefaultErrorType",
    }
  }

  /// El allow esta porque en el futuro el warning no estaria cuando haya mas tipos.
  #[allow(clippy::match_like_matches_macro)]
  pub(crate) fn is_number(&self) -> bool {
    match &self {
      Self::Int32 => true,
      _ => false,
    }
  }

  #[allow(clippy::match_like_matches_macro)]
  pub(crate) fn is_boolean(&self) -> bool {
    match &self {
      Self::Bool => true,
      _ => false,
    }
  }

  #[allow(clippy::match_like_matches_macro)]
  pub(crate) fn is_error(&self) -> bool {
    match &self {
      Self::DefaultErrorType => true,
      _ => false,
    }
  }
}

impl Display for SemanticType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_string())
  }
}

impl From<ConstValue> for SemanticType {
  fn from(value: ConstValue) -> Self {
    match value {
      ConstValue::Int32(_) => Self::Int32,
      ConstValue::Bool(_) => Self::Bool,
    }
  }
}
