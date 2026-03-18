//! Guarda el resultado del solver de forma estandar.
//! Expone consultas comodas para los analisis y futuros passes.
//! Por ejemplo, hechos `in` y `out` por bloque.

use rustc_hash::FxHashMap;

use crate::ir::BlockId;

#[derive(Debug, Clone)]
pub(crate) struct DataflowResult<Fact: Clone + PartialEq> {
  block_map_to_in_fact: FxHashMap<BlockId, Fact>,
  block_map_to_out_fact: FxHashMap<BlockId, Fact>,
}

impl<Fact> DataflowResult<Fact>
where
  Fact: Clone + PartialEq,
{
  pub(crate) fn new() -> Self {
    Self {
      block_map_to_in_fact: FxHashMap::default(),
      block_map_to_out_fact: FxHashMap::default(),
    }
  }

  pub(crate) fn set_in_fact(&mut self, block: BlockId, fact: Fact) {
    self.block_map_to_in_fact.insert(block, fact);
  }

  pub(crate) fn set_out_fact(&mut self, block: BlockId, fact: Fact) {
    self.block_map_to_out_fact.insert(block, fact);
  }

  pub(crate) fn in_fact(&self, block: BlockId) -> &Fact {
    self
      .block_map_to_in_fact
      .get(&block)
      .expect("el hecho debe existir")
  }

  pub(crate) fn out_fact(&self, block: BlockId) -> &Fact {
    self
      .block_map_to_out_fact
      .get(&block)
      .expect("el hecho debe existir")
  }

  pub(crate) fn block_count(&self) -> usize {
    self.block_map_to_in_fact.len()
  }

  pub(crate) fn facts_for(&self, block: BlockId) -> (&Fact, &Fact) {
    (self.in_fact(block), self.out_fact(block))
  }

  /// Consume el `DataflowResult`, y devuelve ambos hashmaps.
  pub(crate) fn into_parts(self) -> (FxHashMap<BlockId, Fact>, FxHashMap<BlockId, Fact>) {
    (self.block_map_to_in_fact, self.block_map_to_out_fact)
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = (&BlockId, &Fact, &Fact)> {
    self
      .block_map_to_in_fact
      .iter()
      .filter_map(|(k, v)| self.block_map_to_out_fact.get(k).map(|w| (k, v, w)))
  }
}
