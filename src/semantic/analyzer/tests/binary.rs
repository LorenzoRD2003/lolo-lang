use crate::{
  ast::{
    ast::Ast,
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr},
  },
  diagnostics::diagnostic::Diagnostic,
  semantic::{analyzer::tests::semantic_analyzer, types::Type},
};

#[test]
fn analyze_binary_const_expr() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 2..3);
  let expr_id = ast.add_expr(
    Expr::Binary(BinaryExpr {
      lhs,
      op: BinaryOp::Add,
      rhs,
    }),
    0..3,
  );

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr_id);
  let info = sem.semantic_info.expr_info(expr_id);
  let category = info.category();
  assert!(category.is_value() && !category.is_place() && category.is_constant());
  assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(5)));
}

#[test]
fn type_mismatch_error_in_binary_expr() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Bool(false)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 2..3);
  let expr_id = ast.add_expr(
    Expr::Binary(BinaryExpr {
      lhs,
      rhs,
      op: BinaryOp::Sub,
    }),
    0..3,
  );
  let mut sem = semantic_analyzer(&ast);

  sem.analyze_expr(expr_id);
  assert_eq!(sem.diagnostics().len(), 2);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("mismatch de tipos"))
  );
}

#[test]
fn binary_rhs_type_mismatch_only() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 4..8);
  let expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Add,
      lhs,
      rhs,
    }),
    0..8,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr);
  let info = sem.semantic_info.expr_info(expr);
  assert_eq!(info.r#type(), Type::DefaultErrorType);
}
