// Modelo conceptual del scope. Para sacarle peso al analyzer.
// - Representa scopes léxicos
// - Maneja nesting (if, bloques, etc.)
// - Shadowing rules
// Por ejemplo, aca viven el scope padre, las variables visibles y el lookup jerarquico.

use crate::{ast::expr::VarId, semantic::symbol::SymbolId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]

pub(crate) struct ScopeId(pub(crate) usize);

#[derive(Debug, Clone)]
pub(crate) struct Scope {
  id: ScopeId,
  /// Puede tener un padre para permitir scopes anidados.
  parent: Option<ScopeId>,
  /// Referencias a los simbolos declarados en este scope
  symbols: HashMap<VarId, SymbolId>,
}

impl Scope {
  pub(crate) fn add_symbol(&mut self, name: &VarId, id: SymbolId) {
    self.symbols.insert(name.clone(), id);
  }

  pub(crate) fn id(&self) -> ScopeId {
    self.id
  }

  pub(crate) fn parent(&self) -> Option<ScopeId> {
    self.parent
  }

  pub(crate) fn symbols(&self) -> &HashMap<VarId, SymbolId> {
    &self.symbols
  }
}

/// En `ScopeArena` van a vivir las referencias a todos los scopes
#[derive(Debug, Clone)]
pub(crate) struct ScopeArena {
  scopes: Vec<Scope>,
}

impl ScopeArena {
  pub(crate) fn new() -> Self {
    Self {
      scopes: Vec::<Scope>::new(),
    }
  }

  /// Crea un scope hijo del padre dado.
  pub(crate) fn new_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
    let new_scope_id = ScopeId(self.scopes.len());
    let scope = Scope {
      id: new_scope_id,
      parent,
      symbols: HashMap::<VarId, SymbolId>::new(),
    };
    self.scopes.push(scope);
    new_scope_id
  }

  /// Devuelve una referencia al scope.
  pub(crate) fn scope(&self, id: ScopeId) -> &Scope {
    &self.scopes[id.0]
  }

  /// Agrega simbolo a scope existente.
  pub(crate) fn insert_symbol(&mut self, name: &VarId, scope: ScopeId, symbol: SymbolId) {
    self.scopes[scope.0].add_symbol(name, symbol);
  }

  /// Devuelve el padre de un scope.
  pub(crate) fn parent_of(&self, scope: ScopeId) -> Option<ScopeId> {
    self.scope(scope).parent()
  }

  /// Busca hacia arriba en la jerarquía de scopes hasta encontrar el símbolo.
  /// Permite hallar variables usadas pero no declaradas (si `resolve()` devuelve `None`).
  pub(crate) fn resolve(&self, name: &VarId, current_scope: ScopeId) -> Option<SymbolId> {
    let mut current_scope_opt = Some(current_scope);
    while let Some(current_scope) = current_scope_opt {
      // Busco el simbolo en este scope
      let scope = self.scope(current_scope);
      // buscarlo en la SymbolTable para obtener su `VarId`
      match scope.symbols().get(name) {
        Some(symbol) => {
          return Some(symbol.clone());
        }
        None => current_scope_opt = scope.parent(),
      };
    }
    None
  }
}
