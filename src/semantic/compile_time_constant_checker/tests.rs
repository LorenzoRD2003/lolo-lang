use crate::{
  Diagnostic,
  ast::{Ast, AstVisitor, BinaryExpr, BinaryOp, ConstValue, Expr, Program, Stmt},
  parser::parse_program,
  semantic::{
    compile_time_constant_checker::{CompileTimeConstantChecker, CompileTimeConstantInfo},
    name_resolver::NameResolver,
  },
};

pub(crate) fn compile_time_check(
  source: &str,
) -> (CompileTimeConstantInfo, Vec<Diagnostic>, Ast, Program) {
  let (ast, program) = parse_program(source);
  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let (resolution_info, _) = resolver.into_semantic_info();
  let mut compile_time_constant_checker = CompileTimeConstantChecker::new(&ast, &resolution_info);
  compile_time_constant_checker.visit_program(&program);
  let diagnostics = compile_time_constant_checker.diagnostics().to_vec();
  let resolution_info = compile_time_constant_checker.into_compile_time_constant_info();
  (resolution_info, diagnostics, ast, program)
}

#[test]
fn int_literal_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Int32(5)));
  }
}

#[test]
fn bool_literal_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { true; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Bool(true)));
  }
}

#[test]
fn unary_neg_of_constant_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { -5; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Int32(-5)));
  }
}

#[test]
fn add_two_constants_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 2 + 3; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Int32(5)));
  }
}

#[test]
fn nested_constant_expression_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 4 * (2 + 3); }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Int32(20)));
  }
}

#[test]
fn comparison_is_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 5 > 3; }");
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Bool(true)));
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
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Bool(false)));
  }
}

#[test]
fn variable_is_not_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { let x = 5; x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    assert!(info.get(expr_id).is_none());
  }
}

#[test]
fn mixed_expression_is_not_constant() {
  let (info, diagnostics, ast, program) = compile_time_check("main { let x = 5; x + 2; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    assert!(info.get(expr_id).is_none());
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

  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert!(info.get(expr_id).is_none());
  }
}

#[test]
fn division_by_zero_is_reported() {
  let (info, diagnostics, ast, program) = compile_time_check("main { 11 / 1; 10 / 0; }");
  assert_eq!(diagnostics.len(), 1);
  assert!(
    diagnostics[0]
      .msg()
      .contains(&"division por cero encontrada".to_string())
  );

  let stmt = ast.block(program.main_block(&ast)).stmts()[1];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert!(info.get(expr_id).is_none());
  }
}

#[test]
fn subexpressions_can_be_constant_even_if_parent_is_not() {
  let (info, diagnostics, ast, program) = compile_time_check("main { let x = 5; 2 * 3 + x; }");
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmts = block.stmts();
  if let Stmt::Expr(expr_id) = ast.stmt(stmts[1]) {
    // el padre no es constante
    assert!(info.get(expr_id).is_none());
    // pero el (2 * 3) sí deberia estar en el map
    let expr = ast.expr(expr_id);
    if let Expr::Binary(BinaryExpr { op: _, lhs, rhs }) = expr {
      assert_eq!(info.get(lhs), Some(&ConstValue::Int32(6)));
      assert!(info.get(rhs).is_none());
    }
  }
}

#[test]
fn const_propagation_chain() {
  let source = r#"
    main {
      const x = 5;
      const y = x + 3;
      const z = 2 * y;
      return z;
    }
  "#;
  let (compile_time_constant_info, diagnostics, ast, program) = compile_time_check(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmt_id = block.stmts()[3];
  if let Stmt::Return(Some(expr_id)) = ast.stmt(stmt_id) {
    assert_eq!(
      compile_time_constant_info.get(expr_id),
      Some(&ConstValue::Int32(16))
    );
  }
}

#[test]
fn division_by_zero_in_const_is_error() {
  let source = r#"
    main {
      const x = 5 / 0;
      print x;
    }
  "#;

  let (info, diagnostics, ast, program) = compile_time_check(source);
  assert_eq!(diagnostics.len(), 1);
  assert!(
    diagnostics[0]
      .msg()
      .contains(&"division por cero encontrada".to_string())
  );
  let block = ast.block(program.main_block(&ast));
  let stmt = block.stmts()[1];
  if let Stmt::Print(expr_id) = ast.stmt(stmt) {
    assert!(info.get(expr_id).is_none());
  }
}

#[test]
fn const_initialized_with_block() {
  let source = r#"
    main {
      const x = { return 2 + 7; };
    }
  "#;
  let (compile_time_constant_info, diagnostics, ast, program) = compile_time_check(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmt_id = block.stmts()[0];
  if let Stmt::ConstBinding {
    var: _,
    initializer,
  } = ast.stmt(stmt_id)
  {
    assert_eq!(
      compile_time_constant_info.get(initializer),
      Some(&ConstValue::Int32(9))
    );
  }
}

#[test]
fn const_initialized_with_block_without_value() {
  let source = r#"
    main {
      const x = { return; };
    }
  "#;
  // Esto no debe dar un error, ni agregar nada al mapa. El error debe ser emitido en el CategoryChecker.
  let (compile_time_constant_info, diagnostics, _, _) = compile_time_check(source);
  assert!(compile_time_constant_info.is_empty());
  assert!(diagnostics.is_empty());
}

#[test]
fn if_expression_with_constant_condition_is_constant() {
  let source = r#"
    main {
      if true { return 10; } else { return 20; };
    }
  "#;
  let (info, diagnostics, ast, program) = compile_time_check(source);
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Int32(10)));
  }
}

#[test]
fn else_if_chain_constant_selects_matching_branch() {
  let source = r#"
    main {
      if false { return 10; } else if true { return 20; } else { return 30; };
    }
  "#;
  let (info, diagnostics, ast, program) = compile_time_check(source);
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(stmt) {
    assert_eq!(info.get(expr_id), Some(&ConstValue::Int32(20)));
  }
}

#[test]
fn if_expression_with_non_constant_condition_is_not_constant() {
  let source = r#"
    main {
      let cond = true;
      if cond { return 1; } else { return 2; };
    }
  "#;
  let (info, diagnostics, ast, program) = compile_time_check(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let if_stmt = block.stmts()[1];
  if let Stmt::Expr(expr_id) = ast.stmt(if_stmt) {
    assert!(info.get(expr_id).is_none());
  }
}

#[test]
fn const_bindings_are_exposed_by_symbol() {
  let source = r#"
    main {
      const x = 5;
      let y = 7;
      const z = x + 3;
    }
  "#;

  let (ast, program) = parse_program(source);
  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let (resolution_info, _) = resolver.into_semantic_info();

  let mut checker = CompileTimeConstantChecker::new(&ast, &resolution_info);
  checker.visit_program(&program);
  let info = checker.into_compile_time_constant_info();

  let block = ast.block(program.main_block(&ast));
  let [x_stmt, y_stmt, z_stmt] = block.stmts() else {
    panic!("se esperaban 3 statements");
  };

  let x_symbol = match ast.stmt(*x_stmt) {
    Stmt::ConstBinding { var, .. } => resolution_info
      .symbol_of(*var)
      .expect("x debe tener simbolo resuelto"),
    _ => panic!("primer statement debe ser const"),
  };
  let y_symbol = match ast.stmt(*y_stmt) {
    Stmt::LetBinding { var, .. } => resolution_info
      .symbol_of(*var)
      .expect("y debe tener simbolo resuelto"),
    _ => panic!("segundo statement debe ser let"),
  };
  let z_symbol = match ast.stmt(*z_stmt) {
    Stmt::ConstBinding { var, .. } => resolution_info
      .symbol_of(*var)
      .expect("z debe tener simbolo resuelto"),
    _ => panic!("tercer statement debe ser const"),
  };

  assert_eq!(info.symbol_constant(x_symbol), Some(&ConstValue::Int32(5)));
  assert_eq!(info.symbol_constant(z_symbol), Some(&ConstValue::Int32(8)));
  assert!(info.symbol_constant(y_symbol).is_none());
}

#[test]
fn const_binding_with_non_constant_initializer_is_not_exposed_by_symbol() {
  let source = r#"
    main {
      let y = 2;
      const x = y + 1;
    }
  "#;

  let (ast, program) = parse_program(source);
  let mut resolver = NameResolver::new(&ast);
  resolver.visit_program(&program);
  let (resolution_info, _) = resolver.into_semantic_info();

  let mut checker = CompileTimeConstantChecker::new(&ast, &resolution_info);
  checker.visit_program(&program);
  let info = checker.into_compile_time_constant_info();

  let block = ast.block(program.main_block(&ast));
  let x_symbol = match ast.stmt(block.stmts()[1]) {
    Stmt::ConstBinding { var, .. } => resolution_info
      .symbol_of(*var)
      .expect("x debe tener simbolo resuelto"),
    _ => panic!("segundo statement debe ser const"),
  };

  assert!(info.symbol_constant(x_symbol).is_none());
}
