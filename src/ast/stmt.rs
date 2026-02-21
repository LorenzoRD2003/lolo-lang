use super::expr::{Expr, VarExpr};

pub enum Stmt {
  Let(VarExpr, Expr),
  Return(Expr),
  If(Expr, Block),
  IfElse(Expr, Block, Block),
  Print(Expr),
}

pub type Block = Vec<Stmt>;
