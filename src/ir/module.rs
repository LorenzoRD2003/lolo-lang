// Representar el programa completo en IR.
// Por ahora no hay funciones, cuando haya funciones mucha de la logica de aca ira a otro archivo.

#[cfg(test)]
use std::collections::{BTreeSet, VecDeque};

#[cfg(test)]
use crate::ir::{inst::InstKind, value::IrConstant};

use crate::ir::{
  block::BlockData,
  ids::{BlockId, InstId, ValueId},
  inst::InstData,
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

  #[allow(dead_code)]
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

  #[cfg(test)]
  #[allow(dead_code)]
  /// Obtiene los predecesores de un bloque en el CFG del modulo.
  /// Es supralineal, luego TODO seria ideal construir una estructura CFG y usar eso.
  fn predecessors(&self, block: BlockId) -> Vec<BlockId> {
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

  #[cfg(test)]
  /// Obtiene los sucesores de un bloque en el CFG del modulo.
  fn successors(&self, block: BlockId) -> Vec<BlockId> {
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

  // ======================
  //   Helpers para tests
  // ======================

  #[cfg(test)]
  fn reachable_blocks(&self) -> Vec<BlockId> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(self.entry_block());
    while let Some(block) = queue.pop_front() {
      if !seen.insert(block.0) {
        continue;
      }
      for succ in self.successors(block) {
        queue.push_back(succ);
      }
    }
    seen.into_iter().map(BlockId).collect()
  }

  #[cfg(test)]
  pub(crate) fn reachable_inst_ids(&self) -> Vec<InstId> {
    let mut seen = BTreeSet::new();
    for block in self.reachable_blocks() {
      // insert phi instructions before the other instructions
      for inst_id in self.block(block).phis() {
        seen.insert(inst_id.0);
      }
      for inst_id in self.block(block).insts() {
        seen.insert(inst_id.0);
      }
      seen.insert(self.block(block).terminator().0);
    }
    seen.into_iter().map(InstId).collect()
  }

  #[cfg(test)]
  pub(crate) fn count_insts_by_kind(&self, pred: impl Fn(&InstKind) -> bool) -> usize {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter(|inst_id| pred(&self.inst(*inst_id).kind))
      .count()
  }

  #[cfg(test)]
  pub(crate) fn const_results(&self, constant: IrConstant) -> Vec<ValueId> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match &self.inst(inst_id).kind {
        InstKind::Const(c) if *c == constant => self.inst(inst_id).result,
        _ => None,
      })
      .collect()
  }

  #[cfg(test)]
  pub(crate) fn print_operands(&self) -> Vec<ValueId> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match &self.inst(inst_id).kind {
        InstKind::Print(value_id) => Some(*value_id),
        _ => None,
      })
      .collect()
  }

  #[cfg(test)]
  pub(crate) fn phi_results_with_types(&self) -> Vec<(ValueId, IrType)> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match self.inst(inst_id).kind {
        InstKind::Phi { .. } => {
          let result = self
            .inst(inst_id)
            .result
            .expect("todo phi debe tener resultado");
          Some((result, self.value(result).ty().clone()))
        }
        _ => None,
      })
      .collect()
  }

  #[cfg(test)]
  pub(crate) fn return_values(&self) -> Vec<Option<ValueId>> {
    self
      .reachable_inst_ids()
      .into_iter()
      .filter_map(|inst_id| match &self.inst(inst_id).kind {
        InstKind::Return { value } => Some(*value),
        _ => None,
      })
      .collect()
  }
}
