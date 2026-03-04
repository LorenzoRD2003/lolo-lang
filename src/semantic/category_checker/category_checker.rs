use rustc_hash::FxHashMap;

use crate::{
  ast::{Ast, AstVisitor, BlockId, Expr, ExprId, Stmt, StmtId, walk_block, walk_expr, walk_stmt},
  diagnostics::{Diagnosable, Diagnostic},
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

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn into_category_info(self) -> CategoryInfo {
    self.category_info
  }

  fn check_assignment(&mut self, var: ExprId, rhs: ExprId) {
    // Verificamos que el LHS sea una PlaceExpr, es decir un identificador de variable donde se pueda asignar algo
    let var_cat = self.category_info.get(&var).expect("debe tener categoria");
    if !var_cat.is_place() {
      self.emit_error(&CategoryError::ExpectedPlaceExpression {
        span: self.ast.expr_span(var),
      });
    }
    // Verificamos que el RHS sea una ValueExpr, es decir una expresion que evalue a un valor
    let rhs_cat = self.category_info.get(&rhs).expect("debe tener categoria");
    if !rhs_cat.is_value() {
      self.emit_error(&CategoryError::ExpectedValueExpression {
        span: self.ast.expr_span(rhs),
      });
    }
  }

  fn check_const_binding(&mut self, var: ExprId, rhs: ExprId) {
    let var_cat = self.category_info.get(&var).expect("debe tener categoria");
    if !var_cat.is_place() {
      self.emit_error(&CategoryError::ExpectedPlaceExpression {
        span: self.ast.expr_span(var),
      });
    }
    let rhs_cat = *self.category_info.get(&rhs).expect("debe tener categoria");
    if !rhs_cat.is_value() {
      self.emit_error(&CategoryError::ExpectedValueExpression {
        span: self.ast.expr_span(rhs),
      });
    }
    if !rhs_cat.is_constant() {
      self.emit_error(&CategoryError::ExpectedConstantExpression {
        span: self.ast.expr_span(rhs),
      });
    }
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

impl AstVisitor for CategoryChecker<'_> {
  /// Resuelve las categorias para el bloque indicado.
  fn visit_block(&mut self, block_id: BlockId) {
    walk_block(self, self.ast, block_id);
  }

  /// Resuelve las categorias para el statement indicado.
  fn visit_stmt(&mut self, stmt_id: StmtId) {
    walk_stmt(self, self.ast, stmt_id);

    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding {
        var,
        initializer: value_expr,
      }
      | Stmt::Assign { var, value_expr } => self.check_assignment(var, value_expr),
      Stmt::ConstBinding { var, initializer } => self.check_const_binding(var, initializer),
      _ => {}
    }
  }

  /// Resuelve la categoria de la expresion indicada.
  /// Invariante: Si check_expr fue llamado, esa expresion tiene la categoria guardada.
  fn visit_expr(&mut self, expr_id: ExprId) {
    walk_expr(self, self.ast, expr_id);

    let cat = match self.ast.expr(expr_id) {
      Expr::Const(_) => ExprCategory::constant().with(ExprCategory::value()),
      Expr::Var(_) => {
        ExprCategory::value().with(ExprCategory::place()) // todo()
      }
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
    self.category_info.insert(expr_id, cat);
  }
}

#[cfg(test)]
pub mod tests;
