// Responsabilidad: Definir las instrucciones de la IR.

use std::fmt::{self, Display};

use crate::{
  ast::{BinaryOp, UnaryOp},
  ir::{
    ids::{BlockId, ValueId},
    types::IrType,
    value::IrConstant,
  },
};

#[derive(Debug, Clone, PartialEq)]
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

impl Display for InstData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.result {
      Some(v) => {
        let ty = self
          .kind
          .produced_value_type()
          .map_or_else(|| "<?>".to_string(), |ty| ty.to_string());
        write!(f, "{v}: {ty} = {}", self.kind)
      }
      None => write!(f, "{}", self.kind),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InstKind {
  // valores
  Const(IrConstant),
  #[allow(dead_code)]
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

  fn produced_value_type(&self) -> Option<IrType> {
    match self {
      Self::Const(IrConstant::Unit) => Some(IrType::Unit),
      Self::Const(IrConstant::Int32(_)) => Some(IrType::Int32),
      Self::Const(IrConstant::Bool(_)) => Some(IrType::Bool),
      Self::Unary { op, .. } => Some(op.result_type().into()),
      Self::Binary { op, .. } => Some(op.result_type().into()),
      Self::Copy(_) | Self::Phi { .. } => None,
      Self::Jump { .. } | Self::Branch { .. } | Self::Return { .. } | Self::Print(_) => None,
    }
  }
}

impl fmt::Display for InstKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      InstKind::Const(c) => write!(f, "const {c}"),
      InstKind::Copy(v) => write!(f, "copy {v}"),
      InstKind::Unary { op, operand } => write!(f, "{op} {operand}"),
      InstKind::Binary { op, lhs, rhs } => write!(f, "{lhs} {op} {rhs}"),
      InstKind::Phi { inputs } => {
        let mut out = String::from("phi");
        for input in inputs {
          out.push_str(&input.to_string());
        }
        write!(f, "{out}")
      }
      InstKind::Jump { target } => write!(f, "jump {target}"),
      InstKind::Branch {
        condition,
        if_block,
        else_block,
      } => write!(f, "branch {condition}, {if_block}, {else_block}"),
      InstKind::Print(v) => write!(f, "print {v}"),
      InstKind::Return { value: Some(v) } => write!(f, "return {v}"),
      InstKind::Return { .. } => write!(f, "return"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PhiInput {
  pred_block: BlockId,
  value: ValueId,
}

impl Display for PhiInput {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[{} -> {}]", self.pred_block, self.value)
  }
}

impl PhiInput {
  pub(crate) fn new(pred_block: BlockId, value: ValueId) -> Self {
    Self { pred_block, value }
  }

  pub(crate) fn pred_block(&self) -> BlockId {
    self.pred_block
  }

  pub(crate) fn value(&self) -> ValueId {
    self.value
  }
}
