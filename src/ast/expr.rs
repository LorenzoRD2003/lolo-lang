// Los tipos de este archivo deben ser publicos, ya que los vamos a usar desde el parser / lowering / IR

use crate::{
  ast::ast::ExprId,
  lexer::token::{Token, TokenKind},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Var(VarId),
  Const(ConstValue),
  Unary(UnaryExpr),
  Binary(BinaryExpr),
}

impl Expr {
  pub(crate) fn is_comparison(&self) -> bool {
    matches!(
      self,
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Eq
          | BinaryOp::Neq
          | BinaryOp::Gt
          | BinaryOp::Lt
          | BinaryOp::Gte
          | BinaryOp::Lte,
        ..
      })
    )
  }
}

pub type VarId = String; // Identificador de cada variable

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
  Int(i32),
  Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
  pub(crate) op: UnaryOp,
  pub(crate) operand: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
  Neg,
  Not,
}

impl UnaryOp {
  pub(crate) fn from_token(token: &Token) -> Option<Self> {
    match token.kind {
      TokenKind::Bang => Some(Self::Not),
      TokenKind::Minus => Some(Self::Neg),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BinaryExpr {
  pub(crate) op: BinaryOp,
  pub(crate) lhs: ExprId,
  pub(crate) rhs: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BinaryOp {
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

impl BinaryOp {
  pub(crate) fn from_token(token: &Token) -> Option<Self> {
    match token.kind {
      TokenKind::Plus => Some(Self::Add),
      TokenKind::Minus => Some(Self::Sub),
      TokenKind::Star => Some(Self::Mul),
      TokenKind::Slash => Some(Self::Div),
      TokenKind::EqualEqual => Some(Self::Eq),
      TokenKind::BangEqual => Some(Self::Neq),
      TokenKind::Greater => Some(Self::Gt),
      TokenKind::Less => Some(Self::Lt),
      TokenKind::GreaterEqual => Some(Self::Gte),
      TokenKind::LessEqual => Some(Self::Lte),
      TokenKind::AndAnd => Some(Self::And),
      TokenKind::OrOr => Some(Self::Or),
      TokenKind::CaretCaret => Some(Self::Xor),
      _ => None,
    }
  }
}
