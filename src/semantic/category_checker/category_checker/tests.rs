use crate::{
  ast::{
    ast::Ast,
    expr::{ConstValue, Expr},
    program::Program,
    stmt::{Block, Stmt},
  },
  diagnostics::diagnostic::Diagnostic,
  parser::program_parsing::parse_program,
  semantic::{
    category_checker::category_checker::{CategoryChecker, CategoryInfo},
    compile_time_constant::compile_time_constant_checker::CompileTimeConstantChecker,
  },
};

fn category_check(source: &str) -> (CategoryInfo, Vec<Diagnostic>, Ast, Program) {
  let (ast, program) = parse_program(source);
  let mut diagnostics = Vec::new();
  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast, &mut diagnostics);
  compile_time_constant_checker.check_program(&program);
  let const_info = compile_time_constant_checker.into_compile_time_constant_info();
  let mut category_checker = CategoryChecker::new(&ast, &const_info, &mut diagnostics);
  category_checker.check_program(&program);
  let info = category_checker.into_category_info();
  (info, diagnostics, ast, program)
}

#[test]
fn int_literal_is_value_and_constant() {
  let (info, diagnostics, ast, program) = category_check("main { 5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    let cat = info.get(&expr_id).unwrap();
    assert!(cat.is_value());
    assert!(cat.is_constant());
    assert!(!cat.is_place());
  }
}

#[test]
fn variable_is_value_and_place() {
  let (info, diagnostics, ast, program) = category_check("main { let x = 5; x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    let cat = info.get(&expr_id).unwrap();
    assert!(cat.is_value());
    assert!(cat.is_place());
    assert!(!cat.is_constant());
  }
}

#[test]
fn binary_constant_is_value_and_constant() {
  let (info, diagnostics, ast, program) = category_check("main { 2 + 3; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    let cat = info.get(&expr_id).unwrap();
    assert!(cat.is_value());
    assert!(cat.is_constant());
    assert!(!cat.is_place());
  }
}

#[test]
fn binary_non_constant_is_only_value() {
  let (info, diagnostics, ast, program) = category_check("main { let x = 5; x + 3; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    let cat = info.get(&expr_id).unwrap();
    assert!(cat.is_value());
    assert!(!cat.is_constant());
    assert!(!cat.is_place());
  }
}

#[test]
fn unary_constant_is_value_and_constant() {
  let (info, diagnostics, ast, program) = category_check("main { -5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    let cat = info.get(&expr_id).unwrap();
    assert!(cat.is_value());
    assert!(cat.is_constant());
    assert!(!cat.is_place());
  }
}

#[test]
fn unary_non_constant_is_only_value() {
  let (info, diagnostics, ast, program) = category_check("main { let x = 5; -x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    let cat = info.get(&expr_id).unwrap();
    assert!(cat.is_value());
    assert!(!cat.is_constant());
    assert!(!cat.is_place());
  }
}

#[test]
fn valid_assignment_produces_no_error() {
  let (_, diagnostics, _, _) = category_check("main { let x = 5; x = 10; }");
  assert!(diagnostics.is_empty());
}

#[test]
fn assignment_to_non_place_is_error() {
  // let (_, diagnostics, _, _) = category_check("main { 5 = 10; }");
  // Este test lo hago construyendo el AST a mano, porque es defensivo ante cambios futuros
  // El parser no me permite usar un codigo fuente tal que el LHS no sea una PlaceExpr (por ahora)
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(5)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(10)), 3..4);
  let stmt = ast.add_stmt(
    Stmt::Assign {
      var: lhs,
      value_expr: rhs,
    },
    0..4,
  );
  let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..4);
  let program = Program::new(block, 0..4);

  let mut diagnostics = Vec::new();
  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast, &mut diagnostics);
  compile_time_constant_checker.check_program(&program);
  let compile_time_constant_info = compile_time_constant_checker.into_compile_time_constant_info();

  let mut category_checker =
    CategoryChecker::new(&ast, &compile_time_constant_info, &mut diagnostics);
  category_checker.check_program(&program);

  assert_eq!(diagnostics.len(), 1);
  assert!(
    diagnostics[0]
      .msg()
      .contains(&format!("se esperaba una place expression"))
  );
}

// TODO: cuando tenga algo que no sea ValueExpr
// #[test]
// fn assignment_with_non_value_is_error() {
//   let (_, diagnostics, _, _) = category_check("main { 5 = 10; }");
//   assert_eq!(diagnostics.len(), 1);
//   assert!(diagnostics[0].msg().contains(&format!("se esperaba una value expression")));
// }

#[test]
fn subexpression_constant_marked_correctly() {
  let (info, diagnostics, ast, program) = category_check("main { (2 + 3) + 4; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block()).stmts()[0];
  if let Stmt::Expr(root_id) = ast.stmt(stmt) {
    // root debería ser constante
    let root_cat = info.get(&root_id).unwrap();
    assert!(root_cat.is_constant());
  }
}

#[test]
fn variable_is_never_constant() {
  let (info, diagnostics, ast, program) = category_check("main { let x = 5; x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    let cat = info.get(&expr_id).unwrap();
    assert!(!cat.is_constant());
  }
}

#[test]
fn constant_flag_depends_on_compile_time_analysis() {
  let (info, diagnostics, ast, program) = category_check("main { let x = 5; (2 + 3) + x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    let cat = info.get(&expr_id).unwrap();
    assert!(!cat.is_constant());
  }
}

#[test]
fn check_if_stmt() {
  let source = r#"
    main {
      if true {
        let x = 1;
      }
      if false {
        let y = 1;
      } else {
        let y = 2;
      }
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(diagnostics.is_empty());
}
