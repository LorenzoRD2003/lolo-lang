use std::collections::HashMap;

use crate::{ir::ids::ValueId, semantic::SymbolId};

/// SSA-env responde la siguiente pregunta: en este punto exacto del control flow,
/// cual es el ValueId SSA actual de cada simbolo fuente.
/// No es "la variable" en abstracto, sino su version viva en este punto.
#[derive(Debug, Clone)]
pub(crate) struct SsaEnv {
  // mapa de simbolos del codigo fuente a valores SSA
  current_values: HashMap<SymbolId, ValueId>,
}

// Por ejemplo, cuando entro a un If, el SSA-env representa el estado actual antes del branch.
// Lo clono para ambas ramas.

impl SsaEnv {
  pub(crate) fn new() -> Self {
    Self {
      current_values: HashMap::new(),
    }
  }

  pub(crate) fn get(&self, symbol: SymbolId) -> Option<ValueId> {
    self.current_values.get(&symbol).copied()
  }

  pub(crate) fn set(&mut self, symbol: SymbolId, value: ValueId) {
    self.current_values.insert(symbol, value);
  }

  pub(crate) fn clone_for_branch(&self) -> Self {
    self.clone()
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = (&SymbolId, &ValueId)> {
    self.current_values.iter()
  }
}

// #[derive(Debug, Clone)]
// pub(crate) struct BlockSsaState {
//   pub(crate) incoming: HashMap<SymbolId, ValueId>,
//   pub(crate) outgoing: HashMap<SymbolId, ValueId>,
// }
