use crate::{
  ast::{
    ast::Ast,
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, VarId},
    program::Program,
    stmt::{Block, Stmt},
  },
  diagnostics::diagnostic::Diagnostic,
  semantic::analyzer::tests::semantic_analyzer,
};

#[test]
fn analyze_expr_scope_is_current_scope() {
  let mut ast = Ast::empty();
  let expr_id = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let mut sem = semantic_analyzer(&ast);
  let scope_id = sem.current_scope().unwrap();

  sem.analyze_expr(expr_id);
  let info = sem.semantic_info.expr_info(expr_id);
  assert_eq!(info.scope(), scope_id);
}

#[test]
fn let_declares_symbol_in_scope() {
  // let x = 42;
  let mut ast = Ast::empty();
  let var = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
  let init = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 8..10);
  let stmt = ast.add_stmt(
    Stmt::LetBinding {
      var,
      initializer: init,
    },
    0..11,
  );
  let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..11);
  let program = Program::new(block, 0..11);

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  let stmt_info = sem.semantic_info.stmt_info(stmt);
  assert!(stmt_info.symbol_declared().is_some());
  assert!(sem.diagnostics().is_empty());
}

#[test]
fn redeclaration_in_same_scope_is_error() {
  // let x = 1;
  // let x = 2;
  let mut ast = Ast::empty();
  let var1 = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
  let init1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
  let stmt1 = ast.add_stmt(
    Stmt::LetBinding {
      var: var1,
      initializer: init1,
    },
    0..5,
  );

  let var2 = ast.add_expr(Expr::Var(VarId("x".into())), 6..7);
  let init2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 10..11);
  let stmt2 = ast.add_stmt(
    Stmt::LetBinding {
      var: var2,
      initializer: init2,
    },
    6..11,
  );
  let block = ast.add_block(Block::with_stmts(vec![stmt1, stmt2]), 0..11);
  let program = Program::new(block, 0..11);

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);

  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("la variable 'x' ya fue declarada"))
  );
}

#[test]
fn shadowing_in_inner_block_is_allowed() {
  // let x = 1;
  // if x <= 3 { let x = 2; }
  let mut ast = Ast::empty();
  let var_outer = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
  let init_outer = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 8..9);
  let stmt_outer = ast.add_stmt(
    Stmt::LetBinding {
      var: var_outer,
      initializer: init_outer,
    },
    0..10,
  );

  let condition_lhs = ast.add_expr(Expr::Var(VarId("x".into())), 13..14);
  let condition_rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 18..19);
  let condition = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Lte,
      lhs: condition_lhs,
      rhs: condition_rhs,
    }),
    13..19,
  );

  let var_inner = ast.add_expr(Expr::Var(VarId("x".into())), 26..27);
  let init_inner = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 30..31);
  let stmt_inner = ast.add_stmt(
    Stmt::LetBinding {
      var: var_inner,
      initializer: init_inner,
    },
    22..32,
  );
  let if_block = ast.add_block(Block::with_stmts(vec![stmt_inner]), 20..34);
  let stmt_if = ast.add_stmt(
    Stmt::If {
      condition,
      if_block,
    },
    10..19,
  );

  let main_block = ast.add_block(Block::with_stmts(vec![stmt_outer, stmt_if]), 0..34);
  let program = Program::new(main_block, 0..14);

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);

  // No hay errores semanticos
  assert!(sem.diagnostics().is_empty());
  // Ambos lets deben haber declarado un simbolo
  let outer_info = sem.semantic_info.stmt_info(stmt_outer);
  let inner_info = sem.semantic_info.stmt_info(stmt_inner);
  let outer_symbol = outer_info
    .symbol_declared()
    .expect("outer let debe declarar símbolo");
  let inner_symbol = inner_info
    .symbol_declared()
    .expect("inner let debe declarar símbolo");
  // Los simbolos deben ser distintos (shadowing real)
  assert_ne!(outer_symbol, inner_symbol);
  // La condición del if debe referirse al símbolo externo
  dbg!(10);
  let condition_lhs_info = sem.semantic_info.expr_info(condition_lhs);
  let resolved_symbol = condition_lhs_info
    .symbol()
    .expect("la x en la condición debe resolver");
  assert_eq!(resolved_symbol, outer_symbol);
}

#[test]
fn inner_block_variable_not_visible_outside() {
  // if true { let x = 1; }
  // return x;
  let mut ast = Ast::empty();
  let var_inner = ast.add_expr(Expr::Var(VarId("x".into())), 14..15);
  let init_inner = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 18..19);
  let stmt_inner = ast.add_stmt(
    Stmt::LetBinding {
      var: var_inner,
      initializer: init_inner,
    },
    10..20,
  );
  let if_block = ast.add_block(Block::with_stmts(vec![stmt_inner]), 8..22);
  let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 3..7);
  let stmt_if = ast.add_stmt(
    Stmt::If {
      condition,
      if_block,
    },
    0..2,
  );

  let use_x = ast.add_expr(Expr::Var(VarId("x".into())), 29..30);
  let stmt_use = ast.add_stmt(Stmt::Return(use_x), 22..31);
  let main_block = ast.add_block(Block::with_stmts(vec![stmt_if, stmt_use]), 0..31);
  let program = Program::new(main_block, 0..31);

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("variable 'x' indefinida"))
  );
}

#[test]
fn triple_nested_shadowing() {
  // let x = 1;
  // if x == 1 {
  //   let x = 2;
  //   if x == 2 {
  //     let x = 3;
  //   }
  // }
  let mut ast = Ast::empty();
  let v1 = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
  let i1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
  let s1 = ast.add_stmt(
    Stmt::LetBinding {
      var: v1,
      initializer: i1,
    },
    0..5,
  );
  let v2 = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
  let i2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 14..15);
  let s2 = ast.add_stmt(
    Stmt::LetBinding {
      var: v2,
      initializer: i2,
    },
    10..15,
  );
  let v3 = ast.add_expr(Expr::Var(VarId("x".into())), 20..21);
  let i3 = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 24..25);
  let s3 = ast.add_stmt(
    Stmt::LetBinding {
      var: v3,
      initializer: i3,
    },
    20..25,
  );

  let inner_inner_block = ast.add_block(Block::with_stmts(vec![s3]), 18..27);
  let inner_condition_lhs = ast.add_expr(Expr::Var(VarId("x".into())), 16..17);
  let inner_condition_rhs = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 16..17);
  let inner_condition = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Eq,
      lhs: inner_condition_lhs,
      rhs: inner_condition_rhs,
    }),
    16..17,
  );
  let inner_if = ast.add_stmt(
    Stmt::If {
      condition: inner_condition,
      if_block: inner_inner_block,
    },
    16..27,
  );

  let inner_block = ast.add_block(Block::with_stmts(vec![s2, inner_if]), 8..27);
  let outer_condition_lhs = ast.add_expr(Expr::Var(VarId("x".into())), 6..17);
  let outer_condition_rhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 16..7);
  let outer_condition = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Eq,
      lhs: outer_condition_lhs,
      rhs: outer_condition_rhs,
    }),
    6..7,
  );
  let outer_if = ast.add_stmt(
    Stmt::If {
      condition: outer_condition,
      if_block: inner_block,
    },
    6..27,
  );
  let main_block = ast.add_block(Block::with_stmts(vec![s1, outer_if]), 0..27);
  let program = Program::new(main_block, 0..27);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(sem.diagnostics().is_empty());
}

#[test]
fn redeclaration_after_inner_scope_is_error() {
  // let x = 1;
  // if true { let x = 2; }
  // let x = 3;  // error
  let mut ast = Ast::empty();
  let v1 = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
  let i1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
  let s1 = ast.add_stmt(
    Stmt::LetBinding {
      var: v1,
      initializer: i1,
    },
    0..5,
  );
  let v2 = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
  let i2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 14..15);
  let s2 = ast.add_stmt(
    Stmt::LetBinding {
      var: v2,
      initializer: i2,
    },
    10..15,
  );
  let inner_block = ast.add_block(Block::with_stmts(vec![s2]), 8..17);
  let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 6..7);
  let if_stmt = ast.add_stmt(
    Stmt::If {
      condition: condition,
      if_block: inner_block,
    },
    6..17,
  );
  let v3 = ast.add_expr(Expr::Var(VarId("x".into())), 18..19);
  let i3 = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 22..23);
  let s3 = ast.add_stmt(
    Stmt::LetBinding {
      var: v3,
      initializer: i3,
    },
    18..23,
  );
  let main_block = ast.add_block(Block::with_stmts(vec![s1, if_stmt, s3]), 0..23);
  let program = Program::new(main_block, 0..23);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("la variable 'x' ya fue declarada"))
  );
}

#[test]
fn if_else_scopes_are_independent() {
  // if true { let x = 1; }
  // else { let x = 2; }
  let mut ast = Ast::empty();
  let v_if = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
  let i_if = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 14..15);
  let s_if = ast.add_stmt(
    Stmt::LetBinding {
      var: v_if,
      initializer: i_if,
    },
    10..15,
  );
  let v_else = ast.add_expr(Expr::Var(VarId("x".into())), 25..26);
  let i_else = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 29..30);
  let s_else = ast.add_stmt(
    Stmt::LetBinding {
      var: v_else,
      initializer: i_else,
    },
    25..30,
  );
  let if_block = ast.add_block(Block::with_stmts(vec![s_if]), 8..17);
  let else_block = ast.add_block(Block::with_stmts(vec![s_else]), 23..32);
  let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 3..7);
  let stmt = ast.add_stmt(
    Stmt::IfElse {
      condition,
      if_block,
      else_block,
    },
    0..32,
  );
  let main = ast.add_block(Block::with_stmts(vec![stmt]), 0..32);
  let program = Program::new(main, 0..32);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(sem.diagnostics().is_empty());
}

#[test]
fn variable_declared_in_if_not_visible_in_else() {
  // if true { let x = 1; }
  // else { print(x); }
  let mut ast = Ast::empty();
  let v_if = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
  let i_if = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 14..15);
  let s_if = ast.add_stmt(
    Stmt::LetBinding {
      var: v_if,
      initializer: i_if,
    },
    10..15,
  );
  let use_x = ast.add_expr(Expr::Var(VarId("x".into())), 30..31);
  let s_else = ast.add_stmt(Stmt::Print(use_x), 24..32);
  let if_block = ast.add_block(Block::with_stmts(vec![s_if]), 8..17);
  let else_block = ast.add_block(Block::with_stmts(vec![s_else]), 22..34);
  let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 3..7);
  let stmt = ast.add_stmt(
    Stmt::IfElse {
      condition,
      if_block,
      else_block,
    },
    0..34,
  );
  let main = ast.add_block(Block::with_stmts(vec![stmt]), 0..34);
  let program = Program::new(main, 0..34);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("variable 'x' indefinida"))
  );
}
