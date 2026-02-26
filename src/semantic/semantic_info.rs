use std::collections::HashMap;

use crate::{
  ast::{
    ast::{BlockId, ExprId, StmtId},
    expr::ConstValue,
  },
  semantic::{category::ExprCategory, scope::ScopeId, symbol::SymbolId, types::Type},
};

#[derive(Debug, Clone)]
pub(crate) struct SemanticInfo {
  // Mapeos desde los IDs del AST a metadata semantica
  /// ExprId -> SemanticExprInfo. La clave es el ID en el AST.
  expr_info_by_id: HashMap<ExprId, SemanticExprInfo>,
  /// StmtId -> SemanticStmtInfo. La clave es el ID en el AST.
  stmt_info_by_id: HashMap<StmtId, SemanticStmtInfo>,
  /// BlockId -> SemanticBlockInfo. La clave es el ID en el AST.
  block_info_by_id: HashMap<BlockId, SemanticBlockInfo>,
}

impl SemanticInfo {
  pub(crate) fn new() -> Self {
    Self {
      expr_info_by_id: HashMap::new(),
      stmt_info_by_id: HashMap::new(),
      block_info_by_id: HashMap::new(),
    }
  }

  /// Obtiene la informacion semantica correspondiente a la expresion de ID ExprId.
  pub(crate) fn expr_info(&self, expr_id: ExprId) -> &SemanticExprInfo {
    self
      .expr_info_by_id
      .get(&expr_id)
      .expect("ExprId no encontrado en el AST")
  }

  /// Obtiene la informacion semantica correspondiente al statement de ID StmtId.
  pub(crate) fn stmt_info(&self, stmt_id: StmtId) -> &SemanticStmtInfo {
    self
      .stmt_info_by_id
      .get(&stmt_id)
      .expect("StmtId no encontrado en el AST")
  }

  /// Obtiene la informacion semantica correspondiente al bloque de ID BlockId.
  pub(crate) fn block_info(&self, block_id: BlockId) -> &SemanticBlockInfo {
    self
      .block_info_by_id
      .get(&block_id)
      .expect("BlockId no encontrado en el AST")
  }

  /// Obtiene el simbolo para la variable de ID ExprId. Para otras expresiones, deberia devolver None.
  pub(crate) fn symbol_for_var(&self, expr_id: ExprId) -> Option<SymbolId> {
    self.expr_info(expr_id).symbol
  }

  /// Obtiene el scope en el que vive el statement de ID StmtId.
  pub(crate) fn scope_for_stmt(&self, stmt_id: StmtId) -> ScopeId {
    self.stmt_info(stmt_id).scope
  }

  pub(crate) fn insert_expr_info(&mut self, expr_id: ExprId, info: SemanticExprInfo) {
    self.expr_info_by_id.insert(expr_id, info);
  }

  pub(crate) fn insert_stmt_info(&mut self, stmt_id: StmtId, info: SemanticStmtInfo) {
    self.stmt_info_by_id.insert(stmt_id, info);
  }

  pub(crate) fn insert_block_info(&mut self, block_id: BlockId, info: SemanticBlockInfo) {
    self.block_info_by_id.insert(block_id, info);
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SemanticExprInfo {
  /// Que simbolo representa la expresion (si aplica).
  symbol: Option<SymbolId>,
  /// Tipo de la expresion.
  r#type: Type,
  /// Categoria de la expresion.
  category: ExprCategory,
  /// Scope donde se evaluo la expresion.
  scope: ScopeId,
  /// Si es una constante conocida en compile-time. Util para optimizaciones.
  compile_time_constant: Option<ConstValue>,
}

impl SemanticExprInfo {
  pub(crate) fn new(
    symbol: Option<SymbolId>,
    r#type: Type,
    category: ExprCategory,
    scope: ScopeId,
    compile_time_constant: Option<ConstValue>,
  ) -> Self {
    Self {
      symbol,
      r#type,
      category,
      scope,
      compile_time_constant,
    }
  }

  pub(crate) fn symbol(&self) -> Option<SymbolId> {
    self.symbol
  }

  pub(crate) fn r#type(&self) -> Type {
    self.r#type
  }

  pub(crate) fn category(&self) -> ExprCategory {
    self.category
  }

  pub(crate) fn scope(&self) -> ScopeId {
    self.scope
  }

  pub(crate) fn compile_time_constant(&self) -> Option<&ConstValue> {
    self.compile_time_constant.as_ref()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SemanticStmtInfo {
  /// Scope del statement.
  scope: ScopeId,
  /// Si el statement es terminador de bloque.
  is_terminator: bool,
  /// Simbolos declarados en esta expresion.
  symbols_declared: Vec<SymbolId>,
}

impl SemanticStmtInfo {
  pub(crate) fn new(scope: ScopeId, is_terminator: bool, symbols_declared: Vec<SymbolId>) -> Self {
    Self {
      scope,
      is_terminator,
      symbols_declared,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SemanticBlockInfo {
  /// Scope asociado al bloque.
  scope: ScopeId,
  /// El statement terminador del bloque.
  terminator: StmtId,
}

impl SemanticBlockInfo {
  pub(crate) fn new(scope: ScopeId, terminator: StmtId) -> Self {
    Self { scope, terminator }
  }
}
