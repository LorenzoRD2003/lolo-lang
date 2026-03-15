// Responsabilidad: Definir el sitema de tipos de la IR.

use std::fmt::Display;

use crate::semantic::SemanticType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IrType {
  Unit,
  Int32,
  Bool,
  Never,
}

impl Display for IrType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let str = match self {
      IrType::Unit => "()",
      IrType::Int32 => "Int32",
      IrType::Bool => "Bool",
      IrType::Never => "Never",
    };
    write!(f, "{str}")
  }
}

impl IrType {
  #[allow(dead_code)]
  fn is_unit(&self) -> bool {
    matches!(self, IrType::Unit)
  }

  #[allow(dead_code)]
  fn is_numeric(&self) -> bool {
    matches!(self, IrType::Int32)
  }

  #[allow(dead_code)]
  fn is_boolean(&self) -> bool {
    matches!(self, IrType::Bool)
  }
}

impl From<SemanticType> for IrType {
  fn from(value: SemanticType) -> Self {
    match value {
      SemanticType::Int32 => Self::Int32,
      SemanticType::Bool => Self::Bool,
      SemanticType::Unit => Self::Unit,
      SemanticType::DefaultErrorType => Self::Never,
    }
  }
}
