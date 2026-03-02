use crate::{
  ast::{
    ast::Ast,
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::Diagnostic,
  parser::program_parsing::parse_program,
  semantic::compile_time_constant::compile_time_constant_checker::{
    CompileTimeConstantChecker, CompileTimeConstantInfo,
  },
};

pub(crate) fn compile_time_check(
  source: &str,
) -> (CompileTimeConstantInfo, Vec<Diagnostic>, Ast, Program) {
  let (ast, program) = parse_program(source);
  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast);
  compile_time_constant_checker.check_program(&program);
  let diagnostics = compile_time_constant_checker.diagnostics().to_vec();
  let resolution_info = compile_time_constant_checker.into_compile_time_constant_info();
  (resolution_info, diagnostics, ast, program)
}

#[test]
fn int_literal_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Int32(5)));
  }
}

#[test]
fn bool_literal_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { true; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Bool(true)));
  }
}

#[test]
fn unary_neg_of_constant_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { -5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Int32(-5)));
  }
}

#[test]
fn add_two_constants_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 2 + 3; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Int32(5)));
  }
}

#[test]
fn nested_constant_expression_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 4 * (2 + 3); }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Int32(20)));
  }
}

#[test]
fn comparison_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 5 > 3; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Bool(true)));
  }
}

#[test]
fn logical_expression_constants() {
  let source = r#"
    main {
      true && false;
      false && true;
      true || false;
      false || true;
      true ^^ false;
      false ^^ true;
    }
  "#;
  let (info, diagnostics, ast, program) = compile_time_check(source);
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(&expr_id), Some(&ConstValue::Bool(false)));
  }
}

#[test]
fn variable_is_not_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { let x = 5; x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    assert!(info.get(&expr_id).is_none());
  }
}

#[test]
fn mixed_expression_is_not_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { let x = 5; x + 2; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    assert!(info.get(&expr_id).is_none());
  }
}

#[test]
fn overflow_is_reported() {
  let source = r#"
    main {
      2147483647 + 1;
      100000 * 100000;
      -2147483647 - 2;
    }
  "#;
  let (info, diagnostics, ast, program) = compile_time_check(source);
  assert_eq!(diagnostics.len(), 3);
  assert!(diagnostics[0].msg().contains(&format!(
    "overflow evaluando {} {} {}",
    ConstValue::Int32(2147483647),
    BinaryOp::Add,
    ConstValue::Int32(1)
  )));

  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert!(info.get(&expr_id).is_none());
  }
}

#[test]
fn division_by_zero_is_reported() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 11 / 1; 10 / 0; }");
  assert_eq!(diagnostics.len(), 1);
  assert!(
    diagnostics[0]
      .msg()
      .contains(&format!("division por cero encontrada"))
  );

  let stmt = ast.block(program.main_block()).stmts()[1];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert!(info.get(&expr_id).is_none());
  }
}

#[test]
fn subexpressions_can_be_constant_even_if_parent_is_not() {
  let (info, diagnostics, ast, program) = compile_time_check("main { let x = 5; 2 * 3 + x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    // el padre no es constante
    assert!(info.get(&expr_id).is_none());
    // pero el (2 * 3) sí deberia estar en el map
    let expr = ast.expr(expr_id);
    if let Expr::Binary(BinaryExpr { op: _, lhs, rhs }) = expr {
      assert_eq!(info.get(&lhs), Some(&ConstValue::Int32(6)));
      assert!(info.get(&rhs).is_none());
    }
  }
}
