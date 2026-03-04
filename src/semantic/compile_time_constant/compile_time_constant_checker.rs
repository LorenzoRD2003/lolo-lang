use rustc_hash::FxHashMap;

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, UnaryExpr, UnaryOp},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    compile_time_constant::error::CompileTimeConstantError, id_generator::SymbolId,
    resolver::resolution_info::ResolutionInfo,
  },
};

pub type CompileTimeConstantInfo = FxHashMap<ExprId, ConstValue>;

#[derive(Debug)]
pub struct CompileTimeConstantChecker<'a> {
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
  pub fn new(ast: &'a Ast, resolution_info: &'a ResolutionInfo) -> Self {
    Self {
      ast,
      resolution_info,
      diagnostics: Vec::new(),
      const_bindings: FxHashMap::default(),
      compile_time_constant_info: FxHashMap::default(),
    }
  }

  pub fn check_program(&mut self, program: &Program) {
    self.check_expr(program.main_block_expr());
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn into_compile_time_constant_info(self) -> CompileTimeConstantInfo {
    self.compile_time_constant_info
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

  fn check_block_expr(&mut self, block_id: BlockId) -> Option<ConstValue> {
    self.check_block(block_id);
    let block = self.ast.block(block_id);
    if let Stmt::Return(Some(expr)) = self.ast.stmt(block.terminator()?) {
      self.compile_time_constant_info.get(&expr).cloned()
    } else {
      None
    }
  }

  fn check_stmt(&mut self, stmt_id: StmtId) {
    match self.ast.stmt(stmt_id) {
      Stmt::ConstBinding { var, initializer } => {
        // Asocio un const binding al simbolo de la variable, si el initializer es constante
        if let Some(value) = self.check_expr(initializer)
          && let Some(symbol_id) = self.resolution_info.symbol_of(var)
        {
          self.const_bindings.insert(symbol_id, value);
        }
      }
      Stmt::LetBinding {
        var: _,
        initializer: expr_id,
      }
      | Stmt::Assign {
        var: _,
        value_expr: expr_id,
      }
      | Stmt::Return(Some(expr_id))
      | Stmt::Print(expr_id)
      | Stmt::Expr(expr_id) => {
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

  /// Resuelve si la expresion indicada es una constante en tiempo de compilacion. La devuelve.
  fn check_expr(&mut self, expr_id: ExprId) -> Option<ConstValue> {
    if let Some(const_value) = self.compile_time_constant_info.get(&expr_id) {
      return Some(const_value.clone());
    }

    let expr = self.ast.expr(expr_id);
    let compile_time_constant = match expr {
      Expr::Const(v) => Some(v),
      Expr::Var(_) => {
        let symbol_id = self.resolution_info.symbol_of(expr_id)?;
        self.const_bindings.get(&symbol_id).cloned()
      }
      Expr::Block(block_id) => self.check_block_expr(block_id),
      // Todos los operadores que tenemos son puros asi que esto está bien
      Expr::Unary(UnaryExpr { op, operand }) => {
        let operand_value = self.check_expr(operand);
        match (operand_value, op) {
          (Some(ConstValue::Int32(x)), UnaryOp::Neg) => Some(ConstValue::Int32(-x)),
          (Some(ConstValue::Bool(b)), UnaryOp::Not) => Some(ConstValue::Bool(!b)),
          _ => None, // recordar que el type mismatch ya fue analizado antes
        }
      }
      Expr::Binary(BinaryExpr { op, lhs, rhs }) => {
        let lvalue = self.check_expr(lhs);
        let rvalue = self.check_expr(rhs);
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
    if let Some(const_value) = compile_time_constant.clone() {
      self.compile_time_constant_info.insert(expr_id, const_value);
    }
    compile_time_constant
  }

  fn emit_error(&mut self, err: &CompileTimeConstantError) {
    self.diagnostics.push(err.to_diagnostic());
  }
}

#[cfg(test)]
pub mod tests;
