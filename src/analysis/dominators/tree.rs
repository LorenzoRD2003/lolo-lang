use crate::ir::BlockId;

/// Arbol de dominadores construido desde `idom`.
#[derive(Debug, Clone)]
pub(crate) struct DominatorTree {
  entry: BlockId,
  idom: Vec<Option<BlockId>>,
  children: Vec<Vec<BlockId>>,
}

impl DominatorTree {
  pub(crate) fn from_idom(
    block_count: usize,
    entry: BlockId,
    idom: Vec<Option<BlockId>>,
  ) -> Self {
    todo!()
  }

  pub(crate) fn entry(&self) -> BlockId {
    self.entry
  }

  /// Dominador inmediato de un bloque.
  pub(crate) fn idom(&self, block: BlockId) -> Option<BlockId> {
    self.idom[block.0]
  }

  /// Hijos directos en el arbol de dominadores.
  pub(crate) fn children(&self, block: BlockId) -> &[BlockId] {
    &self.children[block.0]
  }

  /// `true` si `a` domina a `b`.
  pub(crate) fn dominates(&self, a: BlockId, b: BlockId) -> bool {
    todo!()
  }

  pub(crate) fn strictly_dominates(&self, a: BlockId, b: BlockId) -> bool {
    todo!()
  }
}
