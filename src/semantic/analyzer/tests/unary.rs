use crate::{
  ast::{
    ast::Ast,
    expr::{ConstValue, Expr, UnaryExpr, UnaryOp},
  },
  diagnostics::diagnostic::Diagnostic,
  semantic::{analyzer::tests::semantic_analyzer, types::Type},
};

#[test]
fn analyze_unary_const_expr() {
  let mut ast = Ast::empty();
  let inner = ast.add_expr(Expr::Const(ConstValue::Int32(5)), 0..1);
  let expr_id = ast.add_expr(
    Expr::Unary(UnaryExpr {
      op: UnaryOp::Neg,
      operand: inner,
    }),
    0..2,
  );
  let mut sem = semantic_analyzer(&ast);

  sem.analyze_expr(expr_id);
  let info = sem.semantic_info.expr_info(expr_id);
  assert_eq!(info.r#type(), Type::Int32);
  let category = info.category();
  assert!(category.is_value() && !category.is_place() && category.is_constant());
  assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(-5)));
  assert!(sem.diagnostics().is_empty());
}

#[test]
fn type_mismatch_error_in_unary_expr() {
  // !5  → error
  let mut ast = Ast::empty();
  let operand = ast.add_expr(Expr::Const(ConstValue::Int32(5)), 1..2);
  let expr = ast.add_expr(
    Expr::Unary(UnaryExpr {
      op: UnaryOp::Not,
      operand,
    }),
    0..2,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr);
  let info = sem.semantic_info.expr_info(expr);
  assert_eq!(info.r#type(), Type::DefaultErrorType);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("mismatch"))
  );
}
