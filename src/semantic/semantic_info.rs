use std::collections::HashMap;

use crate::{
  ast::{
    ast::{BlockId, ExprId, StmtId},
    expr::ConstValue,
  },
  semantic::{category::ExprCategory, scope::ScopeId, symbol::SymbolId, types::Type},
};

#[derive(Debug, Clone)]
pub struct SemanticInfo {
  // Mapeos desde los IDs del AST a metadata semantica
  /// ExprId -> SemanticExprInfo. La clave es el ID en el AST.
  expr_info_by_id: HashMap<ExprId, SemanticExprInfo>,
  /// StmtId -> SemanticStmtInfo. La clave es el ID en el AST.
  stmt_info_by_id: HashMap<StmtId, SemanticStmtInfo>,
  /// BlockId -> SemanticBlockInfo. La clave es el ID en el AST.
  block_info_by_id: HashMap<BlockId, SemanticBlockInfo>,
}

impl SemanticInfo {
  pub fn new() -> Self {
    Self {
      expr_info_by_id: HashMap::new(),
      stmt_info_by_id: HashMap::new(),
      block_info_by_id: HashMap::new(),
    }
  }

  /// Obtiene la informacion semantica correspondiente a la expresion de ID ExprId.
  pub fn expr_info(&self, expr_id: ExprId) -> &SemanticExprInfo {
    self
      .expr_info_by_id
      .get(&expr_id)
      .expect("ExprId no encontrado en el AST")
  }

  /// Obtiene la informacion semantica correspondiente al statement de ID StmtId.
  pub fn stmt_info(&self, stmt_id: StmtId) -> &SemanticStmtInfo {
    self
      .stmt_info_by_id
      .get(&stmt_id)
      .expect("StmtId no encontrado en el AST")
  }

  /// Obtiene la informacion semantica correspondiente al bloque de ID BlockId.
  pub fn block_info(&self, block_id: BlockId) -> &SemanticBlockInfo {
    self
      .block_info_by_id
      .get(&block_id)
      .expect("BlockId no encontrado en el AST")
  }

  // /// Obtiene el simbolo para la variable de ID ExprId. Para otras expresiones, deberia devolver None.
  // pub fn symbol_for_var(&self, expr_id: ExprId) -> Option<SymbolId> {
  //   self.expr_info(expr_id).symbol
  // }

  // /// Obtiene el scope en el que vive el statement de ID StmtId.
  // pub fn scope_for_stmt(&self, stmt_id: StmtId) -> ScopeId {
  //   self.stmt_info(stmt_id).scope
  // }

  pub fn insert_expr_info(&mut self, expr_id: ExprId, info: SemanticExprInfo) {
    self.expr_info_by_id.insert(expr_id, info);
  }

  pub fn insert_stmt_info(&mut self, stmt_id: StmtId, info: SemanticStmtInfo) {
    self.stmt_info_by_id.insert(stmt_id, info);
  }

  pub fn insert_block_info(&mut self, block_id: BlockId, info: SemanticBlockInfo) {
    self.block_info_by_id.insert(block_id, info);
  }

  pub fn stmt_info_by_id(&self) -> &HashMap<StmtId, SemanticStmtInfo> {
    &self.stmt_info_by_id
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticExprInfo {
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
  pub fn new(
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

  pub fn symbol(&self) -> Option<SymbolId> {
    self.symbol
  }

  pub fn r#type(&self) -> Type {
    self.r#type
  }

  pub fn category(&self) -> ExprCategory {
    self.category
  }

  pub fn scope(&self) -> ScopeId {
    self.scope
  }

  pub fn compile_time_constant(&self) -> Option<&ConstValue> {
    self.compile_time_constant.as_ref()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticStmtInfo {
  /// Scope del statement.
  scope: ScopeId,
  /// Simbolo declarado en esta expresion (si es una asignacion de variable).
  symbol_declared: Option<SymbolId>,
}

impl SemanticStmtInfo {
  pub fn new(scope: ScopeId, symbol_declared: Option<SymbolId>) -> Self {
    Self {
      scope,
      symbol_declared,
    }
  }

  pub fn scope(&self) -> ScopeId {
    self.scope
  }

  pub fn symbol_declared(&self) -> Option<SymbolId> {
    self.symbol_declared
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticBlockInfo {
  /// Scope asociado al bloque.
  scope: ScopeId,
  /// El statement terminador del bloque. Es `None`` si y solo si el bloque esta vacio.
  terminator: Option<StmtId>,
}

impl SemanticBlockInfo {
  pub fn new(scope: ScopeId, terminator: Option<StmtId>) -> Self {
    Self { scope, terminator }
  }

  pub fn scope(&self) -> ScopeId {
    self.scope
  }

  pub fn terminator(&self) -> Option<StmtId> {
    self.terminator
  }
}
