use crate::{
  Diagnostic,
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

fn has_error(diagnostics: &[Diagnostic], pattern: &str) -> bool {
  diagnostics.iter().any(|diag| diag.msg().contains(pattern))
}

#[test]
fn verify_accepts_module_lowered_by_frontend() {
  let (ir, diagnostics) = lower_source("main { let x = 1; if true { x = 2; } else { x = 3; }; print x; }");
  assert!(
    diagnostics.is_empty(),
    "el source del test debe bajar sin diagnosticos"
  );
  let mut diagnostics = Vec::new();
  ir.verify(&mut diagnostics);
  assert!(diagnostics.is_empty());
}

#[test]
fn verify_reports_missing_entry_block() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "no tiene entry block"));
}

#[test]
fn verify_reports_block_without_terminator() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  module.set_entry_block(BlockId(0));
  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "no tiene terminador"));
}

#[test]
fn verify_reports_non_terminator_as_block_terminator() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  module.set_entry_block(BlockId(0));

  module.add_value(IrConstant::Int32(1).as_value());
  module.add_inst(InstData::with_result(
    ValueId(0),
    InstKind::Const(IrConstant::Int32(1)),
  ));
  module.block_mut(BlockId(0)).set_terminator(InstId(0));

  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "pero no es terminadora"));
}

#[test]
fn verify_reports_invalid_block_reference_in_jump() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  module.set_entry_block(BlockId(0));

  module.add_inst(InstData::without_result(InstKind::Jump {
    target: BlockId(99),
  }));
  module.block_mut(BlockId(0)).set_terminator(InstId(0));

  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "BlockId"));
  assert!(has_error(&diagnostics, "Jump"));
}

#[test]
fn verify_reports_non_boolean_branch_condition() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  module.add_block(BlockData::new_block());
  module.add_block(BlockData::new_block());
  module.set_entry_block(BlockId(0));

  module.add_value(IrConstant::Int32(1).as_value());
  module.add_inst(InstData::with_result(
    ValueId(0),
    InstKind::Const(IrConstant::Int32(1)),
  ));
  module.block_mut(BlockId(0)).add_inst(InstId(0));

  module.add_inst(InstData::without_result(InstKind::Branch {
    condition: ValueId(0),
    if_block: BlockId(1),
    else_block: BlockId(2),
  }));
  module.block_mut(BlockId(0)).set_terminator(InstId(1));

  module.add_inst(InstData::without_result(InstKind::Return { value: None }));
  module.block_mut(BlockId(1)).set_terminator(InstId(2));
  module.add_inst(InstData::without_result(InstKind::Return { value: None }));
  module.block_mut(BlockId(2)).set_terminator(InstId(3));

  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "condicion de tipo"));
}

#[test]
fn verify_reports_return_type_mismatch() {
  let mut module = IrModule::new("m".into(), IrType::Int32);
  module.add_block(BlockData::new_block());
  module.set_entry_block(BlockId(0));
  module.add_inst(InstData::without_result(InstKind::Return { value: None }));
  module.block_mut(BlockId(0)).set_terminator(InstId(0));

  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "sin valor"));
}

#[test]
fn verify_reports_phi_predecessor_mismatch() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block()); // 0 entry
  module.add_block(BlockData::new_block()); // 1 merge
  module.add_block(BlockData::new_block()); // 2 unrelated
  module.set_entry_block(BlockId(0));

  module.add_value(IrConstant::Int32(7).as_value()); // v0
  module.add_inst(InstData::with_result(
    ValueId(0),
    InstKind::Const(IrConstant::Int32(7)),
  )); // i0
  module.block_mut(BlockId(0)).add_inst(InstId(0));

  module.add_inst(InstData::without_result(InstKind::Jump {
    target: BlockId(1),
  })); // i1
  module.block_mut(BlockId(0)).set_terminator(InstId(1));

  module.add_value(IrConstant::Int32(0).as_value()); // placeholder v1 slot used by phi result type
  module.add_inst(InstData::with_result(
    ValueId(1),
    InstKind::Phi {
      inputs: vec![PhiInput::new(BlockId(2), ValueId(0))],
    },
  )); // i2
  module.block_mut(BlockId(1)).add_phi(InstId(2));
  module.add_inst(InstData::without_result(InstKind::Return { value: None })); // i3
  module.block_mut(BlockId(1)).set_terminator(InstId(3));

  module.add_inst(InstData::without_result(InstKind::Return { value: None })); // i4
  module.block_mut(BlockId(2)).set_terminator(InstId(4));

  let mut diagnostics = Vec::new();
  module.verify(&mut diagnostics);
  assert!(has_error(&diagnostics, "no es predecessor real"));
}
