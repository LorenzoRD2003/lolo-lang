// program = main block

use crate::{ast::ast::BlockId, common::span::Span};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Program {
  block: BlockId,
  span: Span,
}

impl Program {
  pub(crate) fn new(block: BlockId, span: Span) -> Self {
    Self { block, span }
  }

  pub(crate) fn main_block(&self) -> BlockId {
    self.block
  }

  pub(crate) fn span(&self) -> Span {
    self.span.clone()
  }
}
