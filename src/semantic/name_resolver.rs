// Fase semantica de resolucion de nombres
// Debe responder unicamente estas preguntas:
// - Que simbolo corresponde a cada `Expr::Var`?
// - Donde fue declarado?
// - En que scope vive cada statement y expresion?
// - Hay errores de redeclaracion?
// - Hay variables no definidas?

mod error;
mod resolution_info;

pub(crate) use resolution_info::ResolutionInfo;

use crate::{
  ast::{Ast, AstVisitor, BlockId, Expr, ExprId, Stmt, StmtId, walk_block, walk_expr, walk_stmt},
  diagnostics::{Diagnosable, Diagnostic},
  semantic::{
    name_resolver::error::ResolverError, scope::ScopeArena, symbol::SymbolData,
    symbol_table::SymbolTable,
  },
};

#[derive(Debug, PartialEq)]
pub(crate) struct NameResolver<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// La SymbolTable. Solamente tiene sentido en el contexto del analisis semantico, por lo tanto
  /// no es una referencia y tomamos ownership.
  symbol_table: SymbolTable,
  /// Donde se van acumulando los errores encontrados durante el analisis de resolucion de nombres.
  diagnostics: Vec<Diagnostic>,
  /// Informacion sobre resolucion de nombres que se va acumulando.
  resolution_info: ResolutionInfo,
}

impl<'a> NameResolver<'a> {
  pub(crate) fn new(ast: &'a Ast) -> Self {
    let scopes = ScopeArena::new();
    let mut resolver = Self {
      ast,
      symbol_table: SymbolTable::new(scopes),
      diagnostics: Vec::new(),
      resolution_info: ResolutionInfo::new(),
    };
    resolver.symbol_table.enter_global_scope();
    resolver
  }

  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Devuelve la informacion de resolucion de nombres, consumiendo `self`.
  pub(crate) fn into_semantic_info(self) -> (ResolutionInfo, SymbolTable) {
    (self.resolution_info, self.symbol_table)
  }

  /// Resuelve el nombre de una expresion de identificador.
  fn resolve_var_expr(&mut self, expr_id: ExprId, name: String) {
    match self.symbol_table.resolve(&name) {
      Some(symbol_id) => self.resolution_info.insert_expr_symbol(expr_id, symbol_id),
      None => self.emit_error(&ResolverError::UndefinedVariable {
        name,
        span: self.ast.expr_span(expr_id),
      }),
    }
  }

  fn resolve_binding(&mut self, var: ExprId, stmt_id: StmtId) {
    // Confirmar que var sea Expr::Var
    let name = match self.ast.expr(var) {
      Expr::Var(name) => name,
      _ => return,
    };
    // Chequear redeclaracion en el scope actual
    let symbol = match self.symbol_table.declared_in_scope(&name) {
      Some(previous_symbol) => {
        let SymbolData {
          declaration_stmt, ..
        } = self
          .resolution_info
          .symbol_data_of(previous_symbol)
          .expect("debe existir una declaracion anterior");
        self.emit_error(&ResolverError::RedeclaredVariable {
          name,
          span: self.ast.stmt_span(stmt_id),
          previous_span: self.ast.stmt_span(declaration_stmt),
        });
        return;
      }
      // Insertar el simbolo en la tabla
      None => self.symbol_table.add_symbol(&name, self.ast.expr_span(var)),
    };
    self.resolution_info.insert_expr_symbol(var, symbol);
    self.resolution_info.insert_declared_symbol(stmt_id, symbol);
    let symbol_data = SymbolData {
      declaration_stmt: stmt_id,
    };
    self.resolution_info.insert_symbol_data(symbol, symbol_data);
  }

  fn resolve_assign(&mut self, var: ExprId) {
    let name = match self.ast.expr(var) {
      Expr::Var(name) => name,
      _ => return,
    };
    if let Some(symbol) = self.symbol_table.resolve(&name) {
      self.resolution_info.insert_expr_symbol(var, symbol)
    }
  }

  fn emit_error(&mut self, err: &ResolverError) {
    self.diagnostics.push(err.to_diagnostic());
  }
}

impl AstVisitor for NameResolver<'_> {
  /// Resuelve los nombres de variables para un bloque.
  fn visit_block(&mut self, block_id: BlockId) {
    // La idea es que mientras dure esta funcion, todo se va a analizar para el scope de este bloque
    let block_scope = self.symbol_table.enter_scope();
    self
      .resolution_info
      .insert_block_scope(block_id, block_scope);
    walk_block(self, self.ast, block_id);
    self.symbol_table.exit_scope();
  }

  /// Resuelve los nombres de variables para un statement.
  fn visit_stmt(&mut self, stmt_id: StmtId) {
    let scope = self
      .symbol_table
      .current_scope()
      .expect("todo statement debe tener scope");

    self.resolution_info.insert_stmt_scope(stmt_id, scope);

    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding { var, .. } | Stmt::ConstBinding { var, .. } => {
        self.resolve_binding(var, stmt_id)
      }
      Stmt::Assign { var, .. } => self.resolve_assign(var),
      _ => {}
    }
    walk_stmt(self, self.ast, stmt_id);
  }

  /// Resuelve los nombres de variables para una expresion.
  fn visit_expr(&mut self, expr_id: ExprId) {
    let scope = self
      .symbol_table
      .current_scope()
      .expect("toda expresion debe tener scope");
    self.resolution_info.insert_expr_scope(expr_id, scope);

    if let Expr::Var(name) = self.ast.expr(expr_id) {
      self.resolve_var_expr(expr_id, name);
    }

    walk_expr(self, self.ast, expr_id);
  }
}

#[cfg(test)]
use crate::{ast::Program, parser::parse_program};

#[cfg(test)]
pub(crate) fn resolve(
  source: &str,
) -> (ResolutionInfo, SymbolTable, Vec<Diagnostic>, Ast, Program) {
  let (ast, program) = parse_program(source);
  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let diagnostics = resolver.diagnostics().to_vec();
  let (resolution_info, symbol_table) = resolver.into_semantic_info();
  (resolution_info, symbol_table, diagnostics, ast, program)
}

#[cfg(test)]
pub(crate) mod tests;
