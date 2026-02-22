use super::expr::{Expr, VarId};
use super::span::{Span, Spanned};

pub enum Stmt {
  Let {
    name: VarId,
    value: Expr,
  },
  Return(Expr),
  If {
    condition: Expr,
    if_block: Block,
  },
  IfElse {
    condition: Expr,
    if_block: Block,
    else_block: Block,
  },
  Print(Expr),
}

impl Spanned for Stmt {
  fn span(&self) -> Span {
    match &self {
      Stmt::Let { name, value } => todo!(),
      Stmt::Return(expr) => todo!(),
      Stmt::If {
        condition,
        if_block,
      } => todo!(),
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      } => todo!(),
      Stmt::Print(expr) => todo!(),
    }
  }
}

pub type Block = Vec<Stmt>;

impl Spanned for Block {
  fn span(&self) -> Span {
    todo!()
  }
}
