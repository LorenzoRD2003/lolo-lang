// Fase semantica de resolucion de nombres
// Debe responder unicamente estas preguntas:
// - Que simbolo corresponde a cada `Expr::Var`?
// - Donde fue declarado?
// - En que scope vive cada statement y expresion?
// - Hay errores de redeclaracion?
// - Hay variables no definidas?

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{Expr, VarId},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    resolver::{error::ResolverError, resolution_info::ResolutionInfo},
    scope::ScopeArena,
    symbol_table::SymbolTable,
  },
};

#[derive(Debug, PartialEq)]
pub struct NameResolver<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// La SymbolTable. Solamente tiene sentido en el contexto del analisis semantico, por lo tanto
  /// no es una referencia y tomamos ownership.
  symbol_table: SymbolTable,
  /// Donde se van acumulando los errores encontrados durante el analisis de resolucion de nombres.
  diagnostics: &'a mut Vec<Diagnostic>,
  /// Informacion sobre resolucion de nombres que se va acumulando.
  resolution_info: ResolutionInfo,
}

impl<'a> NameResolver<'a> {
  pub fn new(ast: &'a Ast, diagnostics: &'a mut Vec<Diagnostic>) -> Self {
    let scopes = ScopeArena::new();
    let mut resolver = Self {
      ast,
      symbol_table: SymbolTable::new(scopes),
      diagnostics,
      resolution_info: ResolutionInfo::new(),
    };
    resolver.symbol_table.enter_global_scope();
    resolver
  }

  pub fn resolve_program(&mut self, program: &Program) {
    self.resolve_block(program.main_block());
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Devuelve la informacion de resolucion de nombres, consumiendo `self`.
  pub fn into_resolution_info(self) -> ResolutionInfo {
    self.resolution_info
  }

  // ===================
  // Metodos internos
  // ===================

  /// Resuelve los nombres de variables para un bloque.
  fn resolve_block(&mut self, block_id: BlockId) {
    // La idea es que mientras dure esta funcion, todo se va a analizar para el scope de este bloque
    let block_scope = self.symbol_table.enter_scope();
    self
      .resolution_info
      .insert_block_scope(block_id, block_scope);
    let block = self.ast.block(block_id);
    for stmt_id in block.stmts() {
      self.resolve_stmt(*stmt_id);
    }
    self.symbol_table.exit_scope();
  }

  /// Resuelve los nombres de variables para un statement.
  fn resolve_stmt(&mut self, stmt_id: StmtId) {
    let scope = self
      .symbol_table
      .current_scope()
      .expect("todo statement debe tener scope");
    self.resolution_info.insert_stmt_scope(stmt_id, scope);
    let stmt = self.ast.stmt(stmt_id);
    match stmt {
      Stmt::LetBinding { var, initializer } => {
        // Confirmar que var sea Expr::Var
        let name = match self.ast.expr(var) {
          Expr::Var(name) => name,
          _ => return,
        };
        // Chequear redeclaracion en el scope actual
        let symbol = match self.symbol_table.was_declared_in_current_scope(&name) {
          Some(previous_symbol) => {
            let previous_stmt = self
              .resolution_info
              .get_stmt_for_symbol_declaration(previous_symbol)
              .expect("debe existir una declaracion anterior");
            self.emit_error(&ResolverError::RedeclaredVariable {
              name,
              span: self.ast.stmt_span(stmt_id),
              previous_span: self.ast.stmt_span(previous_stmt),
            });
            return;
          }
          // Insertar el simbolo en la tabla
          None => self.symbol_table.add_symbol(&name, self.ast.expr_span(var)),
        };
        self.resolution_info.insert_expr_symbol(var, symbol);
        self.resolution_info.insert_declared_symbol(stmt_id, symbol);
        // Resolver initializer por nombres
        self.resolve_expr(initializer);
      }
      Stmt::Assign { var, value_expr } => {
        let name = match self.ast.expr(var) {
          Expr::Var(name) => name,
          _ => return,
        };
        // Debemos verificar que la variable exista en algun scope
        let symbol = match self.symbol_table.resolve(&name) {
          Some(symbol_id) => symbol_id,
          None => {
            self.emit_error(&ResolverError::UndefinedVariable {
              name,
              span: self.ast.expr_span(var),
            });
            return;
          }
        };
        self.resolution_info.insert_expr_symbol(var, symbol);
        self.resolve_expr(value_expr);
      }
      Stmt::If {
        condition: _,
        if_block,
      } => {
        self.resolve_block(if_block);
      }
      Stmt::IfElse {
        condition: _,
        if_block,
        else_block,
      } => {
        self.resolve_block(if_block);
        self.resolve_block(else_block);
      }
      Stmt::Expr(expr) => {
        self.resolve_expr(expr);
      }
      _ => return,
    }
  }

  /// Resuelve los nombres de variables para una expresion.
  fn resolve_expr(&mut self, expr_id: ExprId) {
    let scope = self
      .symbol_table
      .current_scope()
      .expect("toda expresion debe tener scope");
    self.resolution_info.insert_expr_scope(expr_id, scope);

    match self.ast.expr(expr_id) {
      Expr::Var(name) => self.resolve_var_expr(expr_id, name),
      Expr::Unary(unary) => self.resolve_expr(unary.operand),
      Expr::Binary(binary) => {
        self.resolve_expr(binary.lhs);
        self.resolve_expr(binary.rhs);
      }
      Expr::Const(_) => {}
    };
  }

  /// Resuelve el nombre de una expresion de identificador.
  fn resolve_var_expr(&mut self, expr_id: ExprId, name: VarId) {
    match self.symbol_table.resolve(&name) {
      Some(symbol_id) => self.resolution_info.insert_expr_symbol(expr_id, symbol_id),
      None => self.emit_error(&ResolverError::UndefinedVariable {
        name,
        span: self.ast.expr_span(expr_id),
      }),
    }
  }

  fn emit_error(&mut self, err: &ResolverError) {
    self.diagnostics.push(err.to_diagnostic());
  }
}

#[cfg(test)]
pub mod tests;
