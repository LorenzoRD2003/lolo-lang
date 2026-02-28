use crate::ast::ast::{BlockId, ExprId, StmtId};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
  Expr(ExprId),
  LetBinding {
    var: ExprId, // va a tener que ser una variable o damos un error
    initializer: ExprId,
  },
  // ConstBinding { // La diferencia va a ser en la parte semantica que pondremos Mutability::Immutable
  //   var: ExprId, // va a tener que ser una variable o damos un error
  //   initializer: ExprId,
  // },
  Assign {
    var: ExprId, // va a tener que ser una variable ya inicializada o damos un error
    value_expr: ExprId,
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
