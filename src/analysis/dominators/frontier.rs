use crate::{analysis::cfg::Cfg, ir::BlockId};

use super::DominatorTree;

/// Frontera de dominancia por bloque.
#[derive(Debug, Clone)]
pub(crate) struct DominanceFrontier {
  frontier: Vec<Vec<BlockId>>,
}

impl DominanceFrontier {
  /// Calcula la frontera de dominancia a partir de CFG + arbol de dominadores.
  ///
  /// Estado actual: skeleton (fronteras vacias).
  pub(crate) fn compute(cfg: &Cfg, _tree: &DominatorTree) -> Self {
    // TODO: implementar calculo real de frontera de dominancia.
    Self {
      frontier: vec![Vec::new(); cfg.block_count()],
    }
  }

  pub(crate) fn for_block(&self, block: BlockId) -> &[BlockId] {
    &self.frontier[block.0]
  }
}
