// El “Semantic Analyzer” (como el parser, pero semantico). Es el archivo principal del modulo `semantic`
// Responsabilidades:
// - Recorre AST / Blocks / Statements / Expressions
// - Gestiona scopes
// - Consulta la symbol table
// - Emite errores semanticos

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{ConstValue, Expr, UnaryOp},
    program::Program,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    category::ExprCategory, error::SemanticError, scope::ScopeId, symbol::{self, SymbolId}, symbol_table::SymbolTable, types::Type
  },
};
use std::{any::Any, collections::HashMap};

/// Estructura que transforma un AST puro en algo semanticamente enriquecido y chequeado.
#[derive(Debug)]
pub(crate) struct SemanticInfo<'a> {
  /// El AST. Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// Para insertar y resolver símbolos. Mantiene scopes activos y arena de scopes.
  symbol_table: &'a mut SymbolTable<'a>,
  /// Donde se van acumulando los errores encontrados durante el analisis semantico.
  diagnostics: Vec<Diagnostic>,
  // Mapeos desde los IDs del AST a metadata semantica
  /// ExprId -> SemanticExprInfo. La clave es el ID en el AST.
  expr_info_by_id: HashMap<ExprId, SemanticExprInfo>,
  /// StmtId -> SemanticStmtInfo. La clave es el ID en el AST.
  stmt_info_by_id: HashMap<StmtId, SemanticStmtInfo>,
  /// BlockId -> SemanticBlockInfo. La clave es el ID en el AST.
  block_info_by_id: HashMap<BlockId, SemanticBlockInfo>,
}

impl<'a> SemanticInfo<'a> {
  pub(crate) fn new(ast: &'a Ast, symbol_table: &'a mut SymbolTable<'a>) -> Self {
    Self {
      ast,
      symbol_table,
      diagnostics: Vec::new(),
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

  /// Obtiene el scope de analisis actual, usando la SymbolTable.
  pub(crate) fn current_scope(&self) -> Option<ScopeId> {
    self.symbol_table.current_scope()
  }

  /// Devuelve una referencia iterable a los diagnosticos de error.
  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Convierte el error a Diagnostic, lo acumula en la lista de errores.
  fn emit_error(&mut self, err: &SemanticError) {
    self.diagnostics.push(err.to_diagnostic())
  }

  /// Para una expresion, determina su tipo, categoria, simbolo (si es un VarExpr).
  /// Agrega un diagnostic si es `UndefinedVariable`
  pub(crate) fn analyze_expr(&mut self, expr_id: ExprId) {
    // El objetivo es llegar a construir un SemanticExprInfo y agregarlo a expr_info_by_id
    let symbol = self.symbol_for_var(expr_id);
    let r#type = self.analyze_expr_type(expr_id);
    let category = self.analyze_expr_category(expr_id);
    // let scope = self.symbol_table.scopes().;
    let is_compile_time_constant = todo!();
    // let semantic_expr_info =
    //   SemanticExprInfo::new(symbol, r#type, category, scope, is_compile_time_constant);
    // self.expr_info_by_id.insert(expr_id, semantic_expr_info);
  }

  fn analyze_expr_type(&mut self, expr_id: ExprId) -> Type {
    let expr = self.ast.expr(expr_id);
    let span = self.ast.expr_span(expr_id);
    match expr {
      Expr::Const(ConstValue::Int(_)) => Type::Int32,
      Expr::Const(ConstValue::Bool(_)) => Type::Bool,
      Expr::Var(name) => match self.symbol_for_var(expr_id) {
        Some(symbol_id) => self.symbol_table.symbol(symbol_id).r#type(),
        None => {
          self.emit_error(&SemanticError::UndefinedVariable { name, span });
          Type::DefaultErrorType
        }
      },
      Expr::Unary(unary) => {
        let operand_type = self.analyze_expr_type(unary.operand);
        let expected_operand_type = unary.op.compatible_operand_type();
        if operand_type == expected_operand_type {
          unary.op.result_type()
        } else {
          self.emit_error(&SemanticError::TypeMismatch {
            expected: expected_operand_type,
            found: operand_type,
            span: self.ast.expr_span(unary.operand),
          });
          Type::DefaultErrorType
        }
      }
      Expr::Binary(binary) => {
        let lhs_type = self.analyze_expr_type(binary.lhs);
        let rhs_type = self.analyze_expr_type(binary.rhs);
        let (lhs_expected_type, rhs_expected_type) = binary.op.compatible_operand_types();
        if lhs_type != lhs_expected_type {
          self.emit_error(&SemanticError::TypeMismatch {
            expected: lhs_expected_type,
            found: lhs_type,
            span: self.ast.expr_span(binary.lhs),
          });
        }
        if rhs_type != rhs_expected_type {
          self.emit_error(&SemanticError::TypeMismatch {
            expected: rhs_expected_type,
            found: rhs_type,
            span: self.ast.expr_span(binary.rhs),
          });
        }
        if lhs_type == lhs_expected_type && rhs_type == rhs_expected_type {
          binary.op.result_type()
        } else {
          Type::DefaultErrorType
        }
      }
    }
  }

  fn analyze_expr_category(&mut self, expr_id: ExprId) -> ExprCategory {
    let expr = self.ast.expr(expr_id);
    let span = self.ast.expr_span(expr_id);
    match expr {
      Expr::Const(_) => todo!(),
      Expr::Var(_) => todo!(),
      Expr::Unary(_) | Expr::Binary(_) => todo!(),
    }

    todo!()
  }

  /// Analiza un statement, chequeando redeclaraciones (Let)
  /// y asignabilidad de expresiones (Return, Print).
  /// Agrega un diagnostic si es `RedeclaredVariable`, `TypeMismatch`, o `InvalidAssignmentTarget`.
  pub(crate) fn analyze_stmt(&mut self, stmt_id: StmtId) {}

  /// Crea un nuevo scope, analiza statements dentro del bloque, y marca terminador del bloque.
  pub(crate) fn analyze_block(&mut self, block_id: BlockId) {}

  /// Punto de entrada general, itera sobre los bloques del programa
  pub(crate) fn analyze_program(&mut self, program: Program) {}
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
  /// Es una constante conocida en compile-time. Util para optimizaciones.
  is_compile_time_constant: bool,
}

impl SemanticExprInfo {
  pub(crate) fn new(
    symbol: Option<SymbolId>,
    r#type: Type,
    category: ExprCategory,
    scope: ScopeId,
    is_compile_time_constant: bool,
  ) -> Self {
    Self {
      symbol,
      r#type,
      category,
      scope,
      is_compile_time_constant,
    }
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
