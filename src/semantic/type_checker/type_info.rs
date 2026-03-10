use rustc_hash::FxHashMap;

use crate::{
  ast::ExprId,
  semantic::{symbol::SymbolId, types::SemanticType},
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TypeInfo {
  /// Mapa de expresiones a tipos. Se usa el algoritmo utilizado por el compilador de Rust,
  /// que no es resistente a colisiones pero es mas rapido.
  /// Para mas informacion, ver: https://docs.rs/rustc-hash/latest/rustc_hash/
  expr_types: FxHashMap<ExprId, SemanticType>,
  /// Mapa de simbolos a tipos. Se usa el algoritmo utilizado por el compilador de Rust,
  /// que no es resistente a colisiones pero es mas rapido.
  /// Para mas informacion, ver: https://docs.rs/rustc-hash/latest/rustc_hash/
  symbol_types: FxHashMap<SymbolId, SemanticType>,
}

impl TypeInfo {
  pub(crate) fn new() -> Self {
    Self {
      expr_types: FxHashMap::default(),
      symbol_types: FxHashMap::default(),
    }
  }

  pub(crate) fn insert_expr_type(&mut self, expr_id: ExprId, ty: SemanticType) {
    self.expr_types.insert(expr_id, ty);
  }

  pub(crate) fn type_of_expr<I: Into<ExprId>>(&self, expr_id: I) -> SemanticType {
    let expr_id = expr_id.into();
    *self.expr_types.get(&expr_id).expect("ya debe tener tipo")
  }

  pub(crate) fn set_symbol_type(&mut self, symbol: SymbolId, ty: SemanticType) {
    self.symbol_types.insert(symbol, ty);
  }

  pub(crate) fn type_of_symbol(&self, symbol: SymbolId) -> Option<SemanticType> {
    self.symbol_types.get(&symbol).copied()
  }
}
