use crate::{
  ast::{Ast, AstVisitor, Block, ConstValue, Expr, Program, Stmt},
  diagnostics::Diagnostic,
  parser::program_parsing::parse_program,
  semantic::{
    category_checker::category_checker::{CategoryChecker, CategoryInfo},
    compile_time_constant::compile_time_constant_checker::CompileTimeConstantChecker,
    resolver::name_resolver::NameResolver,
  },
};

fn category_check(source: &str) -> (CategoryInfo, Vec<Diagnostic>, Ast, Program) {
  let (ast, program) = parse_program(source);
  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let (resolution_info, _) = resolver.into_semantic_info();
  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast, &resolution_info);
  compile_time_constant_checker.visit_program(&program);
  let const_info = compile_time_constant_checker.into_compile_time_constant_info();
  let mut category_checker = CategoryChecker::new(&ast, &const_info);
  category_checker.visit_program(&program);
  let diagnostics = category_checker.diagnostics().to_vec();
  let info = category_checker.into_category_info();
  (info, diagnostics, ast, program)
}

#[test]
fn int_literal_is_value_and_constant() {
  let (info, diagnostics, ast, program) = category_check("main { 5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
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
  let block = ast.block(program.main_block(&ast));
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
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
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
  let block = ast.block(program.main_block(&ast));
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
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
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
  let block = ast.block(program.main_block(&ast));
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
  let block = ast.add_block(Block::with_stmts(&ast, vec![stmt]), 0..4);
  let block_expr = ast.add_block_expr(block);
  let program = Program::new(block_expr, 0..4);

  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let (resolution_info, _) = resolver.into_semantic_info();

  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast, &resolution_info);
  compile_time_constant_checker.visit_program(&program);
  let compile_time_constant_info = compile_time_constant_checker.into_compile_time_constant_info();

  let mut category_checker = CategoryChecker::new(&ast, &compile_time_constant_info);
  category_checker.visit_program(&program);

  assert_eq!(category_checker.diagnostics().len(), 1);
  assert!(
    category_checker.diagnostics()[0]
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
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
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
  let block = ast.block(program.main_block(&ast));
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
  let block = ast.block(program.main_block(&ast));
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

#[test]
fn const_binding_literal_is_valid() {
  let source = r#"
    main {
      const x = 5;
      x;
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn const_binding_constant_expression_is_valid() {
  let source = r#"
    main {
      const x = (5 + 3) * 2 - 4 / 2;
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn const_binding_with_non_constant_variable_is_error() {
  let source = r#"
    main {
      let y = 10;
      const x = y;
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains("se esperaba una constant expression")
  );
}

#[test]
fn const_binding_with_non_constant_expression_is_error() {
  let source = r#"
    main {
      let y = 2;
      const x = y + 3;
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains("se esperaba una constant expression")
  );
}

#[test]
fn const_binding_lhs_must_be_place() {
  let mut ast = Ast::empty();
  let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(5)), 0..1);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(10)), 3..4);
  let stmt = ast.add_stmt(
    Stmt::ConstBinding {
      var: lhs,
      initializer: rhs,
    },
    0..4,
  );
  let block = ast.add_block(Block::with_stmts(&ast, vec![stmt]), 0..4);
  let block_expr = ast.add_block_expr(block);
  let program = Program::new(block_expr, 0..4);

  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let (resolution_info, _) = resolver.into_semantic_info();

  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast, &resolution_info);
  compile_time_constant_checker.visit_program(&program);
  let compile_time_constant_info = compile_time_constant_checker.into_compile_time_constant_info();

  let mut category_checker = CategoryChecker::new(&ast, &compile_time_constant_info);
  category_checker.visit_program(&program);

  assert_eq!(category_checker.diagnostics().len(), 1);
  assert!(
    category_checker.diagnostics()[0]
      .msg()
      .contains(&format!("se esperaba una place expression"))
  );
}

#[test]
fn shadowing_const_with_let_breaks_constness() {
  let source = r#"
    main {
      const x = 5;
      if true {
        let x = 10;
        const y = x + 1;
      }
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains("se esperaba una constant expression")
  );
}

#[test]
fn shadowing_let_with_const_restores_constness() {
  let source = r#"
    main {
      let x = 10;
      if false {
        const x = 5;
        const y = x + 1;
      }
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn const_cannot_use_later_const_definition() {
  let source = r#"
    main {
      const y = x + 1;
      const x = 5;
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(
    diagnostics[0]
      .msg()
      .contains("se esperaba una constant expression")
  );
}

#[test]
fn shadowing_does_not_leak_outer_const_value() {
  // aca queremos testear que x + 1 no es una constant expression, porque hay shadowing del const binding.
  let source = r#"
    main {
      const x = 5;
      if true {
        let x = 10;
        const y = x + 1;
      }
    }
  "#;
  let (_, diagnostics, _, _) = category_check(source);
  assert!(
    diagnostics[0]
      .msg()
      .contains("se esperaba una constant expression")
  );
}

#[test]
fn const_initialized_with_block() {
  let source = r#"
    main {
      const x = { return 2 + 7; };
    }
  "#;
  let (category_info, diagnostics, ast, program) = category_check(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmt_id = block.stmts()[0];
  dbg!(&category_info);
  if let Stmt::ConstBinding { var, initializer } = ast.stmt(stmt_id) {
    let var_cat = category_info.get(&var).unwrap();
    // dbg!(const_cat);
    assert!(var_cat.is_value() && !var_cat.is_constant() && var_cat.is_place());
    let block_cat = category_info.get(&initializer).unwrap();
    assert!(block_cat.is_value() && block_cat.is_constant() && !block_cat.is_place());
  }
}

#[test]
fn const_initialized_with_block_without_value() {
  let source = r#"
    main {
      const x = { return; };
    }
  "#;
  let (category_info, diagnostics, ast, program) = category_check(source);
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains("se esperaba una constant expression")
  );
  let block = ast.block(program.main_block(&ast));
  let stmt_id = block.stmts()[0];
  if let Stmt::ConstBinding { var, initializer } = ast.stmt(stmt_id) {
    let var_cat = category_info.get(&var).unwrap();
    let block_cat = category_info.get(&initializer).unwrap();
    assert!(var_cat.is_value() && !var_cat.is_constant() && var_cat.is_place());
    assert!(block_cat.is_value() && !block_cat.is_constant() && !block_cat.is_place());
  }
}
