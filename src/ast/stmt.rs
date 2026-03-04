use crate::ast::ast::{BlockId, ExprId};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
  Expr(ExprId),
  LetBinding {
    var: ExprId, // va a tener que ser una variable o damos un error
    initializer: ExprId,
  },
  ConstBinding {
    // La diferencia va a ser en la parte semantica que pondremos Mutability::Immutable
    var: ExprId, // va a tener que ser una variable o damos un error
    initializer: ExprId,
  },
  Assign {
    var: ExprId, // va a tener que ser una variable ya inicializada o damos un error
    value_expr: ExprId,
  },
  Return(Option<ExprId>),
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
