use std::collections::VecDeque;

use crate::ir::{ids::BlockId, inst::InstKind, module::IrModule, verify::VerifyError};

/// Control Flow Graph (CFG) para un IrModule
#[derive(Debug, Clone)]
pub(crate) struct Cfg {
  /// Bloque de entrada al modulo
  entry: BlockId,
  preds: Vec<Vec<BlockId>>,
  succs: Vec<Vec<BlockId>>,
  reachable: Vec<bool>,
}

impl Cfg {
  /// Construye un CFG a partir de un modulo
  pub(crate) fn build(module: &IrModule, entry: BlockId, errors: &mut Vec<VerifyError>) -> Self {
    let block_count = module.block_count();
    let mut cfg = Self {
      entry,
      preds: vec![vec![]; block_count],
      succs: vec![vec![]; block_count],
      reachable: vec![false; block_count],
    };

    for i in 0..block_count {
      let block_id = BlockId(i);
      let block = module.block(block_id);
      if !block.has_terminator() {
        continue;
      }

      // El grafo se construye segun cual es la instruccion terminadora del bloque
      let terminator_id = block.terminator();
      let terminator_inst_kind = &module.inst(terminator_id).kind;
      
      match terminator_inst_kind {
        // Un Jump permite saltar del bloque actual al bloque target
        InstKind::Jump { target } => {
          if target.0 < block_count {
            // Se agrega una arista al CFG
            cfg.succs[block_id.0].push(*target);
            cfg.preds[target.0].push(block_id);
          } else {
            errors.push(VerifyError::new(format!(
              "Jump {:?} referencia bloque inexistente {:?}",
              terminator_id, target
            )));
          }
        }
        // Un Branch agrega aristas a los bloques if y else
        InstKind::Branch {
          if_block,
          else_block,
          ..
        } => {
          if if_block.0 < block_count {
            cfg.succs[block_id.0].push(*if_block);
            cfg.preds[if_block.0].push(block_id);
          } else {
            errors.push(VerifyError::new(format!(
              "Branch {:?} referencia if_block inexistente {:?}",
              terminator_id, if_block
            )));
          }
          if else_block.0 < block_count {
            cfg.succs[block_id.0].push(*else_block);
            cfg.preds[else_block.0].push(block_id);
          } else {
            errors.push(VerifyError::new(format!(
              "Branch {:?} referencia else_block inexistente {:?}",
              terminator_id, else_block
            )));
          }
        }
        InstKind::Return { .. } => {}
        _ => {}
      }
    }

    cfg.compute_reachability();
    cfg
  }

  /// Bloques predecesores en el CFG para cada bloque
  pub(crate) fn predecessors(&self, block: BlockId) -> &[BlockId] {
    &self.preds[block.0]
  }

  /// Bloques sucesores en el CFG para cada bloque
  pub(crate) fn successors(&self, block: BlockId) -> &[BlockId] {
    &self.succs[block.0]
  }

  /// Devuelve un iterador sobre los bloques alcanzables en el CFG
  #[cfg(test)]
  pub(crate) fn reachable_blocks(&self) -> impl Iterator<Item = BlockId> + '_ {
    (0..self.reachable.len())
      .filter(|&index| self.reachable[index])
      .map(BlockId)
  }

  /// Funcion auxiliar para computar `reachable_blocks`
  fn compute_reachability(&mut self) {
    if self.entry.0 >= self.reachable.len() {
      return;
    }

    let mut queue = VecDeque::new();
    queue.push_back(self.entry);

    while let Some(block) = queue.pop_front() {
      if self.reachable[block.0] {
        continue;
      }
      self.reachable[block.0] = true;
      for &succ in self.successors(block) {
        if !self.reachable[succ.0] {
          queue.push_back(succ);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ir::{
    block::BlockData,
    ids::{BlockId, InstId, ValueId},
    inst::{InstData, InstKind},
    types::IrType,
    value::IrConstant,
  };

  use super::*;

  fn has_error(errors: &[VerifyError], pattern: &str) -> bool {
    errors.iter().any(|err| match err {
      VerifyError::InvariantViolation(msg) => msg.contains(pattern),
    })
  }

  fn base_module_with_blocks(block_count: usize) -> IrModule {
    let mut module = IrModule::new("m".into(), IrType::Unit);
    for _ in 0..block_count {
      module.add_block(BlockData::new_block());
    }
    module.set_entry_block(BlockId(0));
    module
  }

  fn emit_return(module: &mut IrModule, block_id: BlockId) {
    let term_id = InstId(module.inst_count());
    module.add_inst(InstData::without_result(InstKind::Return { value: None }));
    module.block_mut(block_id).set_terminator(term_id);
  }

  #[test]
  fn cfg_builds_jump_edge_and_predecessor() {
    let mut module = base_module_with_blocks(2);

    module.add_inst(InstData::without_result(InstKind::Jump {
      target: BlockId(1),
    }));
    module.block_mut(BlockId(0)).set_terminator(InstId(0));
    emit_return(&mut module, BlockId(1));

    let mut errors = Vec::new();
    let cfg = Cfg::build(&module, BlockId(0), &mut errors);

    assert!(errors.is_empty());
    assert_eq!(cfg.successors(BlockId(0)), &[BlockId(1)]);
    assert_eq!(cfg.predecessors(BlockId(1)), &[BlockId(0)]);
  }

  #[test]
  fn cfg_builds_branch_edges() {
    let mut module = base_module_with_blocks(3);

    module.add_value(IrConstant::Bool(true).as_value());
    module.add_inst(InstData::with_result(
      ValueId(0),
      InstKind::Const(IrConstant::Bool(true)),
    ));
    module.block_mut(BlockId(0)).add_inst(InstId(0));

    module.add_inst(InstData::without_result(InstKind::Branch {
      condition: ValueId(0),
      if_block: BlockId(1),
      else_block: BlockId(2),
    }));
    module.block_mut(BlockId(0)).set_terminator(InstId(1));
    emit_return(&mut module, BlockId(1));
    emit_return(&mut module, BlockId(2));

    let mut errors = Vec::new();
    let cfg = Cfg::build(&module, BlockId(0), &mut errors);

    assert!(errors.is_empty());
    assert_eq!(cfg.successors(BlockId(0)), &[BlockId(1), BlockId(2)]);
    assert_eq!(cfg.predecessors(BlockId(1)), &[BlockId(0)]);
    assert_eq!(cfg.predecessors(BlockId(2)), &[BlockId(0)]);
  }

  #[test]
  fn cfg_reports_invalid_jump_target() {
    let mut module = base_module_with_blocks(1);
    module.add_inst(InstData::without_result(InstKind::Jump {
      target: BlockId(99),
    }));
    module.block_mut(BlockId(0)).set_terminator(InstId(0));

    let mut errors = Vec::new();
    let _cfg = Cfg::build(&module, BlockId(0), &mut errors);

    assert!(has_error(&errors, "Jump"));
    assert!(has_error(&errors, "inexistente"));
  }

  #[test]
  fn cfg_reports_invalid_branch_targets() {
    let mut module = base_module_with_blocks(1);
    module.add_value(IrConstant::Bool(true).as_value());
    module.add_inst(InstData::with_result(
      ValueId(0),
      InstKind::Const(IrConstant::Bool(true)),
    ));
    module.block_mut(BlockId(0)).add_inst(InstId(0));
    module.add_inst(InstData::without_result(InstKind::Branch {
      condition: ValueId(0),
      if_block: BlockId(1),
      else_block: BlockId(2),
    }));
    module.block_mut(BlockId(0)).set_terminator(InstId(1));

    let mut errors = Vec::new();
    let _cfg = Cfg::build(&module, BlockId(0), &mut errors);

    assert!(has_error(&errors, "if_block inexistente"));
    assert!(has_error(&errors, "else_block inexistente"));
  }

  #[test]
  fn cfg_reachability_marks_only_blocks_reachable_from_entry() {
    let mut module = base_module_with_blocks(3);
    module.add_inst(InstData::without_result(InstKind::Jump {
      target: BlockId(1),
    }));
    module.block_mut(BlockId(0)).set_terminator(InstId(0));
    emit_return(&mut module, BlockId(1));
    emit_return(&mut module, BlockId(2));

    let mut errors = Vec::new();
    let cfg = Cfg::build(&module, BlockId(0), &mut errors);
    let reachable: Vec<BlockId> = cfg.reachable_blocks().collect();

    assert!(errors.is_empty());
    assert_eq!(reachable, vec![BlockId(0), BlockId(1)]);
  }
}
