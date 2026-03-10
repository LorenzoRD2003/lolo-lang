// Responsabilidad: representar los valores de la IR.

use crate::{ast::ConstValue, ir::types::IrType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueData {
  ty: IrType,
  kind: ValueKind,
}

impl ValueData {
  pub(crate) fn new(ty: IrType, kind: ValueKind) -> Self {
    Self { ty, kind }
  }

  pub(crate) fn ty(&self) -> &IrType {
    &self.ty
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueKind {
  Const(IrConstant),
  InstResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IrConstant {
  Unit,
  Int32(i32),
  Bool(bool),
}

impl From<&ConstValue> for IrConstant {
  fn from(value: &ConstValue) -> Self {
    match value {
      ConstValue::Int32(x) => Self::Int32(*x),
      ConstValue::Bool(b) => Self::Bool(*b),
    }
  }
}

impl IrConstant {
  fn ty(&self) -> IrType {
    match self {
      IrConstant::Unit => IrType::Unit,
      IrConstant::Bool(_) => IrType::Bool,
      IrConstant::Int32(_) => IrType::Int32,
    }
  }

  pub(crate) fn as_value(&self) -> ValueData {
    ValueData::new(self.ty(), ValueKind::Const(self.clone()))
  }
}
