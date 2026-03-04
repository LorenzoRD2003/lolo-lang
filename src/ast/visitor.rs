use crate::ast::{
  ast::{Ast, BlockId, ExprId, StmtId},
  expr::Expr,
  program::Program,
  stmt::Stmt,
};

pub trait AstVisitor {
  fn visit_block(&mut self, block_id: BlockId);
  fn visit_stmt(&mut self, stmt_id: StmtId);
  fn visit_expr(&mut self, expr_id: ExprId);
}

/// Caminata estandar de programa para un visitor del AST.
pub fn walk_program<V: AstVisitor>(visitor: &mut V, program: &Program) {
  visitor.visit_expr(program.main_block_expr());
}

/// Caminata estandar de bloque para un visitor del AST.
pub fn walk_block<V: AstVisitor>(visitor: &mut V, ast: &Ast, block_id: BlockId) {
  let block = ast.block(block_id);
  for stmt in block.stmts() {
    visitor.visit_stmt(*stmt);
  }
}

/// Caminata estandar de statement para un visitor del AST.
pub fn walk_stmt<V: AstVisitor>(visitor: &mut V, ast: &Ast, stmt_id: StmtId) {
  match ast.stmt(stmt_id) {
    Stmt::LetBinding {
      var,
      initializer: value_expr,
    }
    | Stmt::ConstBinding {
      var,
      initializer: value_expr,
    }
    | Stmt::Assign { var, value_expr } => {
      visitor.visit_expr(var);
      visitor.visit_expr(value_expr);
    }
    Stmt::Expr(expr_id) | Stmt::Print(expr_id) | Stmt::Return(Some(expr_id)) => {
      visitor.visit_expr(expr_id);
    }
    Stmt::Return(None) => {}
    Stmt::If {
      condition,
      if_block,
    } => {
      visitor.visit_expr(condition);
      visitor.visit_block(if_block);
    }
    Stmt::IfElse {
      condition,
      if_block,
      else_block,
    } => {
      visitor.visit_expr(condition);
      visitor.visit_block(if_block);
      visitor.visit_block(else_block);
    }
  }
}

/// Caminata estandar de expresion para un visitor del AST.
pub fn walk_expr<V: AstVisitor>(visitor: &mut V, ast: &Ast, expr_id: ExprId) {
  match ast.expr(expr_id) {
    Expr::Var(_) | Expr::Const(_) => {}
    Expr::Unary(unary_expr) => visitor.visit_expr(unary_expr.operand),
    Expr::Binary(binary_expr) => {
      visitor.visit_expr(binary_expr.lhs);
      visitor.visit_expr(binary_expr.rhs);
    }
    Expr::Block(block_id) => visitor.visit_block(block_id),
  }
}
