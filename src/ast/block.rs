use crate::ast::{
  ast::{Ast, StmtId},
  stmt::Stmt,
};

/// Block tambien va a ser arena-based
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Block {
  stmts: Vec<StmtId>,
  terminator: Option<StmtId>,
}

impl Block {
  pub(crate) fn new() -> Self {
    Self {
      stmts: vec![],
      terminator: None,
    }
  }

  pub(crate) fn with_stmts(ast: &Ast, stmts: Vec<StmtId>) -> Self {
    let terminator = stmts
      .last()
      .copied()
      .and_then(|stmt_id| matches!(ast.stmt(stmt_id), Stmt::Return(_)).then_some(stmt_id));
    Self { stmts, terminator }
  }

  pub(crate) fn stmts(&self) -> &[StmtId] {
    &self.stmts
  }

  pub(crate) fn add_stmt(&mut self, stmt: StmtId) {
    self.stmts.push(stmt)
  }

  pub(crate) fn terminator(&self) -> Option<StmtId> {
    self.terminator
  }

  pub(crate) fn set_terminator(&mut self, terminator: Option<StmtId>) {
    self.terminator = terminator;
  }
}
