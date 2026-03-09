// Responsabilidad: representar los valores de la IR.

use crate::ir::{
  ids::{LocalId, ValueId},
  types::IrType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueData {
  ty: IrType,
  kind: ValueKind,
}

impl ValueData {
  pub(crate) fn new(ty: IrType, kind: ValueKind) -> Self {
    Self { ty, kind }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueKind {
  Local(LocalId),
  Const(Constant),
  InstResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Constant {
  Unit,
  Boolean(bool),
  Int32(i32),
}

impl Constant {
  fn ty(&self) -> IrType {
    match self {
      Constant::Unit => IrType::Unit,
      Constant::Boolean(_) => IrType::Bool,
      Constant::Int32(_) => IrType::Int32,
    }
  }

  pub(crate) fn as_value(&self) -> ValueData {
    ValueData::new(self.ty(), ValueKind::Const(self.clone()))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IrOperand {
  Value(ValueId),
  Const(Constant),
}
