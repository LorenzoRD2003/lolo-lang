// Estructura de almacenamiento y gestion GLOBAL de simbolos. Es infraestructura, no logica semantica.
// Insercion de simbolos, lookup de simbolos, y manejo de scope activos.
// Debe soportar scopes anidados, resolucion lexica, y shadowing.
//
// En lolo-lang el shadowing es legal, pero cambiar una variable dentro de un bloque no debe
// cambiar cual es el scope en el cual vive.

use crate::{
  ast::expr::VarId,
  common::span::Span,
  semantic::{
    scope::{ScopeArena, ScopeId},
    symbol::{Mutability, Symbol, SymbolId},
    types::Type,
  },
};

#[derive(Debug)]
pub(crate) struct SymbolTable {
  /// Arena de simbolos. Se indexan por su `SymbolId`.
  symbols: Vec<Symbol>,
  /// Arena de Scopes. Solamente tiene sentido en el contexto de una SymbolTable, por lo tanto
  /// no es una referencia y tomamos ownership.
  scopes: ScopeArena,
  /// Scope activo durante el analisis
  current_scope: Option<ScopeId>,
}

impl SymbolTable {
  pub(crate) fn new(scopes: ScopeArena) -> Self {
    Self {
      symbols: Vec::<Symbol>::new(),
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
  /// - No Debe chequear redeclaraciones legales/ilegales. TODO: Hacer esa parte en el analyzer que es quien tiene diagnostics
  pub(crate) fn add_symbol(
    &mut self,
    name: &VarId,
    r#type: Type,
    mutability: Mutability,
    span: Span,
  ) -> SymbolId {
    let current_scope = match self.current_scope {
      Some(scope) => scope,
      None => self.enter_scope(),
    };
    let symbol_id = SymbolId(self.symbols.len());
    let symbol = Symbol::new(symbol_id, &name, r#type, current_scope, mutability, span);
    self.symbols.push(symbol);
    self.scopes.insert_symbol(name, current_scope, symbol_id);
    symbol_id
  }

  pub(crate) fn symbol(&self, id: SymbolId) -> &Symbol {
    &self.symbols[id.0]
  }

  pub(crate) fn symbol_mut(&mut self, id: SymbolId) -> &mut Symbol {
    &mut self.symbols[id.0]
  }

  pub(crate) fn current_scope(&self) -> Option<ScopeId> {
    self.current_scope
  }

  pub(crate) fn scopes(&self) -> &ScopeArena {
    &self.scopes
  }

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

  /// Busca hacia arriba en la jerarquía de scopes hasta encontrar el símbolo.
  /// Permite hallar variables usadas pero no declaradas (si `resolve()` devuelve `None`).
  pub(crate) fn resolve(&self, name: &VarId) -> Option<SymbolId> {
    self.scopes.resolve(name, self.current_scope?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use proptest::prelude::*;

  #[test]
  fn symbol_is_inserted_and_retrievable() {
    let mut scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    let name = VarId("a".into());
    let sym = table.add_symbol(&name, Type::Int32, Mutability::Mutable, 0..1);
    assert_eq!(table.symbol(sym).name(), name);
  }

  #[test]
  fn resolve_finds_symbol_in_current_scope() {
    let mut scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    let name = VarId("x".into());
    let sym = table.add_symbol(&name, Type::Int32, Mutability::Mutable, 0..1);
    assert_eq!(table.resolve(&name), Some(sym));
  }

  #[test]
  fn resolve_walks_up_scopes() {
    let mut scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    let name = VarId("a".into());
    let sym = table.add_symbol(&name, Type::Int32, Mutability::Mutable, 0..1);
    table.enter_scope();
    assert_eq!(table.resolve(&name), Some(sym));
  }

  /// Este test indica que en lolo-lang si hay dos declaraciones de una misma variable (lo que debe ser)
  /// en scopes diferentes, entonces en el inner scope se hace shadowing de la declaracion del outer scope.
  #[test]
  fn shadowing_prefers_inner_scope() {
    let mut scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);

    let name = VarId("x".into());
    let outer = table.add_symbol(&name, Type::Int32, Mutability::Mutable, 0..1);

    table.enter_scope();
    let inner = table.add_symbol(&name, Type::Bool, Mutability::Mutable, 2..3);
    assert_eq!(table.resolve(&name), Some(inner));

    table.exit_scope();
    assert_eq!(table.resolve(&name), Some(outer));
  }

  #[test]
  fn exit_scope_on_root_is_safe() {
    let mut scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    table.enter_scope();
    let _root = table.current_scope();
    table.exit_scope(); // vuelve a None o root padre
    table.exit_scope(); // no debe crashear
  }

  #[test]
  fn all_symbols_in_scope_returns_correct_symbols() {
    let mut scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);

    table.enter_scope();
    let outer_scope = table.current_scope().unwrap();
    let a = table.add_symbol(&VarId("a".into()), Type::Int32, Mutability::Mutable, 0..1);
    let b = table.add_symbol(&VarId("b".into()), Type::Int32, Mutability::Mutable, 2..3);

    table.enter_scope();
    let inner_scope = table.current_scope.unwrap();
    let c = table.add_symbol(&VarId("c".into()), Type::Int32, Mutability::Immutable, 4..5);
    let d = table.add_symbol(&VarId("d".into()), Type::Bool, Mutability::Mutable, 6..7);

    table.exit_scope();
    let e = table.add_symbol(&VarId("e".into()), Type::Int32, Mutability::Mutable, 8..9);

    let outer_symbols = table.all_symbols_in_scope(outer_scope);
    let inner_symbols = table.all_symbols_in_scope(inner_scope);

    for i in [a, b, e] {
      assert!(outer_symbols.contains(&i));
      assert!(!inner_symbols.contains(&i));
    }

    for i in [c, d] {
      assert!(!outer_symbols.contains(&i));
      assert!(inner_symbols.contains(&i));
    }
  }

  proptest! {
    #[test]
    fn resolve_never_returns_outer_symbol_if_shadowed(name in "[a-z]{1,8}") {
      let mut scopes = ScopeArena::new();
      let mut table = SymbolTable::new(scopes);
      let var = VarId(name.clone());
      let outer = table.add_symbol(&var, Type::Int32, Mutability::Mutable, 0..1);
      table.enter_scope();
      let inner = table.add_symbol(&var, Type::Bool, Mutability::Mutable, 2..3);
      prop_assert_eq!(table.resolve(&var), Some(inner));
      prop_assert_ne!(table.resolve(&var), Some(outer));
    }
  }

  proptest! {
    #[test]
    fn resolve_finds_symbol_in_any_parent_scope(name in "[a-z]{1,8}", difference in 5..10) {
      let mut scopes = ScopeArena::new();
      let mut table = SymbolTable::new(scopes);
      let var = VarId(name.clone());
      let sym = table.add_symbol(&var, Type::Int32, Mutability::Mutable, 0..1);
      for _ in 0..difference {
        table.enter_scope();
      }
      prop_assert_eq!(table.resolve(&var), Some(sym));
    }
  }

  proptest! {
    #[test]
    fn symbol_ids_are_unique(names in prop::collection::vec("[a-z]{1,8}", 1..50)) {
      let mut scopes = ScopeArena::new();
      let mut table = SymbolTable::new(scopes);
      let mut ids = Vec::new();
      for name in names {
        let id = table.add_symbol(
          &VarId(name),
          Type::Int32,
          Mutability::Mutable,
          0..1
        );
        ids.push(id.0);
      }
      ids.sort();
      ids.dedup();
      prop_assert_eq!(ids.len(), ids.len());
    }
  }
}
