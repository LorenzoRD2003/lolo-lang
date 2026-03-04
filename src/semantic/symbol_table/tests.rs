use proptest::prelude::*;

use crate::semantic::{scope::ScopeArena, symbol_table::SymbolTable};

#[test]
fn symbol_is_inserted_and_retrievable() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);
  let name = "a";
  let sym = table.add_symbol(name, 0..1);
  assert_eq!(table.symbol(sym).name(), name);
}

#[test]
fn resolve_finds_symbol_in_current_scope() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);
  let name = "x";
  let sym = table.add_symbol(name, 0..1);
  assert_eq!(table.resolve(name), Some(sym));
}

#[test]
fn resolve_walks_up_scopes() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);
  let name = "a";
  let sym = table.add_symbol(&name, 0..1);
  table.enter_scope();
  assert_eq!(table.resolve(&name), Some(sym));
}

/// Este test indica que en lolo-lang si hay dos declaraciones de una misma variable (lo que debe ser)
/// en scopes diferentes, entonces en el inner scope se hace shadowing de la declaracion del outer scope.
#[test]
fn shadowing_prefers_inner_scope() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);

  let name = "x";
  let outer = table.add_symbol(&name, 0..1);

  table.enter_scope();
  let inner = table.add_symbol(&name, 2..3);
  assert_eq!(table.resolve(&name), Some(inner));

  table.exit_scope();
  assert_eq!(table.resolve(&name), Some(outer));
}

#[test]
fn exit_scope_on_root_is_safe() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);
  table.enter_scope();
  let _root = table.current_scope();
  table.exit_scope(); // vuelve a None o root padre
  table.exit_scope(); // no debe crashear
}

#[test]
fn all_symbols_in_scope_returns_correct_symbols() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);

  table.enter_scope();
  let outer_scope = table.current_scope().unwrap();
  let a = table.add_symbol("a", 0..1);
  let b = table.add_symbol("b", 2..3);

  table.enter_scope();
  let inner_scope = table.current_scope.unwrap();
  let c = table.add_symbol("c", 4..5);
  let d = table.add_symbol("d", 6..7);

  table.exit_scope();
  let e = table.add_symbol("e", 8..9);

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

#[test]
fn variable_was_declared_in_current_scope() {
  let scopes = ScopeArena::new();
  let mut table = SymbolTable::new(scopes);
  table.enter_global_scope();
  table.enter_scope();
  let name = "x";
  let symbol = table.add_symbol(&name, 0..1);

  assert_eq!(table.declared_in_scope(&name), Some(symbol));
  table.exit_scope();
  assert!(table.declared_in_scope(&name).is_none());
  table.enter_scope();
  assert!(table.declared_in_scope(&name).is_none());
}

proptest! {
  #[test]
  fn resolve_never_returns_outer_symbol_if_shadowed(name in "[a-z]{1,8}") {
    let scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    let var = name.clone();
    let outer = table.add_symbol(&var, 0..1);
    table.enter_scope();
    let inner = table.add_symbol(&var, 2..3);
    prop_assert_eq!(table.resolve(&var), Some(inner));
    prop_assert_ne!(table.resolve(&var), Some(outer));
  }

  #[test]
  fn resolve_finds_symbol_in_any_parent_scope(name in "[a-z]{1,8}", difference in 5..10) {
    let scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    let sym = table.add_symbol(&name, 0..1);
    for _ in 0..difference {
      table.enter_scope();
    }
    prop_assert_eq!(table.resolve(&name), Some(sym));
  }

  #[test]
  fn symbol_ids_are_unique(names in prop::collection::vec("[a-z]{1,8}", 1..50)) {
    let scopes = ScopeArena::new();
    let mut table = SymbolTable::new(scopes);
    let mut ids = Vec::new();
    for name in names {
      let id = table.add_symbol(
        &name,
        0..1,
      );
      ids.push(id.0);
    }
    ids.sort();
    ids.dedup();
    prop_assert_eq!(ids.len(), ids.len());
  }
}
