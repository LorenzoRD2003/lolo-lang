use crate::{
  Diagnostic,
  diagnostics::Diagnosable,
  ir::{BlockId, InstId},
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CfgError {
  JumpTargetMissing {
    terminator_id: InstId,
    target: BlockId,
  },
  BranchIfTargetMissing {
    terminator_id: InstId,
    if_block: BlockId,
  },
  BranchElseTargetMissing {
    terminator_id: InstId,
    else_block: BlockId,
  },
}

impl Diagnosable for CfgError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::JumpTargetMissing {
        terminator_id,
        target,
      } => Diagnostic::error(format!(
        "Jump {:?} referencia bloque inexistente {:?}",
        terminator_id, target
      )),
      Self::BranchIfTargetMissing {
        terminator_id,
        if_block,
      } => Diagnostic::error(format!(
        "Branch {:?} referencia if_block inexistente {:?}",
        terminator_id, if_block
      )),
      Self::BranchElseTargetMissing {
        terminator_id,
        else_block,
      } => Diagnostic::error(format!(
        "Branch {:?} referencia else_block inexistente {:?}",
        terminator_id, else_block
      )),
    }
  }
}
