use crate::{
  ast::{Ast, AstVisitor, BinaryOp, Expr, Program, Stmt, UnaryOp},
  diagnostics::Diagnostic,
  semantic::{
    name_resolver::{ResolutionInfo, resolve},
    type_checker::{TypeChecker, TypeInfo},
    types::Type,
  },
};

fn typecheck(source: &str) -> (ResolutionInfo, TypeInfo, Vec<Diagnostic>, Ast, Program) {
  let (resolution_info, _, _, ast, program) = resolve(source);
  let mut checker = TypeChecker::new(&ast, &resolution_info);
  checker.visit_program(&program);
  let diagnostics = checker.diagnostics().to_vec();
  let type_info = checker.into_type_info();
  (resolution_info, type_info, diagnostics, ast, program)
}

#[test]
fn const_int_has_type_int() {
  let source = r#"
    main {
      let x = 5;
    }
  "#;
  let (_, type_info, diagnostics, ast, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::LetBinding { initializer, .. } = ast.stmt(stmt) {
    assert_eq!(type_info.type_of_expr(initializer), Type::Int32);
  }
}

#[test]
fn variable_usage_has_correct_type() {
  let source = r#"
    main {
      let x = 5;
      let y = x;
    }
  "#;
  let (_, type_info, diagnostics, ast, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block(&ast));
  let stmts = block.stmts();
  let stmt_y = ast.stmt(stmts[1]);
  if let Stmt::LetBinding { initializer, .. } = stmt_y {
    assert_eq!(type_info.type_of_expr(initializer), Type::Int32);
  }
}

#[test]
fn assignment_type_mismatch_detected() {
  let source = r#"
    main {
      let x = 5;
      x = true;
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);

  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains(&format!(
    "mismatch de tipos: se esperaba {}, pero se encontro {}",
    Type::Int32,
    Type::Bool
  )));
}

#[test]
fn binary_int_plus_int_is_int() {
  let source = r#"
    main {
      let x = 1 + 2;
    }
  "#;
  let (_, type_info, diagnostics, ast, program) = typecheck(source);

  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::LetBinding { initializer, .. } = ast.stmt(stmt) {
    assert_eq!(type_info.type_of_expr(initializer), Type::Int32);
  }
}

#[test]
fn invalid_binary_operation_detected() {
  let source = r#"
    main {
      let x = 1 + true;
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);

  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains(&format!(
    "operacion binaria invalida: {}, el LHS es de tipo {} y el RHS es de tipo {}",
    BinaryOp::Add,
    Type::Int32,
    Type::Bool
  )));
}

#[test]
fn if_condition_must_be_bool() {
  let source = r#"
    main {
      if 5 { }
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);

  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains(&format!(
    "se encontro una condicion no booleana, de tipo {}",
    Type::Int32
  )))
}

#[test]
fn if_expression_with_else_flows_type_into_let() {
  let source = r#"
    main {
      let x = if true {
        return 10;
      } else {
        return 20;
      };
    }
  "#;
  let (resolution_info, type_info, diagnostics, ast, program) = typecheck(source);

  assert!(diagnostics.is_empty());
  let main_block = ast.block(program.main_block(&ast));
  let let_stmt_id = main_block.stmts()[0];
  if let Stmt::LetBinding { var, initializer } = ast.stmt(let_stmt_id)
    && let Some(symbol) = resolution_info.symbol_of(var)
  {
    assert_eq!(type_info.type_of_expr(initializer), Type::Int32);
    assert_eq!(type_info.type_of_symbol(symbol), Some(Type::Int32));
  } else {
    panic!("se esperaba un let binding");
  }
}

#[test]
fn if_expression_without_else_has_unit_type() {
  let source = r#"
    main {
      let x = if true { return 10; };
    }
  "#;
  let (resolution_info, type_info, diagnostics, ast, program) = typecheck(source);

  assert!(diagnostics.is_empty());
  let main_block = ast.block(program.main_block(&ast));
  let let_stmt_id = main_block.stmts()[0];
  if let Stmt::LetBinding { var, initializer } = ast.stmt(let_stmt_id)
    && let Some(symbol) = resolution_info.symbol_of(var)
  {
    assert_eq!(type_info.type_of_expr(initializer), Type::Unit);
    assert_eq!(type_info.type_of_symbol(symbol), Some(Type::Unit));
  } else {
    panic!("se esperaba un let binding");
  }
}

#[test]
fn if_expression_branch_mismatch_produces_error() {
  let source = r#"
    main {
      let x = if true { return 10; } else { return false; };
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);

  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains(&format!(
    "mismatch de tipos: se esperaba {}, pero se encontro {}",
    Type::Int32,
    Type::Bool
  )));
}

#[test]
fn unary_negation_on_int_is_int() {
  let source = r#"
    main {
      let x = -5;
    }
  "#;
  let (_, type_info, diagnostics, ast, program) = typecheck(source);

  assert!(diagnostics.is_empty());
  let stmt = ast.block(program.main_block(&ast)).stmts()[0];
  if let Stmt::LetBinding { initializer, .. } = ast.stmt(stmt) {
    assert_eq!(type_info.type_of_expr(initializer), Type::Int32);
  }
}

#[test]
fn invalid_unary_operation_detected() {
  let source = r#"
    main {
      let x = -true;
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);

  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains(&format!(
    "operacion unaria invalida: {}, el operando es de tipo {}",
    UnaryOp::Neg,
    Type::Bool
  )));
}

#[test]
fn shadowing_preserves_inner_type() {
  let source = r#"
    main {
      let x = 1;
      if true {
        let x = true;
        x;
      }
    }"#;
  let (_, type_info, diagnostics, ast, program) = typecheck(source);

  assert!(diagnostics.is_empty());
  let main_block = ast.block(program.main_block(&ast));
  let if_stmt = ast.stmt(main_block.stmts()[1]);
  if let Stmt::Expr(if_expr_id) = if_stmt
    && let Expr::If(if_expr) = ast.expr(if_expr_id)
  {
    let inner_stmt = ast.block(if_expr.if_block).stmts()[1];
    if let Stmt::Expr(expr_id) = ast.stmt(inner_stmt) {
      assert_eq!(type_info.type_of_expr(expr_id), Type::Bool);
    }
  }
  let outer_stmt = main_block.stmts()[0];
  if let Stmt::Expr(expr_id) = ast.stmt(outer_stmt) {
    assert_eq!(type_info.type_of_expr(expr_id), Type::Int32);
  }
}

#[test]
fn error_does_not_crash_checker() {
  let source = r#"
    main {
      let x = 1 + true;
      let y = x;
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);
  assert!(!diagnostics.is_empty());
}

#[test]
fn binary_op_cases() {
  let source = r#"
    main {
      let a = true - 1;
      let b = 1 == true;
      let c = true != 1;
      let d = 2 < 3;
      let e = true == true;
      let f = true ^^ 2;
      let g = 2 && 3;
      let h = 2 == 2;
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);
  assert_eq!(diagnostics.len(), 5);
}

#[test]
fn block_without_return_has_unit_type() {
  let source = r#"
    main {
      let x = 5;
    }
  "#;
  let (_, type_info, diagnostics, _, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let expr_id = program.main_block_expr();
  assert_eq!(type_info.type_of_expr(expr_id), Type::Unit);
}

#[test]
fn block_with_return_expr_has_expr_type() {
  let source = r#"
    main {
      return 42;
    }
  "#;
  let (_, type_info, diagnostics, _, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let expr_id = program.main_block_expr();
  assert_eq!(type_info.type_of_expr(expr_id), Type::Int32);
}

#[test]
fn return_without_expr_has_unit_type() {
  let source = r#"
    main {
      return;
    }
  "#;
  let (_, type_info, diagnostics, _, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let expr_id = program.main_block_expr();
  assert_eq!(type_info.type_of_expr(expr_id), Type::Unit);
}

#[test]
fn block_expression_type_flows_into_let() {
  let source = r#"
    main {
      let x = {
        return 10;
      };
    }
  "#;
  let (resolution_info, type_info, diagnostics, ast, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let main_block = ast.block(program.main_block(&ast));
  let let_expr_id = main_block.stmts()[0];
  if let Stmt::LetBinding { var, .. } = ast.stmt(let_expr_id)
    && let Some(symbol) = resolution_info.symbol_of(var)
  {
    assert_eq!(type_info.type_of_symbol(symbol), Some(Type::Int32));
  } else {
    panic!("ocurrio un error");
  }
}

#[test]
fn block_type_mistmatch_produces_error() {
  let source = r#"
    main {
      let x = {
        return true;
      };
      let y = x / 1;
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);
  assert!(!diagnostics.is_empty());
  assert!(diagnostics[0].msg().contains(&format!(
    "operacion binaria invalida: {}, el LHS es de tipo {} y el RHS es de tipo {}",
    BinaryOp::Div,
    Type::Bool,
    Type::Int32
  )));
}

#[test]
fn nested_block_types_propagate_correctly() {
  let source = r#"
    main {
      let x = {
        let y = {
          return 5;
        };
        return y;
      };
    }
  "#;
  let (resolution_info, type_info, diagnostics, ast, program) = typecheck(source);
  assert!(diagnostics.is_empty());
  let main_block = ast.block(program.main_block(&ast));
  let let_expr_id = main_block.stmts()[0];
  if let Stmt::LetBinding { var, .. } = ast.stmt(let_expr_id)
    && let Some(symbol) = resolution_info.symbol_of(var)
  {
    assert_eq!(type_info.type_of_symbol(symbol), Some(Type::Int32));
  } else {
    panic!("ocurrio un error");
  }
}

#[test]
fn const_initialized_with_block_without_value() {
  let source = r#"
    main {
      const x = { return; };
    }
  "#;
  let (_, _, diagnostics, _, _) = typecheck(source);
  assert!(diagnostics.is_empty());
}
