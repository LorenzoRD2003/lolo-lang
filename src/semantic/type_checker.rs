// El TypeChecker es la fase siguiente al resolver. Debe saber responder:
// - que tipo tiene cada expresion.
// - si son compatibles los tipos en operaciones.
// - si las asignaciones respetan el tipo.
// - si los inicializadores coinciden con el tipo declarado.
// - si las condiciones son booleanas.
// - si, en general, hay errores de tipo.
// Finalmente, debe producir `TypeInfo` y diagnosticos de errores de tipo
// No usa la tabla de simbolos, pero si la ResolutionInfo.

mod error;
mod type_info;

pub(crate) use type_info::TypeInfo;

use crate::{
  ast::{
    Ast, AstVisitor, BinaryExpr, BlockId, Expr, ExprId, IfExpr, Stmt, StmtId, UnaryExpr,
    walk_block, walk_expr, walk_stmt,
  },
  diagnostics::{Diagnosable, Diagnostic},
  semantic::{name_resolver::ResolutionInfo, type_checker::error::TypeError, types::Type},
};

/// Responsabilidades: Recorrer el AST y:
/// - Inferir tipos
/// - Chequear consistencia
/// - Emitir errores
/// - Guardar tipo de cada expresión
#[derive(Debug)]
pub(crate) struct TypeChecker<'a> {
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
  pub(crate) fn new(ast: &'a Ast, resolution_info: &'a ResolutionInfo) -> Self {
    Self {
      ast,
      resolution_info,
      diagnostics: Vec::new(),
      type_info: TypeInfo::new(),
    }
  }

  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Devuelve la informacion de resolucion de tipos, consumiendo `self`.
  pub(crate) fn into_type_info(self) -> TypeInfo {
    self.type_info
  }

  /// Chequea el tipo de una condicion.
  fn check_condition(&mut self, condition: ExprId) {
    let condition_type = self.type_info.type_of_expr(condition);
    if !condition_type.is_boolean() {
      self.emit_error(&TypeError::NonBooleanCondition {
        found: condition_type,
        span: self.ast.expr_span(condition),
      });
    }
  }

  fn emit_error(&mut self, err: &TypeError) {
    self.diagnostics.push(err.to_diagnostic());
  }

  fn block_type(&self, block_id: BlockId) -> Type {
    let block = self.ast.block(block_id);
    match block.terminator() {
      Some(stmt_id) => match self.ast.stmt(stmt_id) {
        Stmt::Return(Some(expr_id)) => self.type_info.type_of_expr(*expr_id),
        Stmt::Return(None) => Type::Unit,
        _ => unreachable!("el terminador de bloque debe ser un Return"),
      },
      None => Type::Unit,
    }
  }

  fn infer_if_expr_type(&mut self, expr_id: ExprId, if_expr: &IfExpr) -> Type {
    self.check_condition(if_expr.condition);
    let if_block_ty = self.block_type(if_expr.if_block);
    if let Some(else_branch_expr) = if_expr.else_branch {
      let else_ty = self.type_info.type_of_expr(else_branch_expr);
      if if_block_ty == Type::DefaultErrorType || else_ty == Type::DefaultErrorType {
        return Type::DefaultErrorType;
      }
      if if_block_ty != else_ty {
        self.emit_error(&TypeError::MismatchedTypes {
          expected: if_block_ty,
          found: else_ty,
          span: self.ast.expr_span(expr_id),
        });
        return Type::DefaultErrorType;
      }
      return if_block_ty;
    }
    Type::Unit
  }
}

impl AstVisitor for TypeChecker<'_> {
  /// Resuelve los tipos para el bloque indicado.
  fn visit_block(&mut self, block_id: BlockId) {
    walk_block(self, self.ast, block_id);
  }

  /// Resuelve los tipos para el statement indicado.
  fn visit_stmt(&mut self, stmt_id: StmtId) {
    walk_stmt(self, self.ast, stmt_id);
    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding { var, initializer } | Stmt::ConstBinding { var, initializer } => {
        let initializer_type = self.type_info.type_of_expr(*initializer);
        if let Some(symbol) = self.resolution_info.symbol_of(*var) {
          self.type_info.set_symbol_type(symbol, initializer_type);
        }
      }
      Stmt::Assign { var, value_expr } => {
        let value_expr_type = self.type_info.type_of_expr(*value_expr);
        if let Some(symbol) = self.resolution_info.symbol_of(*var)
          && let Some(symbol_type) = self.type_info.type_of_symbol(symbol)
          && value_expr_type != symbol_type
        {
          self.emit_error(&TypeError::MismatchedTypes {
            expected: symbol_type,
            found: value_expr_type,
            span: self.ast.stmt_span(stmt_id),
          });
        }
      }
      _ => {}
    }
  }

  /// Resuelve el tipo de la expresion indicada. Devuelve el tipo inferido.
  /// Invariante: Si check_expr fue llamado, esa expresion tiene el tipo guardado.
  fn visit_expr(&mut self, expr_id: ExprId) {
    walk_expr(self, self.ast, expr_id);

    let ty = match self.ast.expr(expr_id) {
      Expr::Const(const_value) => const_value.clone().into(),
      Expr::Var(_) => {
        if let Some(symbol) = self.resolution_info.symbol_of(expr_id) {
          match self.type_info.type_of_symbol(symbol) {
            Some(t) => t,
            None => Type::DefaultErrorType,
          }
        } else {
          Type::DefaultErrorType
        }
      }
      Expr::Unary(UnaryExpr { op, operand }) => {
        let operand_type = self.type_info.type_of_expr(*operand);
        if op.is_valid_for_operand_type(operand_type) {
          op.result_type()
        } else {
          self.emit_error(&TypeError::InvalidUnaryOperation {
            op: *op,
            operand: operand_type,
            span: self.ast.expr_span(expr_id),
          });
          Type::DefaultErrorType
        }
      }
      Expr::Binary(BinaryExpr { op, lhs, rhs }) => {
        let lhs_type = self.type_info.type_of_expr(*lhs);
        let rhs_type = self.type_info.type_of_expr(*rhs);
        if op.is_valid_for_operand_types(lhs_type, rhs_type) {
          op.result_type()
        } else {
          self.emit_error(&TypeError::InvalidBinaryOperation {
            op: *op,
            lhs: lhs_type,
            rhs: rhs_type,
            span: self.ast.expr_span(expr_id),
          });
          Type::DefaultErrorType
        }
      }
      Expr::Block(block_id) => self.block_type(*block_id),
      Expr::If(if_expr) => self.infer_if_expr_type(expr_id, &if_expr),
    };
    self.type_info.insert_expr_type(expr_id, ty);
  }
}

#[cfg(test)]
pub(crate) mod tests;
