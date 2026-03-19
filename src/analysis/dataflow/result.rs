//! Guarda el resultado del solver de forma estandar.
//! Expone consultas comodas para los analisis y futuros passes.
//! Por ejemplo, hechos `in` y `out` por bloque.

use crate::ir::BlockId;

#[derive(Debug, Clone)]
pub(crate) struct DataflowResult<Fact: Clone + PartialEq> {
  in_facts: Vec<Fact>,
  out_facts: Vec<Fact>,
}

impl<Fact> DataflowResult<Fact>
where
  Fact: Clone + PartialEq,
{
  pub(crate) fn new(block_count: usize, initial_fact: Fact) -> Self {
    Self {
      in_facts: vec![initial_fact.clone(); block_count],
      out_facts: vec![initial_fact; block_count],
    }
  }

  pub(crate) fn set_in_fact(&mut self, block: BlockId, fact: Fact) {
    self.in_facts[block.0] = fact;
  }

  pub(crate) fn set_out_fact(&mut self, block: BlockId, fact: Fact) {
    self.out_facts[block.0] = fact;
  }

  pub(crate) fn in_fact(&self, block: BlockId) -> &Fact {
    &self.in_facts[block.0]
  }

  pub(crate) fn out_fact(&self, block: BlockId) -> &Fact {
    &self.out_facts[block.0]
  }

  pub(crate) fn block_count(&self) -> usize {
    self.in_facts.len()
  }

  pub(crate) fn facts_for(&self, block: BlockId) -> (&Fact, &Fact) {
    (&self.in_facts[block.0], &self.out_facts[block.0])
  }

  /// Consume el `DataflowResult`, y devuelve ambos vectores.
  pub(crate) fn into_parts(self) -> (Vec<Fact>, Vec<Fact>) {
    (self.in_facts, self.out_facts)
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = (BlockId, &Fact, &Fact)> {
    self
      .in_facts
      .iter()
      .zip(self.out_facts.iter())
      .enumerate()
      .map(|(i, (in_f, out_f))| (BlockId(i), in_f, out_f))
  }
}
