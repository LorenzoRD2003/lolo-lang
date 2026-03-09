// Responsabilidad: Definir el sitema de tipos de la IR.

use crate::semantic::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IrType {
  Unit,
  Int32,
  Bool,
  Never,
}

impl IrType {
  fn is_unit(&self) -> bool {
    matches!(self, IrType::Unit)
  }

  fn is_numeric(&self) -> bool {
    matches!(self, IrType::Int32)
  }

  fn is_boolean(&self) -> bool {
    matches!(self, IrType::Bool)
  }
}

impl From<Type> for IrType {
  fn from(value: Type) -> Self {
    match value {
      Type::Int32 => Self::Int32,
      Type::Bool => Self::Bool,
      Type::Unit => Self::Unit,
      Type::DefaultErrorType => Self::Never,
    }
  }
}
