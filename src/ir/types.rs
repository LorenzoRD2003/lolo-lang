// Responsabilidad: Definir el sitema de tipos de la IR.

use crate::semantic::SemanticType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IrType {
  Unit,
  Int32,
  Bool,
  Never,
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
