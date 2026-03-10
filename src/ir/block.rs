// Responsabilidad: representar un bloque basico como una secuencia de instrucciones.
// Pensar esto en el sentido de un CFG, no en el sentido de los bloques como expresiones del AST.

use crate::ir::ids::InstId;

#[derive(Debug, Clone)]
pub(crate) struct BlockData {
  /// Instrucciones PHI del bloque
  phis: Vec<InstId>,
  /// Instrucciones NO TERMINADORAS del bloque.
  insts: Vec<InstId>,
  /// Instruccion terminadora del bloque.
  terminator: Option<InstId>,
}

impl BlockData {
  pub(crate) fn new_block() -> Self {
    Self {
      phis: Vec::new(),
      insts: Vec::new(),
      terminator: None,
    }
  }

  pub(crate) fn phis(&self) -> &[InstId] {
    &self.phis
  }

  pub(crate) fn add_phi(&mut self, phi: InstId) {
    self.insts.push(phi);
  }

  pub(crate) fn insts(&self) -> &[InstId] {
    &self.insts
  }

  pub(crate) fn add_inst(&mut self, inst: InstId) {
    self.insts.push(inst);
  }

  pub(crate) fn has_terminator(&self) -> bool {
    self.terminator.is_some()
  }

  /// Precondicion: el bloque debe tener un terminador
  pub(crate) fn terminator(&self) -> InstId {
    self.terminator.expect("un bloque debe tener terminador")
  }

  pub(crate) fn set_terminator(&mut self, terminator: InstId) {
    self.terminator = Some(terminator);
  }
}
