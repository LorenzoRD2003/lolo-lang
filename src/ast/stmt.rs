use crate::ast::{
  ast::{BlockId, ExprId, StmtId},
  expr::VarId,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
  Expr(ExprId),
  Let {
    name: VarId,
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
pub(crate) struct Block {
  pub(crate) stmts: Vec<StmtId>,
}
