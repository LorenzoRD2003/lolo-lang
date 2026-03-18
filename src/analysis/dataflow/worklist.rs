//! Cola de trabajo minima para solvers de dataflow.

use std::collections::VecDeque;

use crate::ir::BlockId;

/// Worklist FIFO con deduplicacion por bloque.
#[derive(Debug, Clone)]
pub(crate) struct Worklist {
  queue: VecDeque<BlockId>,
  queued: Vec<bool>,
}

impl Worklist {
  /// Crea una worklist vacia preparada para `block_count` bloques.
  pub(crate) fn new(block_count: usize) -> Self {
    Self {
      queue: VecDeque::new(),
      queued: vec![false; block_count],
    }
  }

  /// Encola `block` si no estaba ya pendiente.
  pub(crate) fn push(&mut self, block: BlockId) {
    if self.queued[block.0] {
      return;
    }
    self.queue.push_back(block);
    self.queued[block.0] = true;
  }

  /// Saca el proximo bloque pendiente.
  pub(crate) fn pop(&mut self) -> Option<BlockId> {
    let block = self.queue.pop_front()?;
    self.queued[block.0] = false;
    Some(block)
  }

  /// Indica si no quedan bloques pendientes.
  pub(crate) fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  /// Encola una coleccion de bloques.
  pub(crate) fn extend(&mut self, blocks: impl IntoIterator<Item = BlockId>) {
    for block in blocks {
      self.push(block);
    }
  }
}
