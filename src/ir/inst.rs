// Responsabilidad: Definir las instrucciones de la IR.

use std::fmt;

use crate::{
  ast::{BinaryOp, UnaryOp},
  ir::{
    ids::{BlockId, ValueId},
    value::IrConstant,
  },
};

#[derive(Debug, Clone)]
pub(crate) struct InstData {
  pub(crate) result: Option<ValueId>,
  pub(crate) kind: InstKind,
}

impl InstData {
  pub(crate) fn with_result(result: ValueId, kind: InstKind) -> Self {
    debug_assert!(
      kind.produces_value(),
      "la instruccion IR no debe tener resultado, pero se hallo {kind}"
    );
    Self {
      result: Some(result),
      kind,
    }
  }

  pub(crate) fn without_result(kind: InstKind) -> Self {
    debug_assert!(
      !kind.produces_value(),
      "la instruccion IR debe tener resultado, pero se hallo {kind}"
    );
    Self { result: None, kind }
  }
}

#[derive(Debug, Clone)]
pub(crate) enum InstKind {
  // valores
  Const(IrConstant),
  Copy(ValueId),

  // operaciones unarias
  Unary {
    op: UnaryOp,
    operand: ValueId,
  },

  // operaciones binarias
  Binary {
    op: BinaryOp,
    lhs: ValueId,
    rhs: ValueId,
  },

  // control
  Jump {
    target: BlockId,
  },

  Branch {
    condition: ValueId,
    if_block: BlockId,
    else_block: BlockId,
  },

  Phi {
    inputs: Vec<PhiInput>,
  },

  Return {
    value: Option<ValueId>,
  },

  Print(ValueId),
}

impl InstKind {
  pub(crate) fn produces_value(&self) -> bool {
    matches!(
      self,
      Self::Const(_) | Self::Copy(_) | Self::Phi { .. } | Self::Unary { .. } | Self::Binary { .. }
    )
  }

  pub(crate) fn is_terminator(&self) -> bool {
    matches!(
      &self,
      Self::Jump { .. } | Self::Branch { .. } | Self::Return { .. }
    )
  }

  pub(crate) fn is_phi(&self) -> bool {
    matches!(&self, Self::Phi { .. })
  }
}

impl fmt::Display for InstKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let out = match self {
      InstKind::Const(_) => "const",
      InstKind::Copy(_) => "copy",
      InstKind::Unary { op, .. } => &op.to_string(),
      InstKind::Binary { op, .. } => &op.to_string(),
      InstKind::Phi { .. } => "phi",
      InstKind::Jump { .. } => "jump",
      InstKind::Branch { .. } => "branch",
      InstKind::Print(_) => "print",
      InstKind::Return { .. } => "return",
    };
    write!(f, "{out}")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PhiInput {
  pred_block: BlockId,
  value: ValueId,
}

impl PhiInput {
  pub(crate) fn new(pred_block: BlockId, value: ValueId) -> Self {
    Self { pred_block, value }
  }
}
