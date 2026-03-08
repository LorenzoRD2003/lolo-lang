use crate::ast::ExprId;

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
  Print(ExprId),
}

impl PartialEq<Stmt> for &Stmt {
  fn eq(&self, other: &Stmt) -> bool {
    **self == *other
  }
}
