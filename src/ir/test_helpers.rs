use std::collections::BTreeSet;

#[cfg(test)]
use crate::{
  analysis::cfg::Cfg,
  ir::{
    ids::{BlockId, ValueId},
    types::IrType,
    value::IrConstant,
  },
};
use crate::{
  ast::{Ast, Program},
  diagnostics::Diagnostic,
  ir::{LoweringCtx, ids::InstId, inst::InstKind, module::IrModule},
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
          Some((result, self.value(result).ty().clone()))
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
