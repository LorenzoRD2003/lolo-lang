use crate::ast::span::{Span, Spanned};

// Los tipos de este archivo deben ser publicos, ya que los vamos a usar desde el parser / lowering / IR

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
  operand: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
  Neg,
  Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
  op: BinaryOp,
  lhs: Box<Expr>,
  rhs: Box<Expr>,
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

impl Spanned for Expr {
  fn span(&self) -> Span {
    match self {
      Expr::Var(var_id) => var_id.span(),
      Expr::Const(const_value) => const_value.span(),
      Expr::Unary(unary_expr) => unary_expr.span(),
      Expr::Binary(binary_expr) => binary_expr.span(),
    }
  }
}

impl Spanned for VarId {
  fn span(&self) -> Span {
    todo!()
  }
}

impl Spanned for ConstValue {
  fn span(&self) -> Span {
    todo!()
  }
}

impl Spanned for UnaryExpr {
  fn span(&self) -> Span {
    todo!()
  }
}

impl Spanned for BinaryExpr {
  fn span(&self) -> Span {
    todo!()
  }
}
