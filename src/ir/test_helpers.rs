use std::collections::BTreeSet;

use crate::{
  analysis::Cfg,
  ast::{Ast, Program},
  diagnostics::Diagnostic,
  ir::{
    BlockData, BlockId, InstData, InstId, InstKind, IrConstant, IrInvariantError, IrModule,
    IrType, LoweringCtx, ValueId,
  },
  parser::parse_program,
  semantic::{PhaseGraph, SemanticAnalyzer, SemanticResult},
};

pub(crate) fn parse_and_analyze(source: &str) -> (Ast, Program, SemanticResult, Vec<Diagnostic>) {
  let (ast, program) = parse_program(source);
  let mut diagnostics = Vec::new();
  let semantic = {
    let mut analyzer =
      SemanticAnalyzer::new(&ast, PhaseGraph::default_semantic_graph(), &mut diagnostics);
    analyzer.analyze(&program)
  };
  (ast, program, semantic, diagnostics)
}

pub(crate) fn lower_source(source: &str) -> (IrModule, Vec<Diagnostic>) {
  let (ast, program, semantic, mut diagnostics) = parse_and_analyze(source);
  let ir = LoweringCtx::lower_to_ir(&program, &ast, &semantic, &mut diagnostics);
  (ir, diagnostics)
}

impl IrModule {
  fn reachable_blocks(&self) -> Vec<BlockId> {
    self.test_cfg().reachable_blocks().collect()
  }

  pub(crate) fn reachable_inst_ids(&self) -> Vec<InstId> {
    let mut seen = BTreeSet::new();
    for block in self.reachable_blocks() {
      // insert phi instructions before the other instructions
      for inst_id in self.block(block).phis() {
        seen.insert(inst_id.0);
      }
      for inst_id in self.block(block).insts() {
        seen.insert(inst_id.0);
      }
      seen.insert(self.block(block).terminator().0);
    }
    seen.into_iter().map(InstId).collect()
  }

  pub(crate) fn count_insts_by_kind(&self, pred: impl Fn(&InstKind) -> bool) -> usize {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter(|inst_id| pred(&self.inst(*inst_id).kind))
      .count()
  }

  pub(crate) fn const_results(&self, constant: IrConstant) -> Vec<ValueId> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match &self.inst(inst_id).kind {
        InstKind::Const(c) if *c == constant => self.inst(inst_id).result,
        _ => None,
      })
      .collect()
  }

  pub(crate) fn print_operands(&self) -> Vec<ValueId> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match &self.inst(inst_id).kind {
        InstKind::Print(value_id) => Some(*value_id),
        _ => None,
      })
      .collect()
  }

  pub(crate) fn phi_results_with_types(&self) -> Vec<(ValueId, IrType)> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match self.inst(inst_id).kind {
        InstKind::Phi { .. } => {
          let result = self
            .inst(inst_id)
            .result
            .expect("todo phi debe tener resultado");
          Some((result, *self.value(result).ty()))
        }
        _ => None,
      })
      .collect()
  }

  pub(crate) fn return_values(&self) -> Vec<Option<ValueId>> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match &self.inst(inst_id).kind {
        InstKind::Return { value } => Some(*value),
        _ => None,
      })
      .collect()
  }

  fn test_cfg(&self) -> Cfg {
    Cfg::build(self, self.entry_block(), &mut vec![])
  }
}

pub(crate) fn base_test_module_with_blocks(block_count: usize) -> IrModule {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  for _ in 0..block_count {
    module.add_block(BlockData::new_block());
  }
  module.set_entry_block(BlockId(0));
  module
}

pub(crate) fn add_bool_const(module: &mut IrModule, block_id: BlockId, value: bool) -> ValueId {
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

pub(crate) fn set_jump_terminator(module: &mut IrModule, block_id: BlockId, target: BlockId) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Jump { target }));
  module.block_mut(block_id).set_terminator(term_id);
}

pub(crate) fn set_branch_terminator(
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

pub(crate) fn set_return_terminator(module: &mut IrModule, block_id: BlockId) {
  let term_id = InstId(module.inst_count());
  module.add_inst(InstData::without_result(InstKind::Return { value: None }));
  module.block_mut(block_id).set_terminator(term_id);
}

pub(crate) fn build_test_cfg(module: &IrModule, entry: BlockId) -> (Cfg, Vec<IrInvariantError>) {
  let mut errors = Vec::new();
  let cfg = Cfg::build(module, entry, &mut errors);
  (cfg, errors)
}
