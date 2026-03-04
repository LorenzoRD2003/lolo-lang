// Estructura de almacenamiento y gestion GLOBAL de simbolos. Es infraestructura, no logica semantica.
// Insercion de simbolos, lookup de simbolos, y manejo de scope activos.
// Debe soportar scopes anidados, resolucion lexica, y shadowing.
//
// En lolo-lang el shadowing es legal, pero cambiar una variable dentro de un bloque no debe
// cambiar cual es el scope en el cual vive.

use crate::{
  common::{IdGenerator, IncrementalIdGenerator, Span},
  semantic::{
    scope::{ScopeArena, ScopeId},
    symbol::{Symbol, SymbolId},
  },
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SymbolTable {
  symbols: Vec<Symbol>,
  id_gen: IncrementalIdGenerator<SymbolId>,
  /// Arena de Scopes. Solamente tiene sentido en el contexto de una SymbolTable, por lo tanto
  /// no es una referencia y tomamos ownership.
  scopes: ScopeArena,
  /// Scope activo durante el analisis
  current_scope: Option<ScopeId>,
}

impl SymbolTable {
  pub(crate) fn new(scopes: ScopeArena) -> Self {
    Self {
      symbols: Vec::new(),
      id_gen: IncrementalIdGenerator::<SymbolId>::new(),
      scopes,
      current_scope: None,
    }
  }

  /// Crea un scope hijo del `current_scope` y lo hace activo.
  pub(crate) fn enter_scope(&mut self) -> ScopeId {
    let new_scope = self.scopes.new_scope(self.current_scope);
    self.current_scope = Some(new_scope);
    new_scope
  }

  /// Entra al scope global en caso de que no tenga scope actual.
  pub(crate) fn enter_global_scope(&mut self) {
    if self.current_scope.is_none() {
      self.enter_scope();
    }
  }

  /// Retrocede al padre del `current_scope`.
  /// Es defensivo: Si estamos en root_scope (o sea que el padre es None), no subimos.
  pub(crate) fn exit_scope(&mut self) {
    if let Some(scope) = self.current_scope
      && let Some(parent_scope) = self.scopes.parent_of(scope)
    {
      self.current_scope = Some(parent_scope)
    }
  }

  /// - Agrega un simbolo a la tabla.
  /// - Lo inserta en el `current_scope`.
  /// - Devuelve el `SymbolId` del simbolo.
  /// - No debe chequear redeclaraciones legales/ilegales.
  pub(crate) fn add_symbol(&mut self, name: &str, span: Span) -> SymbolId {
    let current_scope = match self.current_scope {
      Some(scope) => scope,
      None => self.enter_scope(),
    };
    let symbol_id = self.id_gen.next_id();
    let symbol = Symbol::new(symbol_id, name.to_string(), current_scope, span);
    self.symbols.push(symbol);
    self.scopes.insert_symbol(name, current_scope, symbol_id);
    symbol_id
  }

  pub(crate) fn symbol(&self, id: SymbolId) -> &Symbol {
    &self.symbols[id.0]
  }

  pub(crate) fn current_scope(&self) -> Option<ScopeId> {
    self.current_scope
  }

  #[cfg(test)]
  /// Devuelve todos los `SymbolId` para el scope con `ScopeId` dado. La complejidad es lineal.
  /// Por lo tanto, esta funcion no debe usarse en las partes criticas, sino solamente para debug y diagnostics.
  pub(crate) fn all_symbols_in_scope(&self, scope_id: ScopeId) -> Vec<SymbolId> {
    self
      .scopes
      .scope(scope_id)
      .symbols()
      .values()
      .copied()
      .collect()
  }

  /// Busca hacia arriba en la jerarquia de scopes hasta encontrar el simbolo.
  /// Permite hallar variables usadas pero no declaradas (si `resolve()` devuelve `None`).
  pub(crate) fn resolve(&self, name: &str) -> Option<SymbolId> {
    self.scopes.resolve(name, self.current_scope()?)
  }

  /// Busca el simbolo exactamente en el scope actual. Tiene que resolverlo para el scope actual,
  /// pero no para un scope por encima del actual.
  pub(crate) fn declared_in_scope(&mut self, name: &str) -> Option<SymbolId> {
    self.scopes.declared_in_scope(name, self.current_scope()?)
  }
}

#[cfg(test)]
mod tests;
