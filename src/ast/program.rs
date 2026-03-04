// program = main block

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId},
    expr::Expr,
  },
  common::span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
  main_block_expr: ExprId,
  span: Span,
}

impl Program {
  pub fn new(main_block_expr: ExprId, span: Span) -> Self {
    Self {
      main_block_expr,
      span,
    }
  }

  pub fn main_block_expr(&self) -> ExprId {
    self.main_block_expr
  }

  pub fn main_block(&self, ast: &Ast) -> BlockId {
    let main_block_expr = ast.expr(self.main_block_expr());
    match main_block_expr {
      Expr::Block(bid) => bid,
      _ => unreachable!("la expresion del main_block deberia ser Block"),
    }
  }

  pub fn span(&self) -> Span {
    self.span.clone()
  }
}
