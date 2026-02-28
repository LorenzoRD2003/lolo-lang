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
fn if_condition_must_be_bool() {
  let mut ast = Ast::empty();
  let condition = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let if_block = ast.add_block(Block::new(), 3..4);
  let stmt = ast.add_stmt(
    Stmt::If {
      condition,
      if_block,
    },
    0..2,
  );
  let outer_block = ast.add_block(Block::with_stmts(vec![stmt]), 0..4);
  let program = Program::new(outer_block, 0..4);

  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("mismatch de tipos"))
  );
}

#[test]
fn block_terminator_is_last_statement() {
  let mut ast = Ast::empty();
  let e1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
  let e2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
  let s2 = ast.add_stmt(Stmt::Expr(e2), 2..3);
  let block = ast.add_block(Block::with_stmts(vec![s1, s2]), 0..3);
  let program = Program::new(block, 0..3);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  let block_info = sem.semantic_info.block_info(block);
  assert_eq!(block_info.terminator(), Some(s2));
}

#[test]
fn analyze_program_analyzes_main_block() {
  let mut ast = Ast::empty();
  let expr = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 0..1);
  let stmt = ast.add_stmt(Stmt::Expr(expr), 0..1);
  let main_block = ast.add_block(Block::with_stmts(vec![stmt]), 0..1);
  let program = Program::new(main_block, 0..1);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert_eq!(
    sem.semantic_info.block_info(main_block).terminator(),
    Some(stmt)
  );
}

#[test]
fn let_cannot_use_variable_in_its_own_initializer() {
  // let x = x + 1;
  let mut ast = Ast::empty();
  let var = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
  let lhs = ast.add_expr(Expr::Var(VarId("x".into())), 8..9);
  let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 12..13);
  let init = ast.add_expr(
    Expr::Binary(BinaryExpr {
      op: BinaryOp::Add,
      lhs,
      rhs,
    }),
    8..13,
  );
  let stmt = ast.add_stmt(
    Stmt::LetBinding {
      var,
      initializer: init,
    },
    0..14,
  );
  let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..14);
  let program = Program::new(block, 0..14);
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
fn if_condition_with_unresolved_variable() {
  // if y { }
  let mut ast = Ast::empty();
  let condition = ast.add_expr(Expr::Var(VarId("xyz".into())), 0..1);
  let if_block = ast.add_block(Block::new(), 3..4);
  let stmt = ast.add_stmt(
    Stmt::If {
      condition,
      if_block,
    },
    0..4,
  );
  let outer = ast.add_block(Block::with_stmts(vec![stmt]), 0..4);
  let program = Program::new(outer, 0..4);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("variable 'xyz' indefinida"))
  );
}

#[test]
fn if_else_with_invalid_condition_still_analyzes_blocks() {
  // if 1 { let x = 2; } else { let y = 3; }
  let mut ast = Ast::empty();
  let v_if = ast.add_expr(Expr::Var(VarId("x".into())), 8..9);
  let i_if = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 12..13);
  let s_if = ast.add_stmt(
    Stmt::LetBinding {
      var: v_if,
      initializer: i_if,
    },
    8..13,
  );
  let v_else = ast.add_expr(Expr::Var(VarId("y".into())), 22..23);
  let i_else = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 26..27);
  let s_else = ast.add_stmt(
    Stmt::LetBinding {
      var: v_else,
      initializer: i_else,
    },
    22..27,
  );
  let condition = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 3..4);
  let if_block = ast.add_block(Block::with_stmts(vec![s_if]), 6..15);
  let else_block = ast.add_block(Block::with_stmts(vec![s_else]), 20..30);
  let if_stmt = ast.add_stmt(
    Stmt::IfElse {
      condition,
      if_block,
      else_block,
    },
    0..30,
  );
  let main = ast.add_block(Block::with_stmts(vec![if_stmt]), 0..30);
  let program = Program::new(main, 0..30);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);

  // Debe haber error de tipo en la condición
  assert!(
    sem
      .diagnostics()
      .iter()
      .any(|d: &Diagnostic| d.msg().contains("mismatch de tipos"))
  );

  // Pero igual deben declararse x e y
  let if_info = sem.semantic_info.stmt_info(s_if);
  let else_info = sem.semantic_info.stmt_info(s_else);
  assert!(if_info.symbol_declared().is_some());
  assert!(else_info.symbol_declared().is_some());
}

// #[test]
// fn print_requires_value_expression() {
//   // Esto para cuando agreguemos alguna expresion que no sea ValueExpr
// }

#[test]
fn print_constant_keeps_constant_info() {
  // print(42);
  let mut ast = Ast::empty();
  let expr = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 6..8);
  let stmt = ast.add_stmt(Stmt::Print(expr), 0..8);
  let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..8);
  let program = Program::new(block, 0..8);
  let mut sem = semantic_analyzer(&ast);
  sem.analyze_program(program);
  let info = sem.semantic_info.expr_info(expr);
  assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(42)));
  assert!(sem.diagnostics().is_empty());
}
