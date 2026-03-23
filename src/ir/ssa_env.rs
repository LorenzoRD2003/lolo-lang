use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{ir::ids::ValueId, semantic::SymbolId};

/// SSA-env responde la siguiente pregunta: en este punto exacto del control flow,
/// cual es el ValueId SSA actual de cada simbolo fuente.
/// No es "la variable" en abstracto, sino su version viva en este punto.
#[derive(Debug, Clone)]
pub(crate) struct SsaEnv {
  // mapa de simbolos del codigo fuente a valores SSA. Optimizado con COW via Rc.
  current_values: Rc<FxHashMap<SymbolId, ValueId>>,
  // simbolos modificados desde el ultimo checkpoint (util para optimizar merge de phis)
  modified: FxHashSet<SymbolId>,
}

// Por ejemplo, cuando entro a un If, el SSA-env representa el estado actual antes del branch.
// Lo clono para ambas ramas.

impl SsaEnv {
  pub(crate) fn new() -> Self {
    Self {
      current_values: Rc::new(FxHashMap::default()),
      modified: FxHashSet::default(),
    }
  }

  pub(crate) fn get(&self, symbol: SymbolId) -> Option<ValueId> {
    self.current_values.get(&symbol).copied()
  }

  pub(crate) fn set(&mut self, symbol: SymbolId, value: ValueId) {
    // Si el Rc esta compartido, Rc::make_mut clonara el mapa interno antes de modificarlo.
    // De lo contrario, modificara el mapa in-place.
    Rc::make_mut(&mut self.current_values).insert(symbol, value);
    self.modified.insert(symbol);
  }

  pub(crate) fn modified(&self) -> &FxHashSet<SymbolId> {
    &self.modified
  }

  pub(crate) fn clone_for_branch(&self) -> Self {
    Self {
      // Clonar el Rc es O(1). El mapa se clonara recien en el primer `set` si es necesario.
      current_values: Rc::clone(&self.current_values),
      modified: FxHashSet::default(),
    }
  }

  #[allow(dead_code)]
  pub(crate) fn iter(&self) -> impl Iterator<Item = (&SymbolId, &ValueId)> {
    self.current_values.iter()
  }
}

// #[derive(Debug, Clone)]
// pub(crate) struct BlockSsaState {
//   pub(crate) incoming: HashMap<SymbolId, ValueId>,
//   pub(crate) outgoing: HashMap<SymbolId, ValueId>,
// }
