// program = main block

use crate::{ast::ExprId, common::Span};

#[cfg(test)]
use crate::ast::{Ast, BlockId, Expr};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Program {
  main_block_expr: ExprId,
  span: Span,
}

impl Program {
  pub(crate) fn new(main_block_expr: ExprId, span: Span) -> Self {
    Self {
      main_block_expr,
      span,
    }
  }

  pub(crate) fn main_block_expr(&self) -> ExprId {
    self.main_block_expr
  }

  #[cfg(test)]
  pub(crate) fn main_block(&self, ast: &Ast) -> BlockId {
    let main_block_expr = ast.expr(self.main_block_expr());
    match main_block_expr {
      Expr::Block(bid) => bid,
      _ => unreachable!("la expresion del main_block deberia ser Block"),
    }
  }

  #[cfg(test)]
  pub(crate) fn span(&self) -> Span {
    self.span.clone()
  }
}
