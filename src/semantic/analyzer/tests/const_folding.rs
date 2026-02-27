use crate::{
  ast::{
    ast::Ast,
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, UnaryExpr, UnaryOp, VarId},
  },
  diagnostics::diagnostic::Diagnostic,
  semantic::analyzer::tests::semantic_analyzer,
};

#[test]
fn arithmetic_constant_fold_success() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(18)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 4..5);
  let add_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Add,
      lhs,
      rhs,
    }),
    0..5,
  );
  let sub_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Sub,
      lhs,
      rhs,
    }),
    0..5,
  );
  let mul_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Mul,
      lhs,
      rhs,
    }),
    0..5,
  );
  let div_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Div,
      lhs,
      rhs,
    }),
    0..5,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(add_expr);
  let add_info = sem.semantic_info.expr_info(add_expr);
  assert_eq!(
    add_info.compile_time_constant(),
    Some(&ConstValue::Int32(21))
  );
  sem.analyze_expr(sub_expr);
  let sub_info = sem.semantic_info.expr_info(sub_expr);
  assert_eq!(
    sub_info.compile_time_constant(),
    Some(&ConstValue::Int32(15))
  );
  sem.analyze_expr(mul_expr);
  let mul_info = sem.semantic_info.expr_info(mul_expr);
  assert_eq!(
    mul_info.compile_time_constant(),
    Some(&ConstValue::Int32(54))
  );
  sem.analyze_expr(div_expr);
  let div_info = sem.semantic_info.expr_info(div_expr);
  assert_eq!(
    div_info.compile_time_constant(),
    Some(&ConstValue::Int32(6))
  );
}

#[test]
fn test_zero_division_error() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(0)), 2..3);
  let expr_id = ast.add_expr(
    Expr::Binary(BinaryExpr {
      lhs,
      rhs,
      op: BinaryOp::Div,
    }),
    0..3,
  );
  let mut sem = semantic_analyzer(&ast);

  sem.analyze_expr(expr_id);
  let info = sem.semantic_info.expr_info(expr_id);
  assert!(info.compile_time_constant().is_none());
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg() == String::from("division por cero encontrada"))
  );
}

#[test]
fn addition_overflow_emits_error() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(i32::MAX)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
  let expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Add,
      lhs,
      rhs,
    }),
    0..5,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("overflow"))
  );
}

#[test]
fn subtraction_overflow_emits_error() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(i32::MIN)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
  let expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Sub,
      lhs,
      rhs,
    }),
    0..5,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("overflow"))
  );
}

#[test]
fn multiplication_overflow_emits_error() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(i32::MAX)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
  let expr_id = ast.add_expr(
    Expr::Binary(BinaryExpr {
      lhs,
      rhs,
      op: BinaryOp::Mul,
    }),
    0..3,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr_id);
  let info = sem.semantic_info.expr_info(expr_id);
  assert!(info.compile_time_constant().is_none());
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("overflow"))
  );
}

#[test]
fn unary_not_constant_fold() {
  let mut ast = Ast::empty();
  let operand = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 1..5);
  let expr = ast.add_expr(
    Expr::Unary(UnaryExpr {
      op: UnaryOp::Not,
      operand,
    }),
    0..5,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr);
  let info = sem.semantic_info.expr_info(expr);
  assert_eq!(info.compile_time_constant(), Some(&ConstValue::Bool(false)));
}

#[test]
fn comparison_constant_fold_success() {
  // 3 > 4 --> false
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(4)), 4..5);
  let eq_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Eq,
      lhs,
      rhs,
    }),
    0..5,
  );
  let neq_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Neq,
      lhs,
      rhs,
    }),
    0..5,
  );
  let gt_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Gt,
      lhs,
      rhs,
    }),
    0..5,
  );
  let lt_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Lt,
      lhs,
      rhs,
    }),
    0..5,
  );
  let gte_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Gte,
      lhs,
      rhs,
    }),
    0..5,
  );
  let lte_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Lte,
      lhs,
      rhs,
    }),
    0..5,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(eq_expr);
  let eq_info = sem.semantic_info.expr_info(eq_expr);
  assert_eq!(
    eq_info.compile_time_constant(),
    Some(&ConstValue::Bool(false))
  );
  sem.analyze_expr(neq_expr);
  let neq_info = sem.semantic_info.expr_info(neq_expr);
  assert_eq!(
    neq_info.compile_time_constant(),
    Some(&ConstValue::Bool(true))
  );
  sem.analyze_expr(gt_expr);
  let gt_info = sem.semantic_info.expr_info(gt_expr);
  assert_eq!(
    gt_info.compile_time_constant(),
    Some(&ConstValue::Bool(false))
  );
  sem.analyze_expr(lt_expr);
  let lt_info = sem.semantic_info.expr_info(lt_expr);
  assert_eq!(
    lt_info.compile_time_constant(),
    Some(&ConstValue::Bool(true))
  );
  sem.analyze_expr(gte_expr);
  let gte_info = sem.semantic_info.expr_info(gte_expr);
  assert_eq!(
    gte_info.compile_time_constant(),
    Some(&ConstValue::Bool(false))
  );
  sem.analyze_expr(lte_expr);
  let lte_info = sem.semantic_info.expr_info(lte_expr);
  assert_eq!(
    lte_info.compile_time_constant(),
    Some(&ConstValue::Bool(true))
  );
}

#[test]
fn logical_and_constant_fold_success() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Bool(false)), 4..5);
  dbg!(1);
  let and_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::And,
      lhs,
      rhs,
    }),
    0..5,
  );
  let or_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Or,
      lhs,
      rhs,
    }),
    5..10,
  );
  let xor_expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Xor,
      lhs,
      rhs,
    }),
    10..15,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(and_expr);
  let and_info = sem.semantic_info.expr_info(and_expr);
  assert_eq!(
    and_info.compile_time_constant(),
    Some(&ConstValue::Bool(false))
  );
  sem.analyze_expr(or_expr);
  let or_info = sem.semantic_info.expr_info(or_expr);
  assert_eq!(
    or_info.compile_time_constant(),
    Some(&ConstValue::Bool(true))
  );
  sem.analyze_expr(xor_expr);
  let xor_info = sem.semantic_info.expr_info(xor_expr);
  assert_eq!(
    xor_info.compile_time_constant(),
    Some(&ConstValue::Bool(true))
  );
}

#[test]
fn binary_with_error_operand_is_not_constant_folded() {
  // x + 3 donde x no existe
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 2..3);
  let expr = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Add,
      lhs,
      rhs,
    }),
    0..3,
  );
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_expr(expr);
  let info = sem.semantic_info.expr_info(expr);
  assert!(info.compile_time_constant().is_none());
  assert!(!sem.diagnostics().is_empty());
}
