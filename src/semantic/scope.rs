// Modelo conceptual del scope. Para sacarle peso al analyzer.
// - Representa scopes léxicos
// - Maneja nesting (if, bloques, etc.)
// - Shadowing rules
// Por ejemplo, aca viven el scope padre, las variables visibles y el lookup jerarquico.

use rustc_hash::FxHashMap;

use crate::{
  common::{IdGenerator, IncrementalId, IncrementalIdGenerator},
  semantic::symbol::SymbolId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ScopeId(pub(crate) usize);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Scope {
  id: ScopeId,
  /// Puede tener un padre para permitir scopes anidados.
  parent: Option<ScopeId>,
  /// Referencias a los simbolos declarados en este scope
  symbols: FxHashMap<String, SymbolId>,
}

impl IncrementalId for ScopeId {
  fn from_usize(value: usize) -> Self {
    ScopeId(value)
  }
}

impl Scope {
  pub(crate) fn add_symbol(&mut self, name: &str, id: SymbolId) {
    self.symbols.insert(name.to_string(), id);
  }

  #[allow(dead_code)]
  pub(crate) fn id(&self) -> ScopeId {
    self.id
  }

  pub(crate) fn parent(&self) -> Option<ScopeId> {
    self.parent
  }

  pub(crate) fn symbols(&self) -> &FxHashMap<String, SymbolId> {
    &self.symbols
  }
}

/// En `ScopeArena` van a vivir las referencias a todos los scopes
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScopeArena {
  scopes: Vec<Scope>,
  id_gen: IncrementalIdGenerator<ScopeId>,
}

impl ScopeArena {
  pub(crate) fn new() -> Self {
    Self {
      scopes: Vec::new(),
      id_gen: IncrementalIdGenerator::<ScopeId>::new(),
    }
  }

  /// Crea un scope hijo del padre dado.
  pub(crate) fn new_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
    let scope_id = self.id_gen.next_id();
    let scope = Scope {
      id: scope_id,
      parent,
      symbols: FxHashMap::default(),
    };
    self.scopes.push(scope);
    scope_id
  }

  /// Devuelve una referencia al scope.
  pub(crate) fn scope(&self, id: ScopeId) -> &Scope {
    &self.scopes[id.0]
  }

  /// Devuelve una referencia mutable al scope.
  fn scope_mut(&mut self, id: ScopeId) -> &mut Scope {
    // self.scopes.get_mut(&id).expect("debe existir el scope")
    &mut self.scopes[id.0]
  }

  /// Agrega simbolo a scope existente.
  pub(crate) fn insert_symbol(&mut self, name: &str, scope: ScopeId, symbol: SymbolId) {
    self.scope_mut(scope).add_symbol(name, symbol);
  }

  /// Devuelve el padre de un scope.
  pub(crate) fn parent_of(&self, scope: ScopeId) -> Option<ScopeId> {
    self.scope(scope).parent()
  }

  /// Busca hacia arriba en la jerarquía de scopes hasta encontrar el símbolo.
  /// Permite hallar variables usadas pero no declaradas (si `resolve()` devuelve `None`).
  pub(crate) fn resolve(&self, name: &str, current_scope: ScopeId) -> Option<SymbolId> {
    let mut current_scope_opt = Some(current_scope);
    while let Some(current_scope) = current_scope_opt {
      // Busco el simbolo en este scope
      let scope = self.scope(current_scope);
      // buscarlo en la SymbolTable para obtener su `VarId`
      match scope.symbols().get(name) {
        Some(&symbol) => {
          return Some(symbol);
        }
        None => current_scope_opt = scope.parent(),
      };
    }
    None
  }

  pub(crate) fn declared_in_scope(&self, name: &str, scope: ScopeId) -> Option<SymbolId> {
    self.scope(scope).symbols().get(name).copied()
  }
}
