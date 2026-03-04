use rustc_hash::FxHashMap;

use crate::{
  ast::ast::{BlockId, ExprId, StmtId}, semantic::{id_generator::{ScopeId, SymbolId}, symbol::SymbolData},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionInfo {
  // Mapeos desde los IDs del AST a simbolos y scope
  /// ExprId -> SymbolId. La clave es el ID en el AST.
  expr_symbol_by_id: FxHashMap<ExprId, SymbolId>,
  /// ExprId -> ScopeId. La clave es el ID en el AST.
  expr_scope_by_id: FxHashMap<ExprId, ScopeId>,
  /// StmtId -> ScopeId. La clave es el ID en el AST.
  stmt_scope_by_id: FxHashMap<StmtId, ScopeId>,
  /// BlockId -> ScopeId. La clave es el ID en el AST.
  block_scope_by_id: FxHashMap<BlockId, ScopeId>,
  /// Simbolo declarado por este statement (solamente para bindings).
  stmt_declared_symbol: FxHashMap<StmtId, SymbolId>,
  /// Metadata adicional util por simbolo
  symbol_data: FxHashMap<SymbolId, SymbolData>,
}

impl ResolutionInfo {
  pub fn new() -> Self {
    Self {
      expr_symbol_by_id: FxHashMap::default(),
      expr_scope_by_id: FxHashMap::default(),
      stmt_scope_by_id: FxHashMap::default(),
      block_scope_by_id: FxHashMap::default(),
      stmt_declared_symbol: FxHashMap::default(),
      symbol_data: FxHashMap::default(),
    }
  }

  pub fn insert_expr_symbol(&mut self, expr: ExprId, symbol: SymbolId) {
    self.expr_symbol_by_id.insert(expr, symbol);
  }

  pub fn insert_expr_scope(&mut self, expr: ExprId, scope: ScopeId) {
    self.expr_scope_by_id.insert(expr, scope);
  }

  pub fn insert_stmt_scope(&mut self, stmt: StmtId, scope: ScopeId) {
    self.stmt_scope_by_id.insert(stmt, scope);
  }

  pub fn insert_block_scope(&mut self, block: BlockId, scope: ScopeId) {
    self.block_scope_by_id.insert(block, scope);
  }

  pub fn insert_declared_symbol(&mut self, stmt: StmtId, symbol: SymbolId) {
    self.stmt_declared_symbol.insert(stmt, symbol);
  }

  pub fn insert_symbol_data(&mut self, symbol: SymbolId, data: SymbolData) {
    self.symbol_data.insert(symbol, data);
  }

  pub fn symbol_of(&self, expr: ExprId) -> Option<SymbolId> {
    self.expr_symbol_by_id.get(&expr).copied()
  }

  pub fn has_symbol(&self, expr: ExprId) -> bool {
    self.expr_symbol_by_id.contains_key(&expr)
  }

  pub fn scope_of_expr(&self, expr: ExprId) -> Option<ScopeId> {
    self.expr_scope_by_id.get(&expr).copied()
  }

  pub fn scope_of_stmt(&self, stmt: StmtId) -> ScopeId {
    *self
      .stmt_scope_by_id
      .get(&stmt)
      .expect("todo statement debe tener scope")
  }

  pub fn scope_of_block(&self, block: BlockId) -> ScopeId {
    *self
      .block_scope_by_id
      .get(&block)
      .expect("todo bloque debe tener scope")
  }

  pub fn declared_symbol_of_stmt(&self, stmt: StmtId) -> Option<SymbolId> {
    self.stmt_declared_symbol.get(&stmt).copied()
  }

  pub fn symbol_data_of(&self, symbol: SymbolId) -> Option<SymbolData> {
    self.symbol_data.get(&symbol).copied()
  }

  // pub fn var_symbol_by_id(&self) -> &FxHashMap<ExprId, SymbolId> {
  //   &self.expr_symbol_by_id
  // }
}
