mod error;

use rustc_hash::FxHashMap;

use crate::{
  ast::{
    Ast, AstVisitor, BinaryExpr, BinaryOp, BlockId, ConstValue, Expr, ExprId, Stmt, StmtId,
    UnaryExpr, UnaryOp, walk_block, walk_expr, walk_stmt,
  },
  diagnostics::{Diagnosable, Diagnostic},
  semantic::{
    compile_time_constant_checker::error::CompileTimeConstantError, name_resolver::ResolutionInfo,
    symbol::SymbolId,
  },
};

pub(crate) type CompileTimeConstantInfo = FxHashMap<ExprId, ConstValue>;

#[derive(Debug)]
pub(crate) struct CompileTimeConstantChecker<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// Informacion de resolucion de nombres, recibida al consumir el NameResolver.
  resolution_info: &'a ResolutionInfo,
  /// Donde se van acumulando los errores encontrados durante el analisis.
  diagnostics: Vec<Diagnostic>,
  /// Tabla donde guardo valores constantes de bindings.
  const_bindings: FxHashMap<SymbolId, ConstValue>,
  /// Informacion sobre compile time constants que se va acumulando.
  compile_time_constant_info: CompileTimeConstantInfo,
}

impl<'a> CompileTimeConstantChecker<'a> {
  pub(crate) fn new(ast: &'a Ast, resolution_info: &'a ResolutionInfo) -> Self {
    Self {
      ast,
      resolution_info,
      diagnostics: Vec::new(),
      const_bindings: FxHashMap::default(),
      compile_time_constant_info: FxHashMap::default(),
    }
  }

  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub(crate) fn into_compile_time_constant_info(self) -> CompileTimeConstantInfo {
    self.compile_time_constant_info
  }

  fn emit_error(&mut self, err: &CompileTimeConstantError) {
    self.diagnostics.push(err.to_diagnostic());
  }

  fn block_const_value(&self, block_id: crate::ast::BlockId) -> Option<ConstValue> {
    let block = self.ast.block(block_id);
    block.terminator().and_then(|stmt| {
      if let Stmt::Return(Some(expr)) = self.ast.stmt(stmt) {
        self.compile_time_constant_info.get(&expr).cloned()
      } else {
        None
      }
    })
  }
}

impl AstVisitor for CompileTimeConstantChecker<'_> {
  /// Resuelve las constantes en tiempo de compilacion para el bloque indicado.
  fn visit_block(&mut self, block_id: BlockId) {
    walk_block(self, self.ast, block_id);
  }

  /// Resuelve las constantes en tiempo de compilacion para el statement indicado.
  fn visit_stmt(&mut self, stmt_id: StmtId) {
    walk_stmt(self, self.ast, stmt_id);

    // Asocio un const binding al simbolo de la variable, si el initializer es constante
    if let Stmt::ConstBinding { var, initializer } = self.ast.stmt(stmt_id)
      && let Some(value) = self.compile_time_constant_info.get(&initializer)
      && let Some(symbol_id) = self.resolution_info.symbol_of(var)
    {
      self.const_bindings.insert(symbol_id, value.clone());
    }
  }

  /// Resuelve las constantes en tiempo de compilacion para la expresion indicada.
  fn visit_expr(&mut self, expr_id: ExprId) {
    walk_expr(self, self.ast, expr_id);

    let ctc_value = match self.ast.expr(expr_id) {
      Expr::Const(v) => Some(v),
      Expr::Var(_) => self
        .resolution_info
        .symbol_of(expr_id)
        .and_then(|symbol_id| self.const_bindings.get(&symbol_id).cloned()),
      Expr::Block(block_id) => self.block_const_value(block_id),
      Expr::If(if_expr) => {
        let condition_value = self.compile_time_constant_info.get(&if_expr.condition);
        match condition_value {
          Some(ConstValue::Bool(true)) => self.block_const_value(if_expr.if_block),
          Some(ConstValue::Bool(false)) => if_expr
            .else_branch
            .and_then(|expr| self.compile_time_constant_info.get(&expr).cloned()),
          _ => None,
        }
      }
      // Todos los operadores que tenemos son puros asi que esto está bien
      Expr::Unary(UnaryExpr { op, operand }) => {
        let operand_value = self.compile_time_constant_info.get(&operand).cloned();
        match (operand_value, op) {
          (Some(ConstValue::Int32(x)), UnaryOp::Neg) => Some(ConstValue::Int32(-x)),
          (Some(ConstValue::Bool(b)), UnaryOp::Not) => Some(ConstValue::Bool(!b)),
          _ => None, // recordar que el type mismatch ya fue analizado antes
        }
      }
      Expr::Binary(BinaryExpr { op, lhs, rhs }) => {
        let lvalue = self.compile_time_constant_info.get(&lhs).cloned();
        let rvalue = self.compile_time_constant_info.get(&rhs).cloned();
        match (lvalue, op, rvalue) {
          (Some(ConstValue::Int32(x)), BinaryOp::Add, Some(ConstValue::Int32(y))) => {
            match x.checked_add(y) {
              Some(res) => Some(ConstValue::Int32(res)),
              None => {
                self.emit_error(&CompileTimeConstantError::ArithmeticOverflow {
                  op: BinaryOp::Add,
                  lhs: ConstValue::Int32(x),
                  rhs: ConstValue::Int32(y),
                  span: self.ast.expr_span(expr_id),
                });
                None
              }
            }
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Sub, Some(ConstValue::Int32(y))) => {
            match x.checked_sub(y) {
              Some(res) => Some(ConstValue::Int32(res)),
              None => {
                self.emit_error(&CompileTimeConstantError::ArithmeticOverflow {
                  op: BinaryOp::Sub,
                  lhs: ConstValue::Int32(x),
                  rhs: ConstValue::Int32(y),
                  span: self.ast.expr_span(expr_id),
                });
                None
              }
            }
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Mul, Some(ConstValue::Int32(y))) => {
            match x.checked_mul(y) {
              Some(res) => Some(ConstValue::Int32(res)),
              None => {
                self.emit_error(&CompileTimeConstantError::ArithmeticOverflow {
                  op: BinaryOp::Mul,
                  lhs: ConstValue::Int32(x),
                  rhs: ConstValue::Int32(y),
                  span: self.ast.expr_span(expr_id),
                });
                None
              }
            }
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Div, Some(ConstValue::Int32(y))) => {
            if y == 0 {
              let rhs_span = self.ast.expr_span(rhs);
              self.emit_error(&CompileTimeConstantError::ZeroDivision { span: rhs_span });
              None
            } else {
              Some(ConstValue::Int32(x / y))
            }
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Eq, Some(ConstValue::Int32(y))) => {
            Some(ConstValue::Bool(x == y))
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Neq, Some(ConstValue::Int32(y))) => {
            Some(ConstValue::Bool(x != y))
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Gt, Some(ConstValue::Int32(y))) => {
            Some(ConstValue::Bool(x > y))
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Lt, Some(ConstValue::Int32(y))) => {
            Some(ConstValue::Bool(x < y))
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Gte, Some(ConstValue::Int32(y))) => {
            Some(ConstValue::Bool(x >= y))
          }
          (Some(ConstValue::Int32(x)), BinaryOp::Lte, Some(ConstValue::Int32(y))) => {
            Some(ConstValue::Bool(x <= y))
          }
          (Some(ConstValue::Bool(p)), BinaryOp::And, Some(ConstValue::Bool(q))) => {
            Some(ConstValue::Bool(p && q))
          }
          (Some(ConstValue::Bool(p)), BinaryOp::Or, Some(ConstValue::Bool(q))) => {
            Some(ConstValue::Bool(p || q))
          }
          (Some(ConstValue::Bool(p)), BinaryOp::Xor, Some(ConstValue::Bool(q))) => {
            Some(ConstValue::Bool(p ^ q))
          }
          _ => None,
        }
      }
    };
    ctc_value.and_then(|const_value| self.compile_time_constant_info.insert(expr_id, const_value));
  }
}

#[cfg(test)]
pub(crate) mod tests;
