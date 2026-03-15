use crate::{
  Diagnostic,
  ast::{BinaryOp, UnaryOp},
  ir::{
    block::BlockData,
    ids::{BlockId, InstId, ValueId},
    inst::{InstData, InstKind, PhiInput},
    module::IrModule,
    test_helpers::lower_source,
    types::IrType,
    value::IrConstant,
  },
};

fn block(id: usize) -> BlockId {
  BlockId(id)
}

fn module_with_entry(block_count: usize, return_ty: IrType) -> IrModule {
  let mut module = IrModule::new("m".into(), return_ty);
  for _ in 0..block_count {
    module.add_block(BlockData::new_block());
  }
  module.set_entry_block(block(0));
  module
}

fn append_value(module: &mut IrModule, constant: IrConstant) -> ValueId {
  let value_id = ValueId(module.value_count());
  module.add_value(constant.as_value());
  value_id
}

fn append_inst(module: &mut IrModule, data: InstData) -> InstId {
  let inst_id = InstId(module.inst_count());
  module.add_inst(data);
  inst_id
}

fn v_i32(module: &mut IrModule, value: i32) -> ValueId {
  append_value(module, IrConstant::Int32(value))
}

fn v_bool(module: &mut IrModule, value: bool) -> ValueId {
  append_value(module, IrConstant::Bool(value))
}

fn inst_const_i32(result: ValueId, value: i32) -> InstData {
  InstData::with_result(result, InstKind::Const(IrConstant::Int32(value)))
}

fn inst_return(value: Option<ValueId>) -> InstData {
  InstData::without_result(InstKind::Return { value })
}

fn inst_jump(target: BlockId) -> InstData {
  InstData::without_result(InstKind::Jump { target })
}

fn inst_branch(condition: ValueId, if_block: BlockId, else_block: BlockId) -> InstData {
  InstData::without_result(InstKind::Branch {
    condition,
    if_block,
    else_block,
  })
}

fn inst_copy(result: ValueId, operand: ValueId) -> InstData {
  InstData::with_result(result, InstKind::Copy(operand))
}

fn inst_unary(result: ValueId, op: UnaryOp, operand: ValueId) -> InstData {
  InstData::with_result(result, InstKind::Unary { op, operand })
}

fn inst_binary(result: ValueId, op: BinaryOp, lhs: ValueId, rhs: ValueId) -> InstData {
  InstData::with_result(result, InstKind::Binary { op, lhs, rhs })
}

fn inst_phi(result: ValueId, inputs: Vec<PhiInput>) -> InstData {
  InstData::with_result(result, InstKind::Phi { inputs })
}

fn add_phi_to_block(module: &mut IrModule, block_id: BlockId, data: InstData) -> InstId {
  let inst_id = append_inst(module, data);
  module.block_mut(block_id).add_phi(inst_id);
  inst_id
}

fn add_inst_to_block(module: &mut IrModule, block_id: BlockId, data: InstData) -> InstId {
  let inst_id = append_inst(module, data);
  module.block_mut(block_id).add_inst(inst_id);
  inst_id
}

fn set_terminator(module: &mut IrModule, block_id: BlockId, data: InstData) -> InstId {
  let inst_id = append_inst(module, data);
  module.block_mut(block_id).set_terminator(inst_id);
  inst_id
}

fn set_return_none(module: &mut IrModule, block_id: BlockId) -> InstId {
  set_terminator(module, block_id, inst_return(None))
}

fn verify_diagnostics(module: &IrModule) -> Vec<Diagnostic> {
  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  diagnostics
}

fn count_errors(diagnostics: &[Diagnostic], pattern: &str) -> usize {
  diagnostics
    .iter()
    .filter(|diag| diag.msg().contains(pattern))
    .count()
}

fn assert_error_once(diagnostics: &[Diagnostic], pattern: &str) {
  assert_eq!(count_errors(diagnostics, pattern), 1);
}

fn assert_has_error(diagnostics: &[Diagnostic], pattern: &str) {
  assert!(count_errors(diagnostics, pattern) > 0);
}

fn assert_error_count_at_least(diagnostics: &[Diagnostic], pattern: &str, min_count: usize) {
  assert!(count_errors(diagnostics, pattern) >= min_count);
}

#[test]
fn verify_accepts_module_lowered_by_frontend() {
  let (ir, lower_diags) =
    lower_source("main { let x = 1; if true { x = 2; } else { x = 3; }; print x; }");
  assert!(lower_diags.is_empty());
  let diagnostics = verify_diagnostics(&ir);
  assert!(diagnostics.is_empty());
}

#[test]
fn verify_accepts_module_with_else_if_expression_returns() {
  let (ir, lower_diags) = lower_source(
    r#"
    main {
      const x = if true {
        return 10;
      } else if false {
        return 20;
      } else {
        return 30;
      };
      print x;
    }
  "#,
  );
  assert!(lower_diags.is_empty());

  let diagnostics = verify_diagnostics(&ir);
  assert!(diagnostics.is_empty());
}

#[test]
fn verify_reports_missing_entry_block() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "no tiene entry block");
}

#[test]
fn verify_reports_invalid_entry_block() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  module.set_entry_block(block(1));
  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "entry block BlockId(1) no existe");
}

#[test]
fn verify_reports_duplicate_inst_in_block() {
  let mut module = module_with_entry(1, IrType::Unit);
  let value = v_i32(&mut module, 1);
  let const_inst = append_inst(&mut module, inst_const_i32(value, 1));
  module.block_mut(block(0)).add_inst(const_inst);
  module.block_mut(block(0)).add_inst(const_inst);
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "contiene InstId InstId(0) repetido");
}

#[test]
fn verify_reports_phi_section_contains_non_phi() {
  let mut module = module_with_entry(1, IrType::Unit);
  let value = v_i32(&mut module, 1);
  add_phi_to_block(&mut module, block(0), inst_const_i32(value, 1));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "seccion phi de BlockId(0), pero no es Phi");
}

#[test]
fn verify_reports_inst_section_contains_terminator() {
  let mut module = module_with_entry(1, IrType::Unit);
  add_inst_to_block(&mut module, block(0), inst_return(None));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "esta en insts de BlockId(0), pero es terminadora",
  );
}

#[test]
fn verify_reports_inst_section_contains_phi() {
  let mut module = module_with_entry(1, IrType::Unit);
  let result = v_i32(&mut module, 0);
  add_inst_to_block(&mut module, block(0), inst_phi(result, vec![]));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "esta en insts de BlockId(0), pero es Phi");
}

#[test]
fn verify_reports_block_without_terminator() {
  let module = module_with_entry(1, IrType::Unit);
  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "el bloque BlockId(0) no tiene terminador");
}

#[test]
fn verify_reports_terminator_is_not_terminator() {
  let mut module = module_with_entry(1, IrType::Unit);
  let value = v_i32(&mut module, 1);
  set_terminator(&mut module, block(0), inst_const_i32(value, 1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "es terminador de BlockId(0), pero no es terminadora",
  );
}

#[test]
fn verify_reports_invalid_inst_id() {
  let mut module = module_with_entry(1, IrType::Unit);
  module.block_mut(block(0)).add_phi(InstId(99));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "InstId InstId(99) invalido en phi de BlockId(0)",
  );
}

#[test]
fn verify_reports_invalid_value_id() {
  let mut module = module_with_entry(1, IrType::Unit);
  add_inst_to_block(
    &mut module,
    block(0),
    InstData::without_result(InstKind::Print(ValueId(99))),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "ValueId ValueId(99) invalido en Print InstId(0)",
  );
}

#[test]
fn verify_reports_invalid_block_id() {
  let mut module = module_with_entry(1, IrType::Unit);
  add_inst_to_block(&mut module, block(0), inst_jump(block(99)));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "BlockId BlockId(99) invalido en Jump InstId(0)",
  );
}

#[test]
fn verify_reports_inst_produces_value_missing_result() {
  let mut module = module_with_entry(1, IrType::Unit);
  add_inst_to_block(
    &mut module,
    block(0),
    InstData {
      result: None,
      kind: InstKind::Const(IrConstant::Int32(1)),
    },
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "produce valor pero no tiene result");
}

#[test]
fn verify_reports_inst_does_not_produce_value_has_result() {
  let mut module = module_with_entry(1, IrType::Unit);
  let result = v_i32(&mut module, 1);
  set_terminator(
    &mut module,
    block(0),
    InstData {
      result: Some(result),
      kind: InstKind::Return { value: None },
    },
  );

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "no produce valor pero tiene result");
}

#[test]
fn verify_reports_branch_condition_type_mismatch() {
  let mut module = module_with_entry(3, IrType::Unit);
  let cond = v_i32(&mut module, 1);
  set_terminator(&mut module, block(0), inst_branch(cond, block(1), block(2)));
  set_return_none(&mut module, block(1));
  set_return_none(&mut module, block(2));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "condicion de tipo");
}

#[test]
fn verify_reports_copy_type_mismatch() {
  let mut module = module_with_entry(1, IrType::Unit);
  let operand = v_bool(&mut module, true);
  let result = v_i32(&mut module, 1);
  add_inst_to_block(&mut module, block(0), inst_copy(result, operand));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "Copy InstId(0) tiene tipos incompatibles");
}

#[test]
fn verify_reports_copy_type_check_missing_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let operand = v_i32(&mut module, 1);
  add_inst_to_block(
    &mut module,
    block(0),
    InstData {
      result: None,
      kind: InstKind::Copy(operand),
    },
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_count_at_least(&diagnostics, "produce valor pero no tiene result", 2);
}

#[test]
fn verify_reports_copy_type_check_invalid_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let operand = v_i32(&mut module, 1);
  add_inst_to_block(&mut module, block(0), inst_copy(ValueId(99), operand));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Copy.result InstId(0)",
  );
}

#[test]
fn verify_reports_copy_type_check_invalid_operand_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let result = v_i32(&mut module, 0);
  add_inst_to_block(&mut module, block(0), inst_copy(result, ValueId(99)));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Copy.operand InstId(0)",
  );
}

#[test]
fn verify_reports_unary_type_mismatch() {
  let mut module = module_with_entry(1, IrType::Unit);
  let operand = v_bool(&mut module, true);
  let result = v_bool(&mut module, false);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_unary(result, UnaryOp::Neg, operand),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "Unary InstId(0) invalida");
}

#[test]
fn verify_reports_unary_type_check_missing_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let operand = v_i32(&mut module, 1);
  add_inst_to_block(
    &mut module,
    block(0),
    InstData {
      result: None,
      kind: InstKind::Unary {
        op: UnaryOp::Neg,
        operand,
      },
    },
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_count_at_least(&diagnostics, "produce valor pero no tiene result", 2);
}

#[test]
fn verify_reports_unary_type_check_invalid_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let operand = v_i32(&mut module, 1);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_unary(ValueId(99), UnaryOp::Neg, operand),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Unary.result InstId(0)",
  );
}

#[test]
fn verify_reports_unary_type_check_invalid_operand_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let result = v_i32(&mut module, 0);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_unary(result, UnaryOp::Neg, ValueId(99)),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Unary.operand InstId(0)",
  );
}

#[test]
fn verify_reports_binary_type_mismatch() {
  let mut module = module_with_entry(1, IrType::Unit);
  let lhs = v_bool(&mut module, true);
  let rhs = v_bool(&mut module, false);
  let result = v_bool(&mut module, true);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_binary(result, BinaryOp::Add, lhs, rhs),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "Binary InstId(0) invalida");
}

#[test]
fn verify_check_binary_types_executes_all_branches() {
  let mut module = module_with_entry(1, IrType::Unit);

  let int_a = v_i32(&mut module, 1);
  let int_b = v_i32(&mut module, 2);
  let bool_a = v_bool(&mut module, true);
  let bool_b = v_bool(&mut module, false);

  let mut emit_binary = |op: BinaryOp, lhs: ValueId, rhs: ValueId, result_const: IrConstant| {
    let result = append_value(&mut module, result_const);
    add_inst_to_block(&mut module, block(0), inst_binary(result, op, lhs, rhs));
  };

  // Add/Sub/Mul/Div (validos)
  emit_binary(BinaryOp::Add, int_a, int_b, IrConstant::Int32(0));
  emit_binary(BinaryOp::Sub, int_a, int_b, IrConstant::Int32(0));
  emit_binary(BinaryOp::Mul, int_a, int_b, IrConstant::Int32(0));
  emit_binary(BinaryOp::Div, int_a, int_b, IrConstant::Int32(0));
  // Add/Sub/Mul/Div (invalidos para cubrir falsos de rhs/result)
  emit_binary(BinaryOp::Add, int_a, bool_a, IrConstant::Int32(0));
  emit_binary(BinaryOp::Sub, int_a, int_b, IrConstant::Bool(false));

  // Gt/Lt/Gte/Lte (validos)
  emit_binary(BinaryOp::Gt, int_a, int_b, IrConstant::Bool(false));
  emit_binary(BinaryOp::Lt, int_a, int_b, IrConstant::Bool(true));
  emit_binary(BinaryOp::Gte, int_a, int_b, IrConstant::Bool(false));
  emit_binary(BinaryOp::Lte, int_a, int_b, IrConstant::Bool(true));
  // Gt/Lt/Gte/Lte (invalidos para cubrir falsos de rhs/result)
  emit_binary(BinaryOp::Gt, int_a, bool_a, IrConstant::Bool(false));
  emit_binary(BinaryOp::Lt, int_a, int_b, IrConstant::Int32(0));

  // Eq/Neq: cubrimos int-int y bool-bool (validos)
  emit_binary(BinaryOp::Eq, int_a, int_b, IrConstant::Bool(false));
  emit_binary(BinaryOp::Neq, bool_a, bool_b, IrConstant::Bool(true));
  // Eq/Neq invalidos: false en rhs del primer branch, rhs del segundo branch y result
  emit_binary(BinaryOp::Eq, int_a, bool_a, IrConstant::Bool(false));
  emit_binary(BinaryOp::Neq, bool_a, int_a, IrConstant::Bool(true));
  emit_binary(BinaryOp::Eq, int_a, int_b, IrConstant::Int32(0));

  // And/Or/Xor (validos)
  emit_binary(BinaryOp::And, bool_a, bool_b, IrConstant::Bool(false));
  emit_binary(BinaryOp::Or, bool_a, bool_b, IrConstant::Bool(true));
  emit_binary(BinaryOp::Xor, bool_a, bool_b, IrConstant::Bool(true));
  // And/Or/Xor (invalidos para cubrir falsos de rhs/result)
  emit_binary(BinaryOp::And, bool_a, int_a, IrConstant::Bool(false));
  emit_binary(BinaryOp::Or, bool_a, bool_b, IrConstant::Int32(0));

  // Caso invalido adicional para ejecutar `!ok` con lhs false inmediato.
  emit_binary(BinaryOp::Xor, int_a, bool_a, IrConstant::Bool(false));

  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_eq!(count_errors(&diagnostics, "Binary InstId("), 10);
}

#[test]
fn verify_reports_binary_type_check_missing_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let lhs = v_i32(&mut module, 1);
  let rhs = v_i32(&mut module, 2);
  add_inst_to_block(
    &mut module,
    block(0),
    InstData {
      result: None,
      kind: InstKind::Binary {
        op: BinaryOp::Add,
        lhs,
        rhs,
      },
    },
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_count_at_least(&diagnostics, "produce valor pero no tiene result", 2);
}

#[test]
fn verify_reports_binary_type_check_invalid_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let lhs = v_i32(&mut module, 1);
  let rhs = v_i32(&mut module, 2);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_binary(ValueId(99), BinaryOp::Add, lhs, rhs),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Binary.result InstId(0)",
  );
}

#[test]
fn verify_reports_binary_type_check_invalid_lhs_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let rhs = v_i32(&mut module, 2);
  let result = v_i32(&mut module, 0);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_binary(result, BinaryOp::Add, ValueId(99), rhs),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Binary.lhs InstId(0)",
  );
}

#[test]
fn verify_reports_binary_type_check_invalid_rhs_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  let lhs = v_i32(&mut module, 1);
  let result = v_i32(&mut module, 0);
  add_inst_to_block(
    &mut module,
    block(0),
    inst_binary(result, BinaryOp::Add, lhs, ValueId(99)),
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Binary.rhs InstId(0)",
  );
}

#[test]
fn verify_reports_phi_input_type_mismatch() {
  let mut module = module_with_entry(2, IrType::Unit);
  let input_value = v_bool(&mut module, true);
  let phi_result = v_i32(&mut module, 0);

  set_terminator(&mut module, block(0), inst_jump(block(1)));
  add_phi_to_block(
    &mut module,
    block(1),
    inst_phi(phi_result, vec![PhiInput::new(block(0), input_value)]),
  );
  set_return_none(&mut module, block(1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "Phi InstId(1) input[0] tiene tipo");
}

#[test]
fn verify_reports_phi_type_check_missing_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  add_phi_to_block(
    &mut module,
    block(0),
    InstData {
      result: None,
      kind: InstKind::Phi { inputs: vec![] },
    },
  );
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_error_count_at_least(&diagnostics, "produce valor pero no tiene result", 2);
}

#[test]
fn verify_reports_phi_type_check_invalid_result_path() {
  let mut module = module_with_entry(1, IrType::Unit);
  add_phi_to_block(&mut module, block(0), inst_phi(ValueId(99), vec![]));
  set_return_none(&mut module, block(0));

  let diagnostics = verify_diagnostics(&module);
  assert_has_error(
    &diagnostics,
    "ValueId ValueId(99) invalido en Phi.result InstId(0)",
  );
}

#[test]
fn verify_reports_return_type_mismatch() {
  let mut module = module_with_entry(1, IrType::Int32);
  let returned = v_bool(&mut module, true);
  set_terminator(&mut module, block(0), inst_return(Some(returned)));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "devuelve Bool, pero el modulo retorna Int32");
}

#[test]
fn verify_reports_return_without_value_in_non_unit() {
  let mut module = module_with_entry(1, IrType::Int32);
  set_return_none(&mut module, block(0));
  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "sin valor en modulo con retorno Int32");
}

#[test]
fn verify_reports_phi_duplicate_predecessor() {
  let mut module = module_with_entry(2, IrType::Unit);
  let input_value = v_i32(&mut module, 1);
  let phi_result = v_i32(&mut module, 0);

  set_terminator(&mut module, block(0), inst_jump(block(1)));
  add_phi_to_block(
    &mut module,
    block(1),
    inst_phi(
      phi_result,
      vec![
        PhiInput::new(block(0), input_value),
        PhiInput::new(block(0), input_value),
      ],
    ),
  );
  set_return_none(&mut module, block(1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "predecessor duplicado");
}

#[test]
fn verify_reports_phi_input_not_real_predecessor() {
  let mut module = module_with_entry(3, IrType::Unit);
  let input_value = v_i32(&mut module, 1);
  let phi_result = v_i32(&mut module, 0);

  set_terminator(&mut module, block(0), inst_jump(block(1)));
  add_phi_to_block(
    &mut module,
    block(1),
    inst_phi(phi_result, vec![PhiInput::new(block(2), input_value)]),
  );
  set_return_none(&mut module, block(1));
  set_return_none(&mut module, block(2));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(&diagnostics, "no es predecessor real del bloque");
}

#[test]
fn verify_reports_phi_input_invalid_value_id() {
  let mut module = module_with_entry(2, IrType::Unit);
  let phi_result = v_i32(&mut module, 0);

  set_terminator(&mut module, block(0), inst_jump(block(1)));
  add_phi_to_block(
    &mut module,
    block(1),
    inst_phi(phi_result, vec![PhiInput::new(block(0), ValueId(99))]),
  );
  set_return_none(&mut module, block(1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "Phi InstId(1) input[0] referencia ValueId invalido ValueId(99)",
  );
}

#[test]
fn verify_reports_phi_does_not_cover_exactly_predecessors() {
  let mut module = module_with_entry(3, IrType::Unit);
  let cond = v_bool(&mut module, true);
  let input_value = v_i32(&mut module, 1);
  let phi_result = v_i32(&mut module, 0);

  set_terminator(&mut module, block(0), inst_branch(cond, block(1), block(2)));
  set_terminator(&mut module, block(2), inst_jump(block(1)));
  add_phi_to_block(
    &mut module,
    block(1),
    inst_phi(phi_result, vec![PhiInput::new(block(0), input_value)]),
  );
  set_return_none(&mut module, block(1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "no cubre exactamente los predecesores del bloque",
  );
}

#[test]
fn verify_reports_cfg_jump_target_missing() {
  let mut module = module_with_entry(1, IrType::Unit);
  set_terminator(&mut module, block(0), inst_jump(block(99)));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "Jump InstId(0) referencia bloque inexistente BlockId(99)",
  );
}

#[test]
fn verify_reports_cfg_branch_if_target_missing() {
  let mut module = module_with_entry(2, IrType::Unit);
  let cond = v_bool(&mut module, true);
  set_terminator(
    &mut module,
    block(0),
    inst_branch(cond, block(99), block(1)),
  );
  set_return_none(&mut module, block(1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "Branch InstId(0) referencia if_block inexistente BlockId(99)",
  );
}

#[test]
fn verify_reports_cfg_branch_else_target_missing() {
  let mut module = module_with_entry(2, IrType::Unit);
  let cond = v_bool(&mut module, true);
  set_terminator(
    &mut module,
    block(0),
    inst_branch(cond, block(1), block(99)),
  );
  set_return_none(&mut module, block(1));

  let diagnostics = verify_diagnostics(&module);
  assert_error_once(
    &diagnostics,
    "Branch InstId(0) referencia else_block inexistente BlockId(99)",
  );
}
