use rustc_hash::FxHashMap;

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryExpr, Expr, UnaryExpr},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    category_checker::{category::ExprCategory, error::CategoryError},
    compile_time_constant::compile_time_constant_checker::CompileTimeConstantInfo,
  },
};

pub type CategoryInfo = FxHashMap<ExprId, ExprCategory>;

#[derive(Debug)]
pub struct CategoryChecker<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// La informacion de que expresiones son constantes en tiempo de compilacion.
  compile_time_constant_info: &'a CompileTimeConstantInfo,
  /// Donde se van acumulando los errores encontrados durante el analisis de categorias.
  diagnostics: Vec<Diagnostic>,
  /// Informacion sobre categorias de las expresiones que se va acumulando.
  category_info: CategoryInfo,
}

impl<'a> CategoryChecker<'a> {
  pub fn new(ast: &'a Ast, compile_time_constant_info: &'a CompileTimeConstantInfo) -> Self {
    Self {
      ast,
      compile_time_constant_info,
      diagnostics: Vec::new(),
      category_info: FxHashMap::default(),
    }
  }

  pub fn check_program(&mut self, program: &Program) {
    self.check_expr(program.main_block_expr());
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn into_category_info(self) -> CategoryInfo {
    self.category_info
  }

  // ===================
  // Metodos internos
  // ===================

  fn check_block(&mut self, block_id: BlockId) {
    let block = self.ast.block(block_id);
    for stmt_id in block.stmts() {
      self.check_stmt(*stmt_id);
    }
  }

  fn check_stmt(&mut self, stmt_id: StmtId) {
    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding {
        var,
        initializer: value_expr,
      }
      | Stmt::Assign { var, value_expr } => self.check_assignment(var, value_expr),
      Stmt::ConstBinding { var, initializer } => self.check_const_binding(var, initializer),
      Stmt::Return(Some(expr_id)) | Stmt::Print(expr_id) | Stmt::Expr(expr_id) => {
        self.check_expr(expr_id);
      }
      Stmt::If {
        condition,
        if_block,
      } => {
        self.check_expr(condition);
        self.check_block(if_block);
      }
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      } => {
        self.check_expr(condition);
        self.check_block(if_block);
        self.check_block(else_block);
      }
      Stmt::Return(None) => {}
    }
  }

  fn check_assignment(&mut self, var: ExprId, value_expr: ExprId) {
    // Verificamos que el LHS sea una PlaceExpr, es decir un identificador de variable donde se pueda asignar algo
    let var_category = self.check_expr(var);
    if !var_category.is_place() {
      self.emit_error(&CategoryError::ExpectedPlaceExpression {
        span: self.ast.expr_span(var),
      });
    }
    // Verificamos que el RHS sea una ValueExpr, es decir una expresion que evalue a un valor
    let value_expr_category = self.check_expr(value_expr);
    if !value_expr_category.is_value() {
      self.emit_error(&CategoryError::ExpectedValueExpression {
        span: self.ast.expr_span(value_expr),
      });
    }
  }

  fn check_const_binding(&mut self, var: ExprId, initializer: ExprId) {
    let var_category = self.check_expr(var);
    if !var_category.is_place() {
      self.emit_error(&CategoryError::ExpectedPlaceExpression {
        span: self.ast.expr_span(var),
      });
    }
    let init_category = self.check_expr(initializer);
    if !init_category.is_value() {
      self.emit_error(&CategoryError::ExpectedValueExpression {
        span: self.ast.expr_span(initializer),
      });
    }
    if !init_category.is_constant() {
      self.emit_error(&CategoryError::ExpectedConstantExpression {
        span: self.ast.expr_span(initializer),
      });
    }
  }

  /// Resuelve la categoria de la expresion indicada. Devuelve la categoria.
  /// Invariante: Si check_expr fue llamado, esa expresion tiene la categoria guardada.
  fn check_expr(&mut self, expr_id: ExprId) -> ExprCategory {
    let expr = self.ast.expr(expr_id);
    match expr {
      Expr::Unary(UnaryExpr { op: _, operand }) => {
        self.check_expr(operand);
      }
      Expr::Binary(BinaryExpr { op: _, lhs, rhs }) => {
        self.check_expr(lhs);
        self.check_expr(rhs);
      }
      Expr::Block(block_id) => self.check_block(block_id),
      _ => {}
    }
    let categories = match expr {
      Expr::Const(_) => ExprCategory::constant().with(ExprCategory::value()),
      Expr::Var(_) => {
        ExprCategory::value().with(ExprCategory::place()) // todo()
      },
      Expr::Unary(_) | Expr::Binary(_) => {
        let value_category = ExprCategory::value();
        if self.compile_time_constant_info.contains_key(&expr_id) {
          value_category.with(ExprCategory::constant())
        } else {
          value_category
        }
      }
      Expr::Block(_) => self.check_block_expr(expr_id),
    };
    self.category_info.insert(expr_id, categories);
    categories
  }

  /// Un bloque debe ser ValueExpr, no ser PlaceExpr, y ser ConstantExpr
  /// si y solo si tiene una constante en compilacion segun CompileTimeConstantInfo
  fn check_block_expr(&mut self, expr_id: ExprId) -> ExprCategory {
    if self.compile_time_constant_info.get(&expr_id).is_some() {
      ExprCategory::value().with(ExprCategory::constant())
    } else {
      ExprCategory::value()
    }
  }

  fn emit_error(&mut self, err: &CategoryError) {
    self.diagnostics.push(err.to_diagnostic());
  }
}

#[cfg(test)]
pub mod tests;
