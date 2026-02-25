// program = main block

use crate::{ast::ast::BlockId, common::span::Span};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Program {
  pub(crate) block: BlockId,
  pub(crate) span: Span,
}
