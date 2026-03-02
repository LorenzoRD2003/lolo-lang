// Los tipos de este archivo deben ser publicos, ya que los vamos a usar desde el parser / lowering / IR

use std::fmt::Display;

use crate::{
  ast::ast::ExprId,
  lexer::token::{Token, TokenKind},
  semantic::types::Type,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Var(VarId),
  Const(ConstValue),
  Unary(UnaryExpr),
  Binary(BinaryExpr),
}

impl Expr {
  pub fn is_var(&self) -> bool {
    matches!(self, Expr::Var(_))
  }

  pub fn is_comparison(&self) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarId(pub String); // Identificador de cada variable

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
  Int32(i32),
  Bool(bool),
}

impl Display for ConstValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Int32(x) => write!(f, "{}", x),
      Self::Bool(b) => write!(f, "{}", b),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
  pub op: UnaryOp,
  pub operand: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
  Neg,
  Not,
}

impl UnaryOp {
  pub fn from_token(token: &Token) -> Option<Self> {
    match token.kind() {
      TokenKind::Bang => Some(Self::Not),
      TokenKind::Minus => Some(Self::Neg),
      _ => None,
    }
  }

  pub fn is_valid_for_operand_type(&self, operand_type: Type) -> bool {
    match self {
      Self::Neg => operand_type.is_number(),
      Self::Not => operand_type.is_boolean(),
    }
  }

  /// Por ahora esta funcion no depende de los operandos.
  /// En un futuro eso podria cambiar, e ir directamente en UnaryExpr
  pub fn result_type(&self) -> Type {
    match self {
      Self::Neg => Type::Int32,
      Self::Not => Type::Bool,
    }
  }

  fn to_string(&self) -> &str {
    match self {
      Self::Neg => "-",
      Self::Not => "!",
    }
  }
}

impl Display for UnaryOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
  pub op: BinaryOp,
  pub lhs: ExprId,
  pub rhs: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl BinaryOp {
  pub fn from_token(token: &Token) -> Option<Self> {
    match token.kind() {
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

  pub fn is_valid_for_operand_types(&self, lhs_type: Type, rhs_type: Type) -> bool {
    match self {
      Self::Add
      | Self::Sub
      | Self::Mul
      | Self::Div
      | Self::Gt
      | Self::Lt
      | Self::Gte
      | Self::Lte => lhs_type.is_number() && rhs_type.is_number(),
      Self::Eq | Self::Neq => {
        (lhs_type.is_number() && rhs_type.is_number())
          || (lhs_type.is_boolean() && rhs_type.is_boolean())
      }
      Self::And | Self::Or | Self::Xor => lhs_type.is_boolean() && rhs_type.is_boolean(),
    }
  }

  /// Por ahora esta funcion no depende de los operandos.
  /// En un futuro eso podria cambiar, e ir directamente en BinaryExpr
  pub fn result_type(&self) -> Type {
    match self {
      Self::Add | Self::Sub | Self::Mul | Self::Div => Type::Int32,
      Self::Eq
      | Self::Neq
      | Self::Gt
      | Self::Lt
      | Self::Gte
      | Self::Lte
      | Self::And
      | Self::Or
      | Self::Xor => Type::Bool,
    }
  }

  pub fn to_string(&self) -> &str {
    match self {
      Self::Add => "+",
      Self::Sub => "-",
      Self::Mul => "*",
      Self::Div => "/",
      Self::Eq => "==",
      Self::Neq => "!=",
      Self::Gt => ">",
      Self::Lt => "<",
      Self::Gte => ">=",
      Self::Lte => "<=",
      Self::And => "&&",
      Self::Or => "||",
      Self::Xor => "^^",
    }
  }
}

impl Display for BinaryOp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}
