// program = main block

use crate::{ast::ast::BlockId, common::span::Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
  block: BlockId,
  span: Span,
}

impl Program {
  pub fn new(block: BlockId, span: Span) -> Self {
    Self { block, span }
  }

  pub fn main_block(&self) -> BlockId {
    self.block
  }

  pub fn span(&self) -> Span {
    self.span.clone()
  }
}
