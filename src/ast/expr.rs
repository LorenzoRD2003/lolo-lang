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
  pub(crate) fn is_var(&self) -> bool {
    matches!(self, Expr::Var(_))
  }

  pub(crate) fn get_var_name(&self) -> Option<VarId> {
    match self {
      Expr::Var(id) => Some(id.clone()),
      _ => None,
    }
  }

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VarId(pub(crate) String); // Identificador de cada variable

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
    match token.kind() {
      TokenKind::Bang => Some(Self::Not),
      TokenKind::Minus => Some(Self::Neg),
      _ => None,
    }
  }

  pub(crate) fn compatible_operand_type(&self) -> Type {
    match self {
      Self::Neg => Type::Int32,
      Self::Not => Type::Bool,
    }
  }

  /// Por ahora esta funcion no depende de los operandos.
  /// En un futuro eso podria cambiar, e ir directamente en UnaryExpr
  pub(crate) fn result_type(&self) -> Type {
    match self {
      Self::Neg => Type::Int32,
      Self::Not => Type::Bool,
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

  /// Obtiene el tipo compatible para el LHS y el RHS.
  /// Por ahora siempre es el mismo, pero puede cambiar en el futuro.
  pub(crate) fn compatible_operand_types(&self) -> (Type, Type) {
    match self {
      Self::Add
      | Self::Sub
      | Self::Mul
      | Self::Div
      | Self::Eq
      | Self::Neq
      | Self::Gt
      | Self::Lt
      | Self::Gte
      | Self::Lte => (Type::Int32, Type::Int32),
      Self::And | Self::Or | Self::Xor => (Type::Bool, Type::Bool),
    }
  }

  /// Por ahora esta funcion no depende de los operandos.
  /// En un futuro eso podria cambiar, e ir directamente en BinaryExpr
  pub(crate) fn result_type(&self) -> Type {
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

  pub(crate) fn to_string(&self) -> &str {
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
