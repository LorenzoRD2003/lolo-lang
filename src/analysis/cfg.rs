mod error;
pub(crate) use error::CfgError;

use std::collections::VecDeque;

use crate::ir::{BlockId, InstKind, IrModule};

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
  pub(crate) fn build(module: &IrModule, entry: BlockId, errors: &mut Vec<CfgError>) -> Self {
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
            errors.push(CfgError::JumpTargetMissing {
              terminator_id,
              target: *target,
            });
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
            errors.push(CfgError::BranchIfTargetMissing {
              terminator_id,
              if_block: *if_block,
            });
          }
          if else_block.0 < block_count {
            cfg.succs[block_id.0].push(*else_block);
            cfg.preds[else_block.0].push(block_id);
          } else {
            errors.push(CfgError::BranchElseTargetMissing {
              terminator_id,
              else_block: *else_block,
            });
          }
        }
        InstKind::Return { .. } => {}
        _ => {}
      }
    }

    cfg.compute_reachability();
    cfg
  }

  /// Bloque de entrada del CFG.
  pub(crate) fn entry(&self) -> BlockId {
    self.entry
  }

  /// Cantidad total de bloques del CFG.
  pub(crate) fn block_count(&self) -> usize {
    self.succs.len()
  }

  /// `true` si el bloque es alcanzable desde `entry`.
  pub(crate) fn is_reachable(&self, block: BlockId) -> bool {
    self.reachable[block.0]
  }

  /// Iterador sobre todos los bloques del CFG (alcanzables o no).
  pub(crate) fn blocks(&self) -> impl Iterator<Item = BlockId> + '_ {
    (0..self.block_count()).map(BlockId)
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
  pub(crate) fn reachable_blocks(&self) -> impl Iterator<Item = BlockId> + '_ {
    (0..self.reachable.len())
      .filter(|&index| self.reachable[index])
      .map(BlockId)
  }

  /// Funcion auxiliar para computar `reachable_blocks`
  fn compute_reachability(&mut self) {
    let mut queue = VecDeque::new();
    queue.push_back(self.entry);

    while let Some(block) = queue.pop_front() {
      if self.reachable[block.0] {
        continue;
      }
      self.reachable[block.0] = true;
      for &succ in self.successors(block) {
        // Esto es unreachable hasta que haya bucles
        if !self.reachable[succ.0] {
          queue.push_back(succ);
        }
      }
    }
  }

  // fn reverse_post_order() -> impl Iterator<Item = BlockId> {
  //   todo!()
  // }

  // fn post_order() -> impl Iterator<Item = BlockId> {
  //   todo!()
  // }
}

#[cfg(test)]
mod tests;
