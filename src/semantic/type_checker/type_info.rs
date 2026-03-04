use rustc_hash::FxHashMap;

use crate::{
  ast::ExprId,
  semantic::{symbol::SymbolId, types::Type},
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
  /// Mapa de expresiones a tipos. Se usa el algoritmo utilizado por el compilador de Rust,
  /// que no es resistente a colisiones pero es mas rapido.
  /// Para mas informacion, ver: https://docs.rs/rustc-hash/latest/rustc_hash/
  expr_types: FxHashMap<ExprId, Type>,
  /// Mapa de simbolos a tipos. Se usa el algoritmo utilizado por el compilador de Rust,
  /// que no es resistente a colisiones pero es mas rapido.
  /// Para mas informacion, ver: https://docs.rs/rustc-hash/latest/rustc_hash/
  symbol_types: FxHashMap<SymbolId, Type>,
}

impl TypeInfo {
  pub fn new() -> Self {
    Self {
      expr_types: FxHashMap::default(),
      symbol_types: FxHashMap::default(),
    }
  }

  pub fn insert_expr_type(&mut self, expr_id: ExprId, ty: Type) {
    self.expr_types.insert(expr_id, ty);
  }

  pub fn type_of_expr(&self, expr_id: ExprId) -> Type {
    *self.expr_types.get(&expr_id).expect("ya debe tener tipo")
  }

  pub fn set_symbol_type(&mut self, symbol: SymbolId, ty: Type) {
    self.symbol_types.insert(symbol, ty);
  }

  pub fn type_of_symbol(&self, symbol: SymbolId) -> Option<Type> {
    self.symbol_types.get(&symbol).copied()
  }
}
