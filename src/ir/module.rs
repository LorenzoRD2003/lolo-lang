// Representar el programa completo en IR.
// Por ahora no hay funciones, cuando haya funciones mucha de la logica de aca ira a otro archivo.

use crate::ir::{
  block::BlockData,
  ids::{BlockId, InstId, ValueId},
  inst::{InstData, InstKind},
  types::IrType,
  value::ValueData,
};

#[derive(Debug, Clone)]
pub(crate) struct IrModule {
  #[allow(dead_code)]
  name: String,
  entry_block: Option<BlockId>,
  // params: Vec<...> en un futuro
  /// tipo de los valores de retorno del programa
  #[allow(dead_code)]
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

  #[allow(dead_code)]
  pub(crate) fn entry_block(&self) -> BlockId {
    self
      .entry_block
      .expect("debe haber un bloque principal del programa")
  }

  pub(crate) fn set_entry_block(&mut self, main_block: BlockId) {
    self.entry_block = Some(main_block);
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

  #[allow(dead_code)]
  /// Obtiene los predecesores de un bloque en el CFG del modulo.
  /// Es supralineal, luego TODO seria ideal construir una estructura CFG y usar eso.
  pub(crate) fn predecessors(&self, block: BlockId) -> Vec<BlockId> {
    let mut preds = vec![];
    for i in 0..self.blocks.len() {
      let block_id = BlockId(i);
      preds.extend(
        self
          .successors(block_id)
          .iter()
          .filter(|&&succ| succ == block),
      );
    }
    preds
  }

  #[allow(dead_code)]
  /// Obtiene los sucesores de un bloque en el CFG del modulo.
  pub(crate) fn successors(&self, block: BlockId) -> Vec<BlockId> {
    let terminator = self.block(block).terminator();
    match self.inst(terminator).kind {
      InstKind::Jump { target } => vec![target],
      InstKind::Branch {
        if_block,
        else_block,
        ..
      } => vec![if_block, else_block],
      InstKind::Return { .. } => vec![],
      _ => unreachable!(),
    }
  }
}
