use crate::ast::{ExprId, StmtId};

#[cfg(test)]
use crate::ast::{Ast, Stmt};

/// Block tambien va a ser arena-based
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Block {
  stmts: Vec<StmtId>,
  tail_expr: Option<ExprId>,
}

impl Block {
  pub(crate) fn new() -> Self {
    Self {
      stmts: vec![],
      tail_expr: None,
    }
  }

  #[cfg(test)]
  pub(crate) fn with_stmts(ast: &Ast, stmts: Vec<StmtId>) -> Self {
    let tail_expr = stmts.last().and_then(|stmt_id| match ast.stmt(*stmt_id) {
      Stmt::Return(Some(expr_id)) => Some(*expr_id),
      _ => None,
    });
    Self { stmts, tail_expr }
  }

  pub(crate) fn stmts(&self) -> &[StmtId] {
    &self.stmts
  }

  pub(crate) fn add_stmt(&mut self, stmt: StmtId) {
    self.stmts.push(stmt)
  }

  pub(crate) fn tail_expr(&self) -> Option<ExprId> {
    self.tail_expr
  }

  pub(crate) fn set_tail_expr(&mut self, tail_expr: Option<ExprId>) {
    self.tail_expr = tail_expr;
  }
}
