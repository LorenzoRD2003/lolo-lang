// Responsabilidad: Chequear invariantes internas de la IR.
// No vuelve a reportar errores de parsing o semantica salvo chequeos
// de robustez defensiva.
// Importante: se busca evitar cascadas de errores, cortando validaciones
// dependientes una vez que hallamos algo invalido.
// Se garantiza validacion cruzada entre el CFG y los Phi.

use std::collections::BTreeSet;

use crate::{
  ast::{BinaryOp, UnaryOp},
  diagnostics::{Diagnosable, Diagnostic},
  ir::{
    cfg::Cfg,
    ids::{BlockId, InstId, ValueId},
    inst::{InstKind, PhiInput},
    module::IrModule,
    types::IrType,
  },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VerifyError {
  InvariantViolation(String),
}

impl VerifyError {
  pub(crate) fn new(msg: impl Into<String>) -> Self {
    Self::InvariantViolation(msg.into())
  }
}

impl Diagnosable for VerifyError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::InvariantViolation(msg) => Diagnostic::error(msg.clone()),
    }
  }
}

impl IrModule {
  pub(crate) fn verify(&self, diagnostics: &mut Vec<Diagnostic>) {
    let mut errors = Vec::new();

    let Some(entry_block) = self.entry_block_opt() else {
      errors.push(VerifyError::new("el modulo no tiene entry block"));
      self.flush_verify_errors(errors, diagnostics);
      return;
    };
    if !self.is_valid_block_id(entry_block) {
      errors.push(VerifyError::new(format!(
        "el entry block {:?} no existe",
        entry_block
      )));
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
        if !seen_ids.insert(phi_id.0) {
          errors.push(VerifyError::new(format!(
            "el bloque {:?} contiene InstId {:?} repetido",
            block_id, phi_id
          )));
        }

        let inst = self.inst(phi_id);
        if !matches!(inst.kind, InstKind::Phi { .. }) {
          errors.push(VerifyError::new(format!(
            "la instruccion {:?} esta en la seccion phi de {:?}, pero no es Phi",
            phi_id, block_id
          )));
        }
        self.check_inst_result_shape(phi_id, &mut errors);
        self.check_inst_operands_and_types(phi_id, &mut errors);
      }

      for &inst_id in block.insts() {
        block_inst_ids[block_index].push(inst_id);
        if !self.check_inst_id(inst_id, &mut errors, format!("inst de {:?}", block_id)) {
          continue;
        }
        if !seen_ids.insert(inst_id.0) {
          errors.push(VerifyError::new(format!(
            "el bloque {:?} contiene InstId {:?} repetido",
            block_id, inst_id
          )));
        }

        let inst = self.inst(inst_id);
        if inst.kind.is_terminator() {
          errors.push(VerifyError::new(format!(
            "la instruccion {:?} esta en insts de {:?}, pero es terminadora",
            inst_id, block_id
          )));
        }
        if matches!(inst.kind, InstKind::Phi { .. }) {
          errors.push(VerifyError::new(format!(
            "la instruccion {:?} esta en insts de {:?}, pero es Phi",
            inst_id, block_id
          )));
        }
        self.check_inst_result_shape(inst_id, &mut errors);
        self.check_inst_operands_and_types(inst_id, &mut errors);
      }

      if !block.has_terminator() {
        errors.push(VerifyError::new(format!(
          "el bloque {:?} no tiene terminador",
          block_id
        )));
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
      if !seen_ids.insert(term_id.0) {
        errors.push(VerifyError::new(format!(
          "el bloque {:?} contiene InstId {:?} repetido",
          block_id, term_id
        )));
      }

      let term = self.inst(term_id);
      if !term.kind.is_terminator() {
        errors.push(VerifyError::new(format!(
          "la instruccion {:?} es terminador de {:?}, pero no es terminadora",
          term_id, block_id
        )));
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

  fn flush_verify_errors(&self, errors: Vec<VerifyError>, diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.extend(errors.iter().map(Diagnosable::to_diagnostic));
  }

  fn is_valid_block_id(&self, id: BlockId) -> bool {
    id.0 < self.block_count()
  }

  fn is_valid_inst_id(&self, id: InstId) -> bool {
    id.0 < self.inst_count()
  }

  fn is_valid_value_id(&self, id: ValueId) -> bool {
    id.0 < self.value_count()
  }

  fn value_type(&self, id: ValueId) -> Option<IrType> {
    if self.is_valid_value_id(id) {
      Some(self.value(id).ty().clone())
    } else {
      None
    }
  }

  fn check_inst_id(&self, id: InstId, errors: &mut Vec<VerifyError>, context: String) -> bool {
    if self.is_valid_inst_id(id) {
      true
    } else {
      errors.push(VerifyError::new(format!(
        "InstId {:?} invalido en {}",
        id, context
      )));
      false
    }
  }

  fn check_value_id(&self, id: ValueId, errors: &mut Vec<VerifyError>, context: String) -> bool {
    if self.is_valid_value_id(id) {
      true
    } else {
      errors.push(VerifyError::new(format!(
        "ValueId {:?} invalido en {}",
        id, context
      )));
      false
    }
  }

  fn check_block_id(&self, id: BlockId, errors: &mut Vec<VerifyError>, context: String) -> bool {
    if self.is_valid_block_id(id) {
      true
    } else {
      errors.push(VerifyError::new(format!(
        "BlockId {:?} invalido en {}",
        id, context
      )));
      false
    }
  }

  fn check_inst_result_shape(&self, inst_id: InstId, errors: &mut Vec<VerifyError>) {
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
        None => errors.push(VerifyError::new(format!(
          "la instruccion {:?} produce valor pero no tiene result",
          inst_id
        ))),
      }
    } else if inst.result.is_some() {
      errors.push(VerifyError::new(format!(
        "la instruccion {:?} no produce valor pero tiene result",
        inst_id
      )));
    }
  }

  fn check_inst_operands_and_types(&self, inst_id: InstId, errors: &mut Vec<VerifyError>) {
    let inst = self.inst(inst_id);
    match &inst.kind {
      InstKind::Const(_) => {}
      InstKind::Copy(value_id) => {
        if self.check_value_id(*value_id, errors, format!("Copy {:?}", inst_id)) {
          self.check_copy_type(inst_id, *value_id, errors);
        }
      }
      InstKind::Unary { op, operand } => {
        if self.check_value_id(*operand, errors, format!("Unary {:?}", inst_id)) {
          self.check_unary_types(inst_id, *op, *operand, errors);
        }
      }
      InstKind::Binary { op, lhs, rhs } => {
        let lhs_ok = self.check_value_id(*lhs, errors, format!("Binary.lhs {:?}", inst_id));
        let rhs_ok = self.check_value_id(*rhs, errors, format!("Binary.rhs {:?}", inst_id));
        if lhs_ok && rhs_ok {
          self.check_binary_types(inst_id, *op, *lhs, *rhs, errors);
        }
      }
      InstKind::Jump { target } => {
        self.check_block_id(*target, errors, format!("Jump {:?}", inst_id));
      }
      InstKind::Branch {
        condition,
        if_block,
        else_block,
      } => {
        let cond_ok = self.check_value_id(
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
        if cond_ok
          && let Some(cond_ty) = self.value_type(*condition)
          && cond_ty != IrType::Bool
        {
          errors.push(VerifyError::new(format!(
            "Branch {:?} tiene condicion de tipo {:?}, se esperaba Bool",
            inst_id, cond_ty
          )));
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

  fn check_copy_type(&self, inst_id: InstId, operand: ValueId, errors: &mut Vec<VerifyError>) {
    let Some(result_id) = self.inst(inst_id).result else {
      return;
    };
    let Some(result_ty) = self.value_type(result_id) else {
      return;
    };
    let Some(operand_ty) = self.value_type(operand) else {
      return;
    };
    if result_ty != operand_ty {
      errors.push(VerifyError::new(format!(
        "Copy {:?} tiene tipos incompatibles: result {:?}, operand {:?}",
        inst_id, result_ty, operand_ty
      )));
    }
  }

  fn check_unary_types(
    &self,
    inst_id: InstId,
    op: UnaryOp,
    operand: ValueId,
    errors: &mut Vec<VerifyError>,
  ) {
    let Some(result_id) = self.inst(inst_id).result else {
      return;
    };
    let Some(result_ty) = self.value_type(result_id) else {
      return;
    };
    let Some(operand_ty) = self.value_type(operand) else {
      return;
    };
    let (expected_operand, expected_result) = match op {
      UnaryOp::Neg => (IrType::Int32, IrType::Int32),
      UnaryOp::Not => (IrType::Bool, IrType::Bool),
    };
    if operand_ty != expected_operand || result_ty != expected_result {
      errors.push(VerifyError::new(format!(
        "Unary {:?} invalida para {:?}: operand {:?}, result {:?}",
        inst_id, op, operand_ty, result_ty
      )));
    }
  }

  fn check_binary_types(
    &self,
    inst_id: InstId,
    op: BinaryOp,
    lhs: ValueId,
    rhs: ValueId,
    errors: &mut Vec<VerifyError>,
  ) {
    let Some(result_id) = self.inst(inst_id).result else {
      return;
    };
    let Some(result_ty) = self.value_type(result_id) else {
      return;
    };
    let Some(lhs_ty) = self.value_type(lhs) else {
      return;
    };
    let Some(rhs_ty) = self.value_type(rhs) else {
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
      errors.push(VerifyError::new(format!(
        "Binary {:?} invalida para {}: lhs {:?}, rhs {:?}, result {:?}",
        inst_id, op, lhs_ty, rhs_ty, result_ty
      )));
    }
  }

  fn check_phi_input_types(
    &self,
    inst_id: InstId,
    inputs: &[PhiInput],
    errors: &mut Vec<VerifyError>,
  ) {
    let Some(result_id) = self.inst(inst_id).result else {
      return;
    };
    let Some(result_ty) = self.value_type(result_id) else {
      return;
    };
    for (index, input) in inputs.iter().enumerate() {
      if let Some(input_ty) = self.value_type(input.value())
        && input_ty != result_ty
      {
        errors.push(VerifyError::new(format!(
          "Phi {:?} input[{index}] tiene tipo {:?}, se esperaba {:?}",
          inst_id, input_ty, result_ty
        )));
      }
    }
  }

  fn check_return_type(
    &self,
    inst_id: InstId,
    returned_value: Option<ValueId>,
    errors: &mut Vec<VerifyError>,
  ) {
    match returned_value {
      Some(value_id) => {
        if self.check_value_id(value_id, errors, format!("Return {:?}", inst_id))
          && let Some(ty) = self.value_type(value_id)
          && ty != self.return_type().clone()
        {
          errors.push(VerifyError::new(format!(
            "Return {:?} devuelve {:?}, pero el modulo retorna {:?}",
            inst_id,
            ty,
            self.return_type()
          )));
        }
      }
      None => {
        if *self.return_type() != IrType::Unit {
          errors.push(VerifyError::new(format!(
            "Return {:?} sin valor en modulo con retorno {:?}",
            inst_id,
            self.return_type()
          )));
        }
      }
    }
  }

  /// Valida que un `phi` este bien conectado al CFG del bloque donde vive.
  /// 
  fn check_phi_predecessors(
    &self,
    phi_id: InstId,
    block_id: BlockId,
    inputs: &[PhiInput],
    expected_preds: &BTreeSet<usize>,
    errors: &mut Vec<VerifyError>,
  ) {
    let mut seen_input_preds = BTreeSet::new();
    for (index, input) in inputs.iter().enumerate() {
      let pred = input.pred_block();
      if !seen_input_preds.insert(pred.0) {
        errors.push(VerifyError::new(format!(
          "Phi {:?} en {:?} tiene predecessor duplicado {:?}",
          phi_id, block_id, pred
        )));
      }
      if !expected_preds.contains(&pred.0) {
        errors.push(VerifyError::new(format!(
          "Phi {:?} en {:?} usa predecessor {:?} que no es predecessor real del bloque",
          phi_id, block_id, pred
        )));
      }
      if !self.is_valid_value_id(input.value()) {
        errors.push(VerifyError::new(format!(
          "Phi {:?} input[{index}] referencia ValueId invalido {:?}",
          phi_id,
          input.value()
        )));
      }
    }

    if seen_input_preds != *expected_preds {
      errors.push(VerifyError::new(format!(
        "Phi {:?} en {:?} no cubre exactamente los predecesores del bloque: esperados {:?}, obtenidos {:?}",
        phi_id, block_id, expected_preds, seen_input_preds
      )));
    }
  }
}

#[cfg(test)]
mod tests;
