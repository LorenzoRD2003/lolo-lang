// El TypeChecker es la fase siguiente al resolver. Debe saber responder:
// - que tipo tiene cada expresion.
// - si son compatibles los tipos en operaciones.
// - si las asignaciones respetan el tipo.
// - si los inicializadores coinciden con el tipo declarado.
// - si las condiciones son booleanas.
// - si, en general, hay errores de tipo.
// Finalmente, debe producir `TypeInfo` y diagnosticos de errores de tipo
// No usa la tabla de simbolos, pero si la ResolutionInfo.

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryExpr, Expr, UnaryExpr},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    resolver::resolution_info::ResolutionInfo,
    type_checker::{error::TypeError, type_info::TypeInfo},
    types::Type,
  },
};

/// Responsabilidades: Recorrer el AST y:
/// - Inferir tipos
/// - Chequear consistencia
/// - Emitir errores
/// - Guardar tipo de cada expresión
#[derive(Debug, Clone)]
pub struct TypeChecker<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// Informacion de resolucion de nombres, recibida al consumir el NameResolver.
  resolution_info: &'a ResolutionInfo,
  /// Donde se van acumulando los errores encontrados durante el analisis de tipos.
  diagnostics: Vec<Diagnostic>,
  /// Informacion sobre tipos que se va acumulando.
  type_info: TypeInfo,
}

impl<'a> TypeChecker<'a> {
  pub fn new(ast: &'a Ast, resolution_info: &'a ResolutionInfo) -> Self {
    Self {
      ast,
      resolution_info,
      diagnostics: Vec::new(),
      type_info: TypeInfo::new(),
    }
  }

  /// Resuelve los tipos para el programa, guardando la informacion en type_info.
  pub fn check_program(&mut self, program: &Program) {
    self.check_block(program.main_block());
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Devuelve la informacion de resolucion de tipos, consumiendo `self`.
  pub fn into_type_info(self) -> TypeInfo {
    self.type_info
  }

  // ===================
  // Metodos internos
  // ===================

  /// Resuelve los tipos para el bloque indicado.
  fn check_block(&mut self, block_id: BlockId) {
    let block = self.ast.block(block_id);
    for stmt_id in block.stmts() {
      self.check_stmt(*stmt_id);
    }
  }

  /// Resuelve los tipos para el statement indicado.
  fn check_stmt(&mut self, stmt_id: StmtId) {
    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding { var, initializer } => {
        let initializer_type = self.check_expr(initializer);
        let symbol = self
          .resolution_info
          .symbol_of(var)
          .expect("la variable debe tener un simbolo");
        self.type_info.set_symbol_type(symbol, initializer_type);
      }
      Stmt::Assign { var, value_expr } => {
        let value_expr_type = self.check_expr(value_expr);
        if let Some(symbol) = self.resolution_info.symbol_of(var)
          && let Some(symbol_type) = self.type_info.type_of_symbol(symbol)
        {
          if value_expr_type != symbol_type {
            self.emit_error(&TypeError::MismatchedTypes {
              expected: symbol_type,
              found: value_expr_type,
              span: self.ast.stmt_span(stmt_id),
            });
          }
        }
      }
      Stmt::Expr(expr_id) | Stmt::Return(expr_id) | Stmt::Print(expr_id) => {
        self.check_expr(expr_id);
      }
      Stmt::If {
        condition,
        if_block,
      } => {
        self.check_condition(condition);
        self.check_block(if_block);
      }
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      } => {
        self.check_condition(condition);
        self.check_block(if_block);
        self.check_block(else_block);
      }
    }
  }

  /// Chequea el tipo de una condicion.
  fn check_condition(&mut self, condition: ExprId) {
    let condition_type = self.check_expr(condition);
    if !condition_type.is_boolean() {
      self.emit_error(&TypeError::NonBooleanCondition {
        found: condition_type,
        span: self.ast.expr_span(condition),
      });
    }
  }

  /// Resuelve el tipo de la expresion indicada. Devuelve el tipo inferido.
  /// Invariante: Si check_expr fue llamado, esa expresion tiene el tipo guardado.
  fn check_expr(&mut self, expr_id: ExprId) -> Type {
    let expr = self.ast.expr(expr_id);
    let ty = match expr {
      Expr::Var(_) => {
        let symbol = self
          .resolution_info
          .symbol_of(expr_id)
          .expect("la variable debe tener un simbolo");
        match self.type_info.type_of_symbol(symbol) {
          Some(t) => t,
          None => Type::DefaultErrorType,
        }
      }
      Expr::Const(const_value) => const_value.into(),
      Expr::Unary(UnaryExpr { op, operand }) => {
        let operand_type = self.check_expr(operand);
        if !op.is_valid_for_operand_type(operand_type) {
          self.emit_error(&TypeError::InvalidUnaryOperation {
            op,
            operand: operand_type,
            span: self.ast.expr_span(expr_id),
          });
        }
        op.result_type()
      }
      Expr::Binary(BinaryExpr { op, lhs, rhs }) => {
        let lhs_type = self.check_expr(lhs);
        let rhs_type = self.check_expr(rhs);
        if !op.is_valid_for_operand_types(lhs_type, rhs_type) {
          self.emit_error(&TypeError::InvalidBinaryOperation {
            op,
            lhs: lhs_type,
            rhs: rhs_type,
            span: self.ast.expr_span(expr_id),
          });
        }
        op.result_type()
      }
    };
    self.type_info.insert_expr_type(expr_id, ty);
    ty
  }

  fn emit_error(&mut self, err: &TypeError) {
    self.diagnostics.push(err.to_diagnostic());
  }
}

#[cfg(test)]
pub mod tests;
