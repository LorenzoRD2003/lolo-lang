// Analisis de dominadores sobre CFG.
//
// Este modulo define la API publica interna (`pub(crate)`) para:
// - `idom` (immediate dominators),
// - arbol de dominadores,
// - frontera de dominancia.

mod frontier;
mod idom;
mod tree;

use crate::analysis::{
  cfg::Cfg,
  dominators::{frontier::DominanceFrontier, idom::compute_idom, tree::DominatorTree},
};

/// Resultado completo del analisis de dominadores.
#[derive(Debug, Clone)]
pub(crate) struct Dominators {
  tree: DominatorTree,
  frontier: DominanceFrontier,
}

impl Dominators {
  /// Construye dominadores + frontera de dominancia para un CFG.
  pub(crate) fn compute(cfg: &Cfg) -> Self {
    let idom = compute_idom(cfg);
    let tree = DominatorTree::from_idom(cfg.block_count(), cfg.entry(), idom);
    let frontier = DominanceFrontier::compute(cfg, &tree);
    Self { tree, frontier }
  }

  /// Acceso al arbol de dominadores.
  pub(crate) fn tree(&self) -> &DominatorTree {
    &self.tree
  }

  /// Acceso a la frontera de dominancia.
  pub(crate) fn frontier(&self) -> &DominanceFrontier {
    &self.frontier
  }
}

#[cfg(test)]
mod tests;
