use std::collections::BTreeSet;

use crate::{
  analysis::CfgError,
  ast::{BinaryOp, UnaryOp},
  diagnostics::{Diagnosable, Diagnostic},
  ir::{
    ids::{BlockId, InstId, ValueId},
    types::IrType,
  },
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IrInvariantError {
  MissingEntryBlock,
  InvalidEntryBlock {
    entry_block: BlockId,
  },
  DuplicateInstInBlock {
    block_id: BlockId,
    inst_id: InstId,
  },
  PhiSectionContainsNonPhi {
    inst_id: InstId,
    block_id: BlockId,
  },
  InstSectionContainsTerminator {
    inst_id: InstId,
    block_id: BlockId,
  },
  InstSectionContainsPhi {
    inst_id: InstId,
    block_id: BlockId,
  },
  BlockMissingTerminator {
    block_id: BlockId,
  },
  TerminatorIsNotTerminator {
    term_id: InstId,
    block_id: BlockId,
  },
  InvalidInstId {
    id: InstId,
    context: String,
  },
  InvalidValueId {
    id: ValueId,
    context: String,
  },
  InvalidBlockId {
    id: BlockId,
    context: String,
  },
  InstProducesValueMissingResult {
    inst_id: InstId,
  },
  InstDoesNotProduceValueHasResult {
    inst_id: InstId,
  },
  BranchConditionTypeMismatch {
    inst_id: InstId,
    cond_ty: IrType,
  },
  CopyTypeMismatch {
    inst_id: InstId,
    result_ty: IrType,
    operand_ty: IrType,
  },
  UnaryTypeMismatch {
    inst_id: InstId,
    op: UnaryOp,
    operand_ty: IrType,
    result_ty: IrType,
  },
  BinaryTypeMismatch {
    inst_id: InstId,
    op: BinaryOp,
    lhs_ty: IrType,
    rhs_ty: IrType,
    result_ty: IrType,
  },
  PhiInputTypeMismatch {
    inst_id: InstId,
    input_index: usize,
    input_ty: IrType,
    result_ty: IrType,
  },
  ReturnTypeMismatch {
    inst_id: InstId,
    returned_ty: IrType,
    module_return_ty: IrType,
  },
  ReturnWithoutValueInNonUnit {
    inst_id: InstId,
    module_return_ty: IrType,
  },
  PhiDuplicatePredecessor {
    phi_id: InstId,
    block_id: BlockId,
    pred_block: BlockId,
  },
  PhiInputNotRealPredecessor {
    phi_id: InstId,
    block_id: BlockId,
    pred_block: BlockId,
  },
  PhiInputInvalidValueId {
    phi_id: InstId,
    input_index: usize,
    value_id: ValueId,
  },
  PhiDoesNotCoverExactlyPredecessors {
    phi_id: InstId,
    block_id: BlockId,
    expected_preds: BTreeSet<usize>,
    obtained_preds: BTreeSet<usize>,
  },
  DominatorsMissingImmediateDominator {
    block_id: BlockId,
  },
  PhiBlockNotInDominanceFrontier {
    phi_id: InstId,
    block_id: BlockId,
  },
  OperandDoesNotDominateUse {
    inst_id: InstId,
    operand: ValueId,
    def_block: BlockId,
    use_block: BlockId,
  },
  PhiOperandDoesNotDominatePredecessor {
    phi_id: InstId,
    operand: ValueId,
    def_block: BlockId,
    pred_block: BlockId,
  },
  Cfg(CfgError),
}

impl Diagnosable for IrInvariantError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::MissingEntryBlock => Diagnostic::error("el modulo no tiene entry block".to_string()),
      Self::InvalidEntryBlock { entry_block } => {
        Diagnostic::error(format!("el entry block {:?} no existe", entry_block))
      }
      Self::DuplicateInstInBlock { block_id, inst_id } => Diagnostic::error(format!(
        "el bloque {:?} contiene InstId {:?} repetido",
        block_id, inst_id
      )),
      Self::PhiSectionContainsNonPhi { inst_id, block_id } => Diagnostic::error(format!(
        "la instruccion {:?} esta en la seccion phi de {:?}, pero no es Phi",
        inst_id, block_id
      )),
      Self::InstSectionContainsTerminator { inst_id, block_id } => Diagnostic::error(format!(
        "la instruccion {:?} esta en insts de {:?}, pero es terminadora",
        inst_id, block_id
      )),
      Self::InstSectionContainsPhi { inst_id, block_id } => Diagnostic::error(format!(
        "la instruccion {:?} esta en insts de {:?}, pero es Phi",
        inst_id, block_id
      )),
      Self::BlockMissingTerminator { block_id } => {
        Diagnostic::error(format!("el bloque {:?} no tiene terminador", block_id))
      }
      Self::TerminatorIsNotTerminator { term_id, block_id } => Diagnostic::error(format!(
        "la instruccion {:?} es terminador de {:?}, pero no es terminadora",
        term_id, block_id
      )),
      Self::InvalidInstId { id, context } => {
        Diagnostic::error(format!("InstId {:?} invalido en {}", id, context))
      }
      Self::InvalidValueId { id, context } => {
        Diagnostic::error(format!("ValueId {:?} invalido en {}", id, context))
      }
      Self::InvalidBlockId { id, context } => {
        Diagnostic::error(format!("BlockId {:?} invalido en {}", id, context))
      }
      Self::InstProducesValueMissingResult { inst_id } => Diagnostic::error(format!(
        "la instruccion {:?} produce valor pero no tiene result",
        inst_id
      )),
      Self::InstDoesNotProduceValueHasResult { inst_id } => Diagnostic::error(format!(
        "la instruccion {:?} no produce valor pero tiene result",
        inst_id
      )),
      Self::BranchConditionTypeMismatch { inst_id, cond_ty } => Diagnostic::error(format!(
        "Branch {:?} tiene condicion de tipo {:?}, se esperaba Bool",
        inst_id, cond_ty
      )),
      Self::CopyTypeMismatch {
        inst_id,
        result_ty,
        operand_ty,
      } => Diagnostic::error(format!(
        "Copy {:?} tiene tipos incompatibles: result {:?}, operand {:?}",
        inst_id, result_ty, operand_ty
      )),
      Self::UnaryTypeMismatch {
        inst_id,
        op,
        operand_ty,
        result_ty,
      } => Diagnostic::error(format!(
        "Unary {:?} invalida para {:?}: operand {:?}, result {:?}",
        inst_id, op, operand_ty, result_ty
      )),
      Self::BinaryTypeMismatch {
        inst_id,
        op,
        lhs_ty,
        rhs_ty,
        result_ty,
      } => Diagnostic::error(format!(
        "Binary {:?} invalida para {}: lhs {:?}, rhs {:?}, result {:?}",
        inst_id, op, lhs_ty, rhs_ty, result_ty
      )),
      Self::PhiInputTypeMismatch {
        inst_id,
        input_index,
        input_ty,
        result_ty,
      } => Diagnostic::error(format!(
        "Phi {:?} input[{input_index}] tiene tipo {:?}, se esperaba {:?}",
        inst_id, input_ty, result_ty
      )),
      Self::ReturnTypeMismatch {
        inst_id,
        returned_ty,
        module_return_ty,
      } => Diagnostic::error(format!(
        "Return {:?} devuelve {:?}, pero el modulo retorna {:?}",
        inst_id, returned_ty, module_return_ty
      )),
      Self::ReturnWithoutValueInNonUnit {
        inst_id,
        module_return_ty,
      } => Diagnostic::error(format!(
        "Return {:?} sin valor en modulo con retorno {:?}",
        inst_id, module_return_ty
      )),
      Self::PhiDuplicatePredecessor {
        phi_id,
        block_id,
        pred_block,
      } => Diagnostic::error(format!(
        "Phi {:?} en {:?} tiene predecessor duplicado {:?}",
        phi_id, block_id, pred_block
      )),
      Self::PhiInputNotRealPredecessor {
        phi_id,
        block_id,
        pred_block,
      } => Diagnostic::error(format!(
        "Phi {:?} en {:?} usa predecessor {:?} que no es predecessor real del bloque",
        phi_id, block_id, pred_block
      )),
      Self::PhiInputInvalidValueId {
        phi_id,
        input_index,
        value_id,
      } => Diagnostic::error(format!(
        "Phi {:?} input[{input_index}] referencia ValueId invalido {:?}",
        phi_id, value_id
      )),
      Self::PhiDoesNotCoverExactlyPredecessors {
        phi_id,
        block_id,
        expected_preds,
        obtained_preds,
      } => Diagnostic::error(format!(
        "Phi {:?} en {:?} no cubre exactamente los predecesores del bloque: esperados {:?}, obtenidos {:?}",
        phi_id, block_id, expected_preds, obtained_preds
      )),
      Self::DominatorsMissingImmediateDominator { block_id } => Diagnostic::error(format!(
        "Dominators invalido: bloque alcanzable {:?} no tiene immediate dominator",
        block_id
      )),
      Self::PhiBlockNotInDominanceFrontier { phi_id, block_id } => Diagnostic::error(format!(
        "Phi {:?} en {:?} no cae en la dominance frontier de ningun predecesor",
        phi_id, block_id
      )),
      Self::OperandDoesNotDominateUse {
        inst_id,
        operand,
        def_block,
        use_block,
      } => Diagnostic::error(format!(
        "Inst {:?} usa operando {:?} definido en {:?}, pero {:?} no domina al bloque de uso {:?}",
        inst_id, operand, def_block, def_block, use_block
      )),
      Self::PhiOperandDoesNotDominatePredecessor {
        phi_id,
        operand,
        def_block,
        pred_block,
      } => Diagnostic::error(format!(
        "Phi {:?} usa operando {:?} definido en {:?}, pero {:?} no domina al bloque predecesor {:?}",
        phi_id, operand, def_block, def_block, pred_block
      )),
      Self::Cfg(error) => error.to_diagnostic(),
    }
  }
}
