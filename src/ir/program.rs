// Representar el programa completo en IR.
// Por ahora no hay funciones, cuando haya funciones mucha de la logica de aca ira a otro archivo.

use crate::ir::{
  block::BlockData,
  ids::{BlockId, InstId, LocalId, ValueId},
  inst::InstData,
  local::LocalData,
  types::IrType,
  value::ValueData,
};

#[derive(Debug, Clone)]
pub(crate) struct Program {
  name: String,
  entry_block: Option<BlockId>,
  // params: Vec<...> en un futuro
  /// tipo de los valores de retorno del programa
  return_type: IrType,
  /// variables locales del programa
  locals: Vec<LocalData>,
  /// instrucciones del programa
  insts: Vec<InstData>,
  /// bloques del programa
  blocks: Vec<BlockData>,
  /// valores del programa
  values: Vec<ValueData>,
}

impl Program {
  pub(crate) fn new(name: String, return_type: IrType) -> Self {
    Self {
      name,
      entry_block: None,
      return_type,
      locals: Vec::new(),
      insts: Vec::new(),
      blocks: Vec::new(),
      values: Vec::new(),
    }
  }

  pub(crate) fn entry_block(&self) -> BlockId {
    self
      .entry_block
      .expect("debe haber un bloque principal del programa")
  }

  pub(crate) fn set_entry_block(&mut self, main_block: BlockId) {
    self.entry_block = Some(main_block);
  }

  pub(crate) fn local(&self, id: LocalId) -> &LocalData {
    &self.locals[id.0]
  }

  pub(crate) fn add_local(&mut self, data: LocalData) {
    self.locals.push(data);
  }

  pub(crate) fn inst(&self, id: InstId) -> &InstData {
    &self.insts[id.0]
  }

  pub(crate) fn add_inst(&mut self, data: InstData) {
    self.insts.push(data);
  }

  pub(crate) fn block(&self, id: BlockId) -> &BlockData {
    &self.blocks[id.0]
  }

  pub(crate) fn block_mut(&mut self, id: BlockId) -> &mut BlockData {
    &mut self.blocks[id.0]
  }

  pub(crate) fn add_block(&mut self, data: BlockData) {
    self.blocks.push(data);
  }

  pub(crate) fn value(&self, id: ValueId) -> &ValueData {
    &self.values[id.0]
  }

  pub(crate) fn add_value(&mut self, data: ValueData) {
    self.values.push(data);
  }

  pub(crate) fn predecessors(&self, block: BlockId) -> Vec<BlockId> {
    todo!()
  }

  pub(crate) fn successors(&self, block: BlockId) -> Vec<BlockId> {
    todo!()
  }
}
