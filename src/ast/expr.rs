// Los tipos de este archivo deben ser publicos, ya que los vamos a usar desde el parser / lowering / IR

use crate::ast::ast::ExprId;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Var(VarId),
  Const(ConstValue),
  Unary(UnaryExpr),
  Binary(BinaryExpr),
}

pub type VarId = String; // Identificador de cada variable

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
  Int(i32),
  Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
  op: UnaryOp,
  operand: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
  Neg,
  Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
  op: BinaryOp,
  lhs: ExprId,
  rhs: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
  // Arithmetic Binary Operations
  Add,
  Sub,
  Mul,
  Div,
  // Comparison Binary Operations
  Eq,
  Neq,
  Gt,
  Lt,
  Gte,
  Lte,
  // Logical Binary Operations
  And,
  Or,
  Xor,
}
