use crate::ir::types::IrType;

/// Datos necesarios para representar una variable local en la IR.
#[derive(Debug, Clone)]
pub(crate) struct LocalData {
  name: String,
  ty: IrType,
  mutable: bool,
}

impl LocalData {
  pub(crate) fn new(name: String, ty: IrType, mutable: bool) -> Self {
    Self {
      name,
      ty,
      mutable
    }
  }

  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  pub(crate) fn ty(&self) -> &IrType {
    &self.ty
  }

  pub(crate) fn is_mutable(&self) -> bool {
    self.mutable
  }
}
