use super::{UcePass, plan::UcePlan};
use crate::{
  ir::test_helpers::build_test_cfg,
  ir::{
    BlockData, BlockId, InstData, InstId, InstKind, IrConstant, IrModule, IrType, PhiInput, ValueId,
  },
  passes::IrPass,
};

fn module_with_blocks(block_count: usize, return_ty: IrType) -> IrModule {
  let mut module = IrModule::new("m".into(), return_ty);
  for _ in 0..block_count {
    module.add_block(BlockData::new_block());
  }
  module.set_entry_block(BlockId(0));
  module
}

fn add_i32_const(module: &mut IrModule, block: BlockId, value: i32) -> ValueId {
  let value_id = ValueId(module.value_count());
  module.add_value(IrConstant::Int32(value).as_value());
  let inst_id = InstId(module.inst_count());
  module.add_inst(InstData::with_result(
    value_id,
    InstKind::Const(IrConstant::Int32(value)),
  ));
  module.block_mut(block).add_inst(inst_id);
  value_id
}

fn add_phi_i32(module: &mut IrModule, block: BlockId, inputs: Vec<PhiInput>) -> ValueId {
  let value_id = ValueId(module.value_count());
  module.add_value(IrConstant::Int32(0).as_value());
  let phi_id = InstId(module.inst_count());
  module.add_inst(InstData::with_result(value_id, InstKind::Phi { inputs }));
  module.block_mut(block).add_phi(phi_id);
  value_id
}

fn set_jump(module: &mut IrModule, block: BlockId, target: BlockId) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Jump { target }));
  module.block_mut(block).set_terminator(term_id);
}

fn set_branch(
  module: &mut IrModule,
  block: BlockId,
  cond: ValueId,
  if_b: BlockId,
  else_b: BlockId,
) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Branch {
    condition: cond,
    if_block: if_b,
    else_block: else_b,
  }));
  module.block_mut(block).set_terminator(term_id);
}

fn set_return(module: &mut IrModule, block: BlockId, value: Option<ValueId>) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Return { value }));
  module.block_mut(block).set_terminator(term_id);
}

#[test]
fn build_plan_maps_only_reachable_blocks() {
  // 0 -> 1, y 2 inalcanzable
  let mut m = module_with_blocks(3, IrType::Unit);
  set_jump(&mut m, BlockId(0), BlockId(1));
  set_return(&mut m, BlockId(1), None);
  set_return(&mut m, BlockId(2), None);

  let (cfg, errors) = build_test_cfg(&m, BlockId(0));
  assert!(errors.is_empty());

  let plan = UcePass::build_plan(&m, &cfg);
  assert_eq!(plan.kept_old_blocks(), &[BlockId(0), BlockId(1)]);
  assert_eq!(plan.remap_block(BlockId(0)), Some(BlockId(0)));
  assert_eq!(plan.remap_block(BlockId(1)), Some(BlockId(1)));
  assert_eq!(plan.remap_block(BlockId(2)), None);
}

#[test]
fn rebuild_block_storage_remaps_entry_and_drops_unreachable() {
  // entry viejo = 1
  let mut m = module_with_blocks(4, IrType::Unit);
  m.set_entry_block(BlockId(1));
  set_jump(&mut m, BlockId(1), BlockId(2));
  set_return(&mut m, BlockId(2), None);
  set_return(&mut m, BlockId(0), None);
  set_return(&mut m, BlockId(3), None);

  let (cfg, errors) = build_test_cfg(&m, BlockId(1));
  assert!(errors.is_empty());
  let plan = UcePass::build_plan(&m, &cfg);

  UcePass::rebuild_block_storage(&mut m, &plan);

  assert_eq!(m.block_count(), 2);
  assert_eq!(m.entry_block(), BlockId(0));
}

#[test]
fn rewrite_terminator_targets_updates_jump_and_branch() {
  let plan = UcePlan::new(
    vec![Some(BlockId(0)), None, Some(BlockId(1)), Some(BlockId(2))],
    vec![BlockId(0), BlockId(2), BlockId(3)],
  );

  let mut jump = InstKind::Jump { target: BlockId(3) };
  plan.rewrite_terminator_targets(&mut jump);
  assert!(matches!(jump, InstKind::Jump { target } if target == BlockId(2)));

  let mut branch = InstKind::Branch {
    condition: ValueId(0),
    if_block: BlockId(2),
    else_block: BlockId(3),
  };
  plan.rewrite_terminator_targets(&mut branch);
  assert!(matches!(
    branch,
    InstKind::Branch {
      if_block: BlockId(1),
      else_block: BlockId(2),
      ..
    }
  ));
}

#[test]
fn rewrite_phi_inputs_remaps_and_filters_removed_predecessors() {
  let plan = UcePlan::new(
    vec![Some(BlockId(0)), None, Some(BlockId(1)), Some(BlockId(2))],
    vec![BlockId(0), BlockId(2), BlockId(3)],
  );

  let mut inputs = vec![
    PhiInput::new(BlockId(1), ValueId(10)), // se elimina (pred inalcanzable)
    PhiInput::new(BlockId(2), ValueId(20)), // remapea -> bb1
    PhiInput::new(BlockId(3), ValueId(30)), // remapea -> bb2
  ];
  plan.rewrite_phi_inputs(&mut inputs);

  assert_eq!(inputs.len(), 2);
  assert_eq!(inputs[0].pred_block(), BlockId(1));
  assert_eq!(inputs[0].value(), ValueId(20));
  assert_eq!(inputs[1].pred_block(), BlockId(2));
  assert_eq!(inputs[1].value(), ValueId(30));
}

#[test]
fn rebuild_module_rewrites_refs_and_reports_stats() {
  // old graph:
  // 0(entry) -> 2
  // 1(unreachable) -> 3
  // 2 -> 3
  // 3: phi([2 -> v2], [1 -> v1]) return phi
  //
  // remap esperado: 0->0, 2->1, 3->2 ; 1 eliminado
  let mut m = module_with_blocks(4, IrType::Int32);

  let v1 = add_i32_const(&mut m, BlockId(1), 11);
  let v2 = add_i32_const(&mut m, BlockId(2), 22);
  let phi = add_phi_i32(
    &mut m,
    BlockId(3),
    vec![PhiInput::new(BlockId(2), v2), PhiInput::new(BlockId(1), v1)],
  );

  set_jump(&mut m, BlockId(0), BlockId(2));
  set_jump(&mut m, BlockId(1), BlockId(3));
  set_jump(&mut m, BlockId(2), BlockId(3));
  set_return(&mut m, BlockId(3), Some(phi));

  let (cfg, errors) = build_test_cfg(&m, BlockId(0));
  assert!(errors.is_empty());

  let stats = UcePass::run(&mut m, &cfg);

  assert_eq!(m.block_count(), 3);
  assert_eq!(m.entry_block(), BlockId(0));
  assert_eq!(stats.removed_blocks, 1);
  assert_eq!(stats.rewritten_jumps, 2);
  assert_eq!(stats.rewritten_branches, 0);
  assert_eq!(stats.removed_phi_inputs, 1);

  let b0_term = m.block(BlockId(0)).terminator();
  assert!(matches!(
    m.inst(b0_term).kind,
    InstKind::Jump { target } if target == BlockId(1)
  ));

  let b1_term = m.block(BlockId(1)).terminator();
  assert!(matches!(
    m.inst(b1_term).kind,
    InstKind::Jump { target } if target == BlockId(2)
  ));

  let phi_id = m.block(BlockId(2)).phis()[0];
  let InstKind::Phi { inputs } = &m.inst(phi_id).kind else {
    panic!("se esperaba phi en el bloque merge");
  };
  assert_eq!(inputs.len(), 1);
  assert_eq!(inputs[0].pred_block(), BlockId(1));
  assert_eq!(inputs[0].value(), v2);
}

#[test]
fn run_noop_when_all_blocks_reachable() {
  // 0 -> 1 -> 2, todos alcanzables
  let mut m = module_with_blocks(3, IrType::Unit);
  set_jump(&mut m, BlockId(0), BlockId(1));
  set_jump(&mut m, BlockId(1), BlockId(2));
  set_return(&mut m, BlockId(2), None);

  let (cfg, errors) = build_test_cfg(&m, BlockId(0));
  assert!(errors.is_empty());
  let stats = UcePass::run(&mut m, &cfg);

  assert_eq!(m.block_count(), 3);
  assert_eq!(stats.removed_blocks, 0);
  assert_eq!(stats.rewritten_jumps, 0);
  assert_eq!(stats.rewritten_branches, 0);
  assert_eq!(stats.removed_phi_inputs, 0);
}

#[test]
fn branch_stats_count_rewrite_when_target_ids_change() {
  // old: 0 -> branch(2,3), 1 unreachable, 2/3 returns
  // remap: 0->0, 2->1, 3->2 => branch debe reescribirse 1 vez.
  let mut m = module_with_blocks(4, IrType::Unit);
  let cond = add_i32_const(&mut m, BlockId(0), 1);
  set_branch(&mut m, BlockId(0), cond, BlockId(2), BlockId(3));
  set_return(&mut m, BlockId(1), None);
  set_return(&mut m, BlockId(2), None);
  set_return(&mut m, BlockId(3), None);

  let (cfg, errors) = build_test_cfg(&m, BlockId(0));
  assert!(errors.is_empty());
  let stats = UcePass::run(&mut m, &cfg);

  assert_eq!(stats.removed_blocks, 1);
  assert_eq!(stats.rewritten_branches, 1);
}

#[test]
fn rewrite_block_references_handles_block_without_terminator() {
  // Caso deliberadamente "mal formado": bloque sin terminador.
  // Sirve para cubrir la rama term_id=None en rewrite_block_references.
  let mut m = module_with_blocks(1, IrType::Unit);
  let plan = UcePlan::new(vec![Some(BlockId(0))], vec![BlockId(0)]);

  let (rewritten_jumps, rewritten_branches, removed_phi_inputs) =
    UcePass::rewrite_block_references(&mut m, &plan);

  assert_eq!(rewritten_jumps, 0);
  assert_eq!(rewritten_branches, 0);
  assert_eq!(removed_phi_inputs, 0);
}

#[test]
fn rewrite_phi_refs_in_block_ignores_non_phi_ids() {
  // Rama false del `if let InstKind::Phi { .. }`.
  let mut m = module_with_blocks(1, IrType::Unit);
  let value_id = add_i32_const(&mut m, BlockId(0), 7);
  let non_phi_id = m.block(BlockId(0)).insts()[0];
  let _ = value_id;

  let plan = UcePlan::new(vec![Some(BlockId(0))], vec![BlockId(0)]);
  let removed = UcePass::rewrite_phi_refs_in_block(&mut m, &plan, &[non_phi_id]);

  assert_eq!(removed, 0);
  assert!(matches!(
    m.inst(non_phi_id).kind,
    InstKind::Const(IrConstant::Int32(7))
  ));
}

#[test]
fn rewrite_terminator_ref_branch_no_change_does_not_increment_stats() {
  // Cubre el guard false: if/else no cambian tras remap identidad.
  let mut m = module_with_blocks(3, IrType::Unit);
  let cond = add_i32_const(&mut m, BlockId(0), 1);
  set_branch(&mut m, BlockId(0), cond, BlockId(1), BlockId(2));
  set_return(&mut m, BlockId(1), None);
  set_return(&mut m, BlockId(2), None);
  let term_id = m.block(BlockId(0)).terminator();

  let plan = UcePlan::new(
    vec![Some(BlockId(0)), Some(BlockId(1)), Some(BlockId(2))],
    vec![BlockId(0), BlockId(1), BlockId(2)],
  );
  let stats = UcePass::rewrite_terminator_ref(&mut m, &plan, term_id);

  assert_eq!(stats.rewritten_branches, 0);
  assert_eq!(stats.rewritten_jumps, 0);
}

#[test]
fn rewrite_terminator_ref_branch_counts_change_when_only_else_target_changes() {
  // Fuerza evaluar el segundo lado del OR del guard:
  // if_block igual, else_block distinto.
  let mut m = module_with_blocks(4, IrType::Unit);
  let cond = add_i32_const(&mut m, BlockId(0), 1);
  set_branch(&mut m, BlockId(0), cond, BlockId(1), BlockId(3));
  set_return(&mut m, BlockId(1), None);
  set_return(&mut m, BlockId(2), None);
  set_return(&mut m, BlockId(3), None);
  let term_id = m.block(BlockId(0)).terminator();

  // old->new: 0->0, 1->1, 2 eliminado, 3->2
  let plan = UcePlan::new(
    vec![Some(BlockId(0)), Some(BlockId(1)), None, Some(BlockId(2))],
    vec![BlockId(0), BlockId(1), BlockId(3)],
  );
  let stats = UcePass::rewrite_terminator_ref(&mut m, &plan, term_id);

  assert_eq!(stats.rewritten_branches, 1);
  assert!(matches!(
    m.inst(term_id).kind,
    InstKind::Branch {
      if_block: BlockId(1),
      else_block: BlockId(2),
      ..
    }
  ));
}

#[test]
fn uce_pass_name_is_stable() {
  assert_eq!(UcePass.name(), "uce");
}
