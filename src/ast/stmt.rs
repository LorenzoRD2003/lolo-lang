use crate::ast::{
  ast::{BlockId, ExprId, StmtId},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
  Expr(ExprId),
  Let {
    var: ExprId, // va a tener que ser una variable o damos un error
    initializer: ExprId,
  },
  Return(ExprId),
  If {
    condition: ExprId,
    if_block: BlockId,
  },
  IfElse {
    condition: ExprId,
    if_block: BlockId,
    else_block: BlockId,
  },
  Print(ExprId),
}

/// Block tambien va a ser arena-based
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  stmts: Vec<StmtId>,
}

impl Block {
  pub fn new() -> Self {
    Self { stmts: vec![] }
  }

  pub fn with_stmts(stmts: Vec<StmtId>) -> Self {
    Self { stmts }
  }

  pub fn stmts(&self) -> &[StmtId] {
    &self.stmts
  }

  pub fn add_stmt(&mut self, stmt: StmtId) {
    self.stmts.push(stmt)
  }
}
