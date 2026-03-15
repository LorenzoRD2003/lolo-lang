// Responsabilidad: Chequear invariantes internas de la IR.
// No vuelve a reportar errores de parsing o semantica salvo chequeos
// de robustez defensiva.
// Importante: se busca evitar cascadas de errores, cortando validaciones
// dependientes una vez que hallamos algo invalido.
// Se garantiza validacion cruzada entre el CFG y los Phi.

use std::collections::BTreeSet;

use crate::{
  analysis::cfg::Cfg,
  ast::{BinaryOp, UnaryOp},
  diagnostics::{Diagnosable, Diagnostic},
  ir::{
    ids::{BlockId, InstId, ValueId},
    inst::{InstKind, PhiInput},
    ir_invariant_error::IrInvariantError,
    module::IrModule,
    types::IrType,
  },
};

impl IrModule {
  pub(crate) fn verify(&self, diagnostics: &mut Vec<Diagnostic>) {
    let mut errors = Vec::new();

    let Some(entry_block) = self.entry_block_opt() else {
      errors.push(IrInvariantError::MissingEntryBlock);
      self.flush_verify_errors(errors, diagnostics);
      return;
    };
    if !self.is_valid_block_id(entry_block) {
      errors.push(IrInvariantError::InvalidEntryBlock { entry_block });
      self.flush_verify_errors(errors, diagnostics);
      return;
    }

    let mut block_inst_ids: Vec<Vec<InstId>> = vec![vec![]; self.block_count()];

    for block_index in 0..self.block_count() {
      let block_id = BlockId(block_index);
      let block = self.block(block_id);

      let mut seen_ids = BTreeSet::new();

      for &phi_id in block.phis() {
        block_inst_ids[block_index].push(phi_id);
        if !self.check_inst_id(phi_id, &mut errors, format!("phi de {:?}", block_id)) {
          continue;
        }
        self.check_duplicate_inst_in_block(&mut seen_ids, block_id, phi_id, &mut errors);

        let inst = self.inst(phi_id);
        if !matches!(inst.kind, InstKind::Phi { .. }) {
          errors.push(IrInvariantError::PhiSectionContainsNonPhi {
            inst_id: phi_id,
            block_id,
          });
        }
        self.check_inst_result_shape(phi_id, &mut errors);
        self.check_inst_operands_and_types(phi_id, &mut errors);
      }

      for &inst_id in block.insts() {
        block_inst_ids[block_index].push(inst_id);
        if !self.check_inst_id(inst_id, &mut errors, format!("inst de {:?}", block_id)) {
          continue;
        }
        self.check_duplicate_inst_in_block(&mut seen_ids, block_id, inst_id, &mut errors);

        let inst = self.inst(inst_id);
        if inst.kind.is_terminator() {
          errors.push(IrInvariantError::InstSectionContainsTerminator { inst_id, block_id });
        }
        if matches!(inst.kind, InstKind::Phi { .. }) {
          errors.push(IrInvariantError::InstSectionContainsPhi { inst_id, block_id });
        }
        self.check_inst_result_shape(inst_id, &mut errors);
        self.check_inst_operands_and_types(inst_id, &mut errors);
      }

      if !block.has_terminator() {
        errors.push(IrInvariantError::BlockMissingTerminator { block_id });
        continue;
      }

      let term_id = block.terminator();
      block_inst_ids[block_index].push(term_id);
      if !self.check_inst_id(
        term_id,
        &mut errors,
        format!("terminador de {:?}", block_id),
      ) {
        continue;
      }
      self.check_duplicate_inst_in_block(&mut seen_ids, block_id, term_id, &mut errors);

      let term = self.inst(term_id);
      if !term.kind.is_terminator() {
        errors.push(IrInvariantError::TerminatorIsNotTerminator { term_id, block_id });
      }
      self.check_inst_result_shape(term_id, &mut errors);
      self.check_inst_operands_and_types(term_id, &mut errors);
    }

    let cfg = Cfg::build(self, entry_block, &mut errors);

    for block_index in 0..self.block_count() {
      let block_id = BlockId(block_index);
      let block = self.block(block_id);
      let expected_preds: BTreeSet<usize> =
        cfg.predecessors(block_id).iter().map(|b| b.0).collect();

      for &phi_id in block.phis() {
        if !self.is_valid_inst_id(phi_id) {
          continue;
        }
        let InstKind::Phi { inputs } = &self.inst(phi_id).kind else {
          continue;
        };
        self.check_phi_predecessors(phi_id, block_id, inputs, &expected_preds, &mut errors);
      }
    }

    self.flush_verify_errors(errors, diagnostics);
  }

  /// Extiende el vector de diagnostics con los errores acumulados
  fn flush_verify_errors(&self, errors: Vec<IrInvariantError>, diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.extend(errors.iter().map(Diagnosable::to_diagnostic));
  }

  /// Determina si hay una instruccion duplicada en un bloque.
  fn check_duplicate_inst_in_block(
    &self,
    seen_ids: &mut BTreeSet<usize>,
    block_id: BlockId,
    inst_id: InstId,
    errors: &mut Vec<IrInvariantError>,
  ) {
    if !seen_ids.insert(inst_id.0) {
      errors.push(IrInvariantError::DuplicateInstInBlock { block_id, inst_id });
    }
  }

  /// Determina si un `BlockId` es valido
  fn is_valid_block_id(&self, id: BlockId) -> bool {
    id.0 < self.block_count()
  }

  /// Determina si un `InstId` es valido
  fn is_valid_inst_id(&self, id: InstId) -> bool {
    id.0 < self.inst_count()
  }

  /// Determina si un `ValueId` es valido
  fn is_valid_value_id(&self, id: ValueId) -> bool {
    id.0 < self.value_count()
  }

  /// Devuelve el `IrType` de un valor, en caso de ser valido.
  fn value_type(&self, id: ValueId) -> Option<IrType> {
    if self.is_valid_value_id(id) {
      Some(*self.value(id).ty())
    } else {
      None
    }
  }

  /// Devuelve el tipo de una instruccion
  fn check_inst_id(&self, id: InstId, errors: &mut Vec<IrInvariantError>, context: String) -> bool {
    if self.is_valid_inst_id(id) {
      true
    } else {
      errors.push(IrInvariantError::InvalidInstId { id, context });
      false
    }
  }

  /// Verifica la validez de un `ValueId`. Recibe contexto para añadir al error
  /// en caso de ser invalido.
  fn check_value_id(
    &self,
    id: ValueId,
    errors: &mut Vec<IrInvariantError>,
    context: String,
  ) -> bool {
    if self.is_valid_value_id(id) {
      true
    } else {
      self.emit_invalid_value_id(id, errors, context);
      false
    }
  }

  fn emit_invalid_value_id(
    &self,
    id: ValueId,
    errors: &mut Vec<IrInvariantError>,
    context: String,
  ) {
    errors.push(IrInvariantError::InvalidValueId { id, context });
  }

  fn checked_value_type(
    &self,
    id: ValueId,
    errors: &mut Vec<IrInvariantError>,
    context: String,
  ) -> Option<IrType> {
    if self.check_value_id(id, errors, context.clone()) {
      self.value_type(id)
    } else {
      None
    }
  }

  /// Verifica la validez de un `BlockId`. Recibe contexto para añadir al error
  /// en caso de ser invalido.
  fn check_block_id(
    &self,
    id: BlockId,
    errors: &mut Vec<IrInvariantError>,
    context: String,
  ) -> bool {
    if self.is_valid_block_id(id) {
      true
    } else {
      errors.push(IrInvariantError::InvalidBlockId { id, context });
      false
    }
  }

  /// Verifica los contenidos de una instruccion
  fn check_inst_result_shape(&self, inst_id: InstId, errors: &mut Vec<IrInvariantError>) {
    let inst = self.inst(inst_id);
    if inst.kind.produces_value() {
      match inst.result {
        Some(value_id) => {
          self.check_value_id(
            value_id,
            errors,
            format!("resultado de la instruccion {:?}", inst_id),
          );
        }
        None => errors.push(IrInvariantError::InstProducesValueMissingResult { inst_id }),
      }
    } else if inst.result.is_some() {
      errors.push(IrInvariantError::InstDoesNotProduceValueHasResult { inst_id });
    }
  }

  /// Verifica los operandos y tipos de una instruccion.
  fn check_inst_operands_and_types(&self, inst_id: InstId, errors: &mut Vec<IrInvariantError>) {
    let inst = self.inst(inst_id);
    match &inst.kind {
      InstKind::Const(_) => {}
      InstKind::Copy(value_id) => {
        self.check_copy_type(inst_id, *value_id, errors);
      }
      InstKind::Unary { op, operand } => {
        self.check_unary_types(inst_id, *op, *operand, errors);
      }
      InstKind::Binary { op, lhs, rhs } => {
        self.check_binary_types(inst_id, *op, *lhs, *rhs, errors);
      }
      InstKind::Jump { target } => {
        self.check_block_id(*target, errors, format!("Jump {:?}", inst_id));
      }
      InstKind::Branch {
        condition,
        if_block,
        else_block,
      } => {
        let cond_ty = self.checked_value_type(
          *condition,
          errors,
          format!("Branch.condition {:?}", inst_id),
        );
        self.check_block_id(*if_block, errors, format!("Branch.if_block {:?}", inst_id));
        self.check_block_id(
          *else_block,
          errors,
          format!("Branch.else_block {:?}", inst_id),
        );
        if let Some(cond_ty) = cond_ty
          && cond_ty != IrType::Bool
        {
          errors.push(IrInvariantError::BranchConditionTypeMismatch { inst_id, cond_ty });
        }
      }
      InstKind::Phi { inputs } => {
        for (index, input) in inputs.iter().enumerate() {
          self.check_block_id(
            input.pred_block(),
            errors,
            format!("Phi {:?} input[{index}].pred_block", inst_id),
          );
          self.check_value_id(
            input.value(),
            errors,
            format!("Phi {:?} input[{index}].value", inst_id),
          );
        }
        self.check_phi_input_types(inst_id, inputs, errors);
      }
      InstKind::Return { value } => self.check_return_type(inst_id, *value, errors),
      InstKind::Print(value_id) => {
        self.check_value_id(*value_id, errors, format!("Print {:?}", inst_id));
      }
    }
  }

  /// Verifica el tipode una instruccion de copia.
  fn check_copy_type(&self, inst_id: InstId, operand: ValueId, errors: &mut Vec<IrInvariantError>) {
    let Some(result_id) = self.inst(inst_id).result else {
      errors.push(IrInvariantError::InstProducesValueMissingResult { inst_id });
      return;
    };
    let Some(result_ty) =
      self.checked_value_type(result_id, errors, format!("Copy.result {:?}", inst_id))
    else {
      return;
    };
    let Some(operand_ty) =
      self.checked_value_type(operand, errors, format!("Copy.operand {:?}", inst_id))
    else {
      return;
    };

    if result_ty != operand_ty {
      errors.push(IrInvariantError::CopyTypeMismatch {
        inst_id,
        result_ty,
        operand_ty,
      });
    }
  }

  /// Verifica los tipos de las operaciones unarias.
  fn check_unary_types(
    &self,
    inst_id: InstId,
    op: UnaryOp,
    operand: ValueId,
    errors: &mut Vec<IrInvariantError>,
  ) {
    let Some(result_id) = self.inst(inst_id).result else {
      errors.push(IrInvariantError::InstProducesValueMissingResult { inst_id });
      return;
    };
    let Some(result_ty) =
      self.checked_value_type(result_id, errors, format!("Unary.result {:?}", inst_id))
    else {
      return;
    };
    let Some(operand_ty) =
      self.checked_value_type(operand, errors, format!("Unary.operand {:?}", inst_id))
    else {
      return;
    };
    let (expected_operand, expected_result) = match op {
      UnaryOp::Neg => (IrType::Int32, IrType::Int32),
      UnaryOp::Not => (IrType::Bool, IrType::Bool),
    };
    if operand_ty != expected_operand || result_ty != expected_result {
      errors.push(IrInvariantError::UnaryTypeMismatch {
        inst_id,
        op,
        operand_ty,
        result_ty,
      });
    }
  }

  /// Verifica los tipos de las operaciones binarias.
  fn check_binary_types(
    &self,
    inst_id: InstId,
    op: BinaryOp,
    lhs: ValueId,
    rhs: ValueId,
    errors: &mut Vec<IrInvariantError>,
  ) {
    let Some(result_id) = self.inst(inst_id).result else {
      errors.push(IrInvariantError::InstProducesValueMissingResult { inst_id });
      return;
    };
    let Some(result_ty) =
      self.checked_value_type(result_id, errors, format!("Binary.result {:?}", inst_id))
    else {
      return;
    };
    let Some(lhs_ty) = self.checked_value_type(lhs, errors, format!("Binary.lhs {:?}", inst_id))
    else {
      return;
    };
    let Some(rhs_ty) = self.checked_value_type(rhs, errors, format!("Binary.rhs {:?}", inst_id))
    else {
      return;
    };

    let ok = match op {
      BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
        lhs_ty == IrType::Int32 && rhs_ty == IrType::Int32 && result_ty == IrType::Int32
      }
      BinaryOp::Gt | BinaryOp::Lt | BinaryOp::Gte | BinaryOp::Lte => {
        lhs_ty == IrType::Int32 && rhs_ty == IrType::Int32 && result_ty == IrType::Bool
      }
      BinaryOp::Eq | BinaryOp::Neq => {
        ((lhs_ty == IrType::Int32 && rhs_ty == IrType::Int32)
          || (lhs_ty == IrType::Bool && rhs_ty == IrType::Bool))
          && result_ty == IrType::Bool
      }
      BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
        lhs_ty == IrType::Bool && rhs_ty == IrType::Bool && result_ty == IrType::Bool
      }
    };

    if !ok {
      errors.push(IrInvariantError::BinaryTypeMismatch {
        inst_id,
        op,
        lhs_ty,
        rhs_ty,
        result_ty,
      });
    }
  }

  /// Verifica los tipos de los `PhiInput` de una instruccion.
  /// En particular, todos tienen que tener el mismo tipo que el del resultado
  /// de la instruccion.
  fn check_phi_input_types(
    &self,
    inst_id: InstId,
    inputs: &[PhiInput],
    errors: &mut Vec<IrInvariantError>,
  ) {
    let Some(result_id) = self.inst(inst_id).result else {
      errors.push(IrInvariantError::InstProducesValueMissingResult { inst_id });
      return;
    };
    let Some(result_ty) =
      self.checked_value_type(result_id, errors, format!("Phi.result {:?}", inst_id))
    else {
      return;
    };
    for (index, input) in inputs.iter().enumerate() {
      let Some(input_ty) = self.checked_value_type(
        input.value(),
        errors,
        format!("Phi {:?} input[{index}].value", inst_id),
      ) else {
        continue;
      };
      if input_ty != result_ty {
        errors.push(IrInvariantError::PhiInputTypeMismatch {
          inst_id,
          input_index: index,
          input_ty,
          result_ty,
        });
      }
    }
  }

  /// Verifica el tipo de una instruccion `Return`. En particular, debe ser
  /// el mismo tipo que el `return_type`` del `IrModule`.
  fn check_return_type(
    &self,
    inst_id: InstId,
    returned_value: Option<ValueId>,
    errors: &mut Vec<IrInvariantError>,
  ) {
    match returned_value {
      Some(value_id) => {
        if let Some(ty) = self.checked_value_type(value_id, errors, format!("Return {:?}", inst_id))
          && ty != self.return_type()
        {
          errors.push(IrInvariantError::ReturnTypeMismatch {
            inst_id,
            returned_ty: ty,
            module_return_ty: self.return_type(),
          });
        }
      }
      None => {
        if self.return_type() != IrType::Unit {
          errors.push(IrInvariantError::ReturnWithoutValueInNonUnit {
            inst_id,
            module_return_ty: self.return_type(),
          });
        }
      }
    }
  }

  /// Valida que un `phi` este bien conectado al CFG del bloque donde vive.
  fn check_phi_predecessors(
    &self,
    phi_id: InstId,
    block_id: BlockId,
    inputs: &[PhiInput],
    expected_preds: &BTreeSet<usize>,
    errors: &mut Vec<IrInvariantError>,
  ) {
    let mut seen_input_preds = BTreeSet::new();
    for (index, input) in inputs.iter().enumerate() {
      let pred = input.pred_block();
      if !seen_input_preds.insert(pred.0) {
        errors.push(IrInvariantError::PhiDuplicatePredecessor {
          phi_id,
          block_id,
          pred_block: pred,
        });
      }
      if !expected_preds.contains(&pred.0) {
        errors.push(IrInvariantError::PhiInputNotRealPredecessor {
          phi_id,
          block_id,
          pred_block: pred,
        });
      }
      if !self.is_valid_value_id(input.value()) {
        errors.push(IrInvariantError::PhiInputInvalidValueId {
          phi_id,
          input_index: index,
          value_id: input.value(),
        });
      }
    }

    if seen_input_preds != *expected_preds {
      errors.push(IrInvariantError::PhiDoesNotCoverExactlyPredecessors {
        phi_id,
        block_id,
        expected_preds: expected_preds.clone(),
        obtained_preds: seen_input_preds,
      });
    }
  }
}

#[cfg(test)]
mod tests;
