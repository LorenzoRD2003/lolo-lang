use super::*;
use crate::{
  diagnostics::Diagnosable,
  ir::{BlockData, BlockId, InstData, InstId, InstKind, IrConstant, IrInvariantError, IrType, ValueId},
};

fn block(id: usize) -> BlockId {
  BlockId(id)
}

fn base_module_with_blocks(block_count: usize) -> IrModule {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  for _ in 0..block_count {
    module.add_block(BlockData::new_block());
  }
  module.set_entry_block(block(0));
  module
}

fn add_bool_const(module: &mut IrModule, block_id: BlockId, value: bool) -> ValueId {
  let value_id = ValueId(module.value_count());
  module.add_value(IrConstant::Bool(value).as_value());
  let inst_id = InstId(module.inst_count());
  module.add_inst(InstData::with_result(
    value_id,
    InstKind::Const(IrConstant::Bool(value)),
  ));
  module.block_mut(block_id).add_inst(inst_id);
  value_id
}

fn set_jump_terminator(module: &mut IrModule, block_id: BlockId, target: BlockId) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Jump { target }));
  module.block_mut(block_id).set_terminator(term_id);
}

fn set_branch_terminator(
  module: &mut IrModule,
  block_id: BlockId,
  condition: ValueId,
  if_block: BlockId,
  else_block: BlockId,
) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Branch {
    condition,
    if_block,
    else_block,
  }));
  module.block_mut(block_id).set_terminator(term_id);
}

fn set_return_terminator(module: &mut IrModule, block_id: BlockId) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Return { value: None }));
  module.block_mut(block_id).set_terminator(term_id);
}

fn build_cfg(module: &IrModule) -> (Cfg, Vec<IrInvariantError>) {
  let mut errors = Vec::new();
  let cfg = Cfg::build(module, block(0), &mut errors);
  (cfg, errors)
}

fn has_error(errors: &[IrInvariantError], pattern: &str) -> bool {
  errors
    .iter()
    .any(|err| err.to_diagnostic().msg().contains(pattern))
}

#[test]
fn cfg_builds_jump_edge_and_predecessor() {
  // 0 -> 1
  let mut module = base_module_with_blocks(2);
  set_jump_terminator(&mut module, block(0), block(1));
  set_return_terminator(&mut module, block(1));

  let (cfg, errors) = build_cfg(&module);

  assert!(errors.is_empty());
  assert_eq!(cfg.successors(block(0)), &[block(1)]);
  assert_eq!(cfg.predecessors(block(1)), &[block(0)]);
}

#[test]
fn cfg_builds_branch_edges() {
  // 0 -> 1; 0 -> 2;
  let mut module = base_module_with_blocks(3);
  let cond = add_bool_const(&mut module, block(0), true);
  set_branch_terminator(&mut module, block(0), cond, block(1), block(2));
  set_return_terminator(&mut module, block(1));
  set_return_terminator(&mut module, block(2));

  let (cfg, errors) = build_cfg(&module);

  assert!(errors.is_empty());
  assert_eq!(cfg.successors(block(0)), &[block(1), block(2)]);
  assert_eq!(cfg.predecessors(block(1)), &[block(0)]);
  assert_eq!(cfg.predecessors(block(2)), &[block(0)]);
}

#[test]
fn cfg_reports_invalid_jump_target() {
  // 0 -/-> 99 (invalido)
  let mut module = base_module_with_blocks(1);
  set_jump_terminator(&mut module, block(0), block(99));

  let (_cfg, errors) = build_cfg(&module);

  assert!(has_error(&errors, "Jump"));
  assert!(has_error(&errors, "inexistente"));
}

#[test]
fn cfg_reports_invalid_branch_targets() {
  // 0 -/-> 1, 2 (invalidos)
  let mut module = base_module_with_blocks(1);
  let cond = add_bool_const(&mut module, block(0), true);
  set_branch_terminator(&mut module, block(0), cond, block(1), block(2));

  let (_cfg, errors) = build_cfg(&module);

  assert!(has_error(&errors, "if_block inexistente"));
  assert!(has_error(&errors, "else_block inexistente"));
}

#[test]
fn cfg_reachability_with_merge() {
  // 0 -> 1; 0 -> 2; 1 -> 3; 2 -> 3
  let mut module = base_module_with_blocks(4);
  let cond = add_bool_const(&mut module, block(0), true);
  set_branch_terminator(&mut module, block(0), cond, block(1), block(2));
  set_jump_terminator(&mut module, block(1), block(3));
  set_jump_terminator(&mut module, block(2), block(3));
  set_return_terminator(&mut module, block(3));

  let (cfg, errors) = build_cfg(&module);
  let reachable: Vec<BlockId> = cfg.reachable_blocks().collect();

  assert!(errors.is_empty());
  assert_eq!(reachable, vec![block(0), block(1), block(2), block(3)]);
  assert_eq!(cfg.predecessors(block(3)), &[block(1), block(2)]);
}
