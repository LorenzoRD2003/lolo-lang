// Representar el programa completo en IR.
// Por ahora no hay funciones, cuando haya funciones mucha de la logica de aca ira a otro archivo.

use crate::ir::{
  block::BlockData,
  ids::{BlockId, InstId, ValueId},
  inst::InstData,
  types::IrType,
  value::ValueData,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IrModule {
  name: String,
  entry_block: Option<BlockId>,
  // params: Vec<...> en un futuro
  /// tipo de los valores de retorno del programa
  return_type: IrType,
  /// instrucciones del programa
  insts: Vec<InstData>,
  /// bloques del programa
  blocks: Vec<BlockData>,
  /// valores del programa
  values: Vec<ValueData>,
}

impl IrModule {
  pub(crate) fn new(name: String, return_type: IrType) -> Self {
    Self {
      name,
      entry_block: None,
      return_type,
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

  #[cfg(any(test, feature = "ir-verify"))]
  pub(crate) fn entry_block_opt(&self) -> Option<BlockId> {
    self.entry_block
  }

  pub(crate) fn inst(&self, id: InstId) -> &InstData {
    &self.insts[id.0]
  }

  pub(crate) fn add_inst(&mut self, data: InstData) {
    self.insts.push(data);
  }

  #[cfg(any(test, feature = "ir-verify"))]
  pub(crate) fn inst_count(&self) -> usize {
    self.insts.len()
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

  pub(crate) fn block_count(&self) -> usize {
    self.blocks.len()
  }

  pub(crate) fn value(&self, id: ValueId) -> &ValueData {
    &self.values[id.0]
  }

  pub(crate) fn add_value(&mut self, data: ValueData) {
    self.values.push(data);
  }

  #[cfg(any(test, feature = "ir-verify"))]
  pub(crate) fn value_count(&self) -> usize {
    self.values.len()
  }

  pub(crate) fn return_type(&self) -> IrType {
    self.return_type
  }

  pub(crate) fn name(&self) -> &str {
    &self.name
  }
}
