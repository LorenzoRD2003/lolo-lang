use crate::{
  ast::{
    ast::Ast,
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, VarId},
    program::Program,
    stmt::{Block, Stmt},
  },
  diagnostics::diagnostic::Diagnostic,
  semantic::{analyzer::tests::semantic_analyzer, symbol::Mutability, types::Type},
};

#[test]
fn analyze_var_expr_resolved() {
  let mut ast = Ast::empty();
  let var_id = VarId("x".into());
  let expr_id = ast.add_expr(Expr::Var(var_id.clone()), 0..1);
  let mut sem = semantic_analyzer(&ast);
  sem.symbol_table.add_symbol(
    &var_id,
    Type::Bool,
    Mutability::Immutable,
    ast.expr_span(expr_id),
  );
  sem.analyze_expr(expr_id);

  let info = sem.semantic_info.expr_info(expr_id);
  assert!(info.symbol().is_some());
  assert_eq!(info.r#type(), Type::Bool);
  let category = info.category();
  assert!(category.is_value() && category.is_place() && !category.is_constant());
  assert!(info.compile_time_constant().is_none());
  assert!(sem.diagnostics().is_empty());
}

#[test]
fn analyze_var_expr_unresolved() {
  let mut ast = Ast::empty();
  let expr_id = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
  let mut sem = semantic_analyzer(&ast);

  sem.analyze_expr(expr_id);
  let info = sem.semantic_info.expr_info(expr_id);
  assert_eq!(info.symbol(), None);
  assert_eq!(info.r#type(), Type::DefaultErrorType);
  assert!(info.compile_time_constant().is_none());
  assert_eq!(sem.diagnostics().len(), 1);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("variable 'x' indefinida"))
  );
}

// TODO: Este test cuando tenga alguna expresion que no tenga la categoria ValueExpr.
// #[test]
// fn let_with_non_value_expr_initializer() {
// }

#[test]
fn let_with_var_non_place_expr() {
  // let x + 42 = 3;
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 8..10);
  let sum_var = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Add,
      lhs,
      rhs,
    }),
    6..7,
  );
  let initializer = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 13..14);
  let stmt = ast.add_stmt(
    Stmt::Let {
      var: sum_var,
      initializer,
    },
    0..15,
  );
  let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..15);
  let program = Program::new(block, 0..15);

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  let stmt_info = sem.semantic_info.stmt_info(stmt);
  assert!(stmt_info.symbol_declared().is_none());
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("place expression"))
  );
}
