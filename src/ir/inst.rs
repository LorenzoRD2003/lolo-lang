// Responsabilidad: Definir las instrucciones de la IR.

use std::fmt;

use crate::{
  ast::{BinaryOp, UnaryOp},
  ir::{
    ids::{BlockId, LocalId, ValueId},
    value::{Constant, IrOperand},
  },
};

#[derive(Debug, Clone)]
pub(crate) struct InstData {
  pub(crate) result: Option<ValueId>,
  pub(crate) kind: InstKind,
} // hay que crear un error en error.rs. debe ser IrError, e implementar el trait Diagnosable de la misma forma
// que hacen otros errores

impl InstData {
  pub(crate) fn inst_with_result(result: ValueId, kind: InstKind) -> Self {
    debug_assert!(
      kind.produces_value(),
      "la instruccion IR no debe tener resultado, pero se hallo {kind}"
    );
    Self {
      result: Some(result),
      kind,
    }
  }

  pub(crate) fn inst_without_result(kind: InstKind) -> Self {
    debug_assert!(
      kind.produces_value(),
      "la instruccion IR debe tener resultado, pero se hallo {kind}"
    );
    Self { result: None, kind }
  }
}

#[derive(Debug, Clone)]
pub(crate) enum InstKind {
  // valores
  Const(Constant),
  Copy(IrOperand),

  // variables locales
  Load(LocalId),
  Store {
    target: LocalId,
    operand: IrOperand,
  },

  // operaciones unarias
  Unary {
    op: UnaryOp,
    operand: IrOperand,
  },

  // operaciones binarias
  Binary {
    op: BinaryOp,
    lhs: IrOperand,
    rhs: IrOperand,
  },

  // control
  Jump {
    target: BlockId,
  },

  Branch {
    condition: IrOperand,
    if_block: BlockId,
    else_block: BlockId,
  },

  Return {
    value: Option<IrOperand>,
  },
}

impl InstKind {
  pub(crate) fn produces_value(&self) -> bool {
    matches!(
      self,
      Self::Const(_) | Self::Copy(_) | Self::Load(_) | Self::Unary { .. } | Self::Binary { .. }
    )
  }

  pub(crate) fn is_terminator(&self) -> bool {
    matches!(
      &self,
      Self::Jump { .. } | Self::Branch { .. } | Self::Return { .. }
    )
  }
}

impl fmt::Display for InstKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let out = match self {
      InstKind::Const(_) => "const",
      InstKind::Copy(_) => "copy",
      InstKind::Load(_) => "load",
      InstKind::Store { .. } => "store",
      InstKind::Unary { op, .. } => &op.to_string(),
      InstKind::Binary { op, .. } => &op.to_string(),
      InstKind::Jump { .. } => "jump",
      InstKind::Branch { .. } => "branch",
      InstKind::Return { .. } => "return",
    };
    write!(f, "{out}")
  }
}
