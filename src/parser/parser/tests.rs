use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, UnaryExpr, UnaryOp, VarId},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::Diagnostic,
  lexer::lexer::Lexer,
  parser::{parser::Parser, precedence::ASSIGN_BP, token_stream::TokenStream},
};
use proptest::prelude::*;

fn parse_expr(input: &str) -> (Ast, Option<ExprId>) {
  let mut lexer = Lexer::new(input);
  let mut ts = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut ts);
  let expr = parser.parse_expr_bp(ASSIGN_BP);
  (parser.ast, expr)
}

fn parse_stmt(input: &str) -> (Ast, Option<StmtId>) {
  let mut lexer = Lexer::new(input);
  let mut ts = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut ts);
  let stmt = parser.parse_statement();
  (parser.ast, stmt)
}

fn parse_block(input: &str) -> (Ast, Option<BlockId>) {
  let mut lexer = Lexer::new(input);
  let mut ts = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut ts);
  let block = parser.parse_block();
  (parser.ast, block)
}

fn parse_program(input: &str) -> (Ast, Option<Program>) {
  let mut lexer = Lexer::new(input);
  let mut ts = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut ts);
  let program = parser.parse_program();
  (parser.ast, program)
}

#[test]
fn parses_number_literal() {
  let (ast, expr_id) = parse_expr("123");
  let expr_id = expr_id.expect("expression expected");
  assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Int32(123)));
}

#[test]
fn parses_boolean_literal() {
  let (ast, expr_id) = parse_expr("true");
  let expr_id = expr_id.expect("expression expected");
  assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Bool(true)));
}

#[test]
fn parses_identifier() {
  let (ast, expr_id) = parse_expr("x");
  let expr_id = expr_id.expect("expression expected");
  assert_eq!(ast.expr(expr_id), Expr::Var(VarId("x".into())));
}

#[test]
fn parses_unary_minus() {
  let (ast, expr_id) = parse_expr("-x");
  let expr_id = expr_id.expect("expression expected");
  match ast.expr(expr_id) {
    Expr::Unary(unary) => {
      assert_eq!(unary.op, UnaryOp::Neg);
      assert_eq!(ast.expr(unary.operand), Expr::Var(VarId("x".into())));
    }
    _ => panic!("expected unary expression"),
  }
}

#[test]
fn parses_unary_not() {
  let (ast, expr_id) = parse_expr("!true");
  let expr_id = expr_id.expect("expression expected");
  match ast.expr(expr_id) {
    Expr::Unary(unary) => {
      assert_eq!(unary.op, UnaryOp::Not);
      assert_eq!(ast.expr(unary.operand), Expr::Const(ConstValue::Bool(true)));
    }
    _ => panic!("expected unary expression"),
  }
}

#[test]
fn unary_span_is_correct() {
  let (ast, expr_id) = parse_expr("-abc");
  let expr_id = expr_id.unwrap();
  assert_eq!(ast.expr_span(expr_id), 0..4); // "-abc" -> span completo
}

#[test]
fn parses_parenthesized_expression() {
  let (ast, expr_id) = parse_expr("(123)");
  let expr_id = expr_id.expect("expression expected");
  // Clave, no crea un nodo adicional
  assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Int32(123)));
}

#[test]
fn parenthesis_not_closed_returns_inner_expr() {
  let (ast, expr_id) = parse_expr("(123;");
  let expr_id = expr_id.expect("expression expected");
  assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Int32(123))); // recovery igualmente
}

#[test]
fn parses_nested_parens() {
  let (ast, expr_id) = parse_expr("(((x)))");
  let expr_id = expr_id.expect("expression expected");
  assert_eq!(ast.expr(expr_id), Expr::Var(VarId("x".into())));
}

#[test]
fn paren_span_is_correct() {
  let (ast, expr_id) = parse_expr("(xy)");
  let expr_id = expr_id.unwrap();
  assert_eq!(ast.expr_span(expr_id), 0..4); // "(xy)" -> span completo
}

#[test]
fn multiplication_has_higher_precedence_than_addition() {
  let (ast, expr_id) = parse_expr("1 + 2 * 3");
  let expr_id = expr_id.unwrap();

  match ast.expr(expr_id) {
    Expr::Binary(add) => {
      assert_eq!(add.op, BinaryOp::Add);
      assert_eq!(ast.expr(add.lhs), Expr::Const(ConstValue::Int32(1)));
      match ast.expr(add.rhs) {
        Expr::Binary(mul) => {
          assert_eq!(mul.op, BinaryOp::Mul);
          assert_eq!(ast.expr(mul.lhs), Expr::Const(ConstValue::Int32(2)));
          assert_eq!(ast.expr(mul.rhs), Expr::Const(ConstValue::Int32(3)));
        }
        _ => panic!("expected multiplication on RHS"),
      }
    }
    _ => panic!("expected addition"),
  }
}

#[test]
fn addition_is_left_associative() {
  let (ast, expr_id) = parse_expr("1 - 2 - 3");
  let expr_id = expr_id.unwrap();
  match ast.expr(expr_id) {
    Expr::Binary(outer) => {
      assert_eq!(outer.op, BinaryOp::Sub);
      match ast.expr(outer.lhs) {
        Expr::Binary(inner) => {
          assert_eq!(inner.op, BinaryOp::Sub);
        }
        _ => panic!("expected left associativity"),
      }
    }
    _ => panic!("expected binary"),
  }
}

#[test]
fn parentheses_override_precedence() {
  let (ast, expr_id) = parse_expr("(1 + 2) * 3");
  let expr_id = expr_id.unwrap();
  match ast.expr(expr_id) {
    Expr::Binary(mul) => {
      assert_eq!(mul.op, BinaryOp::Mul);
      match ast.expr(mul.lhs) {
        Expr::Binary(add) => {
          assert_eq!(add.op, BinaryOp::Add);
        }
        _ => panic!("expected grouped addition"),
      }
    }
    _ => panic!("expected multiplication"),
  }
}

#[test]
fn unary_binds_tighter_than_multiplication() {
  let (ast, expr_id) = parse_expr("-x * y");
  let expr_id = expr_id.unwrap();
  match ast.expr(expr_id) {
    Expr::Binary(mul) => {
      assert_eq!(mul.op, BinaryOp::Mul);
      match ast.expr(mul.lhs) {
        Expr::Unary(_) => {}
        _ => panic!("expected unary on lhs"),
      }
    }
    _ => panic!("expected multiplication"),
  }
}

#[test]
fn logical_and_has_higher_precedence_than_or() {
  let (ast, expr_id) = parse_expr("a || b && c");
  let expr_id = expr_id.unwrap();
  match ast.expr(expr_id) {
    Expr::Binary(or) => {
      assert_eq!(or.op, BinaryOp::Or);
      assert_eq!(ast.expr(or.lhs), Expr::Var(VarId("a".into())));
      match ast.expr(or.rhs) {
        Expr::Binary(and) => {
          assert_eq!(and.op, BinaryOp::And);
          assert_eq!(ast.expr(and.lhs), Expr::Var(VarId("b".into())));
          assert_eq!(ast.expr(and.rhs), Expr::Var(VarId("c".into())));
        }
        _ => panic!("expected AND on RHS"),
      }
    }
    _ => panic!("expected OR at top level"),
  }
}

#[test]
fn binary_span_is_correct() {
  let (ast, expr_id) = parse_expr("abc + def");
  let expr_id = expr_id.unwrap();
  assert_eq!(ast.expr_span(expr_id), 0..9);
}

#[test]
fn complex_span_is_correct() {
  let (ast, expr_id) = parse_expr("(a + b) * c");
  let expr_id = expr_id.unwrap();
  assert_eq!(ast.expr_span(expr_id), 0..11);
}

#[test]
fn parses_comparison() {
  let (ast, expr_id) = parse_expr("a < b");
  let expr_id = expr_id.unwrap();
  match ast.expr(expr_id) {
    Expr::Binary(bin) => {
      assert_eq!(bin.op, BinaryOp::Lt);
    }
    _ => panic!("expected comparison"),
  }
}

#[test]
fn comparison_is_not_associative() {
  let mut lexer = Lexer::new("a < b < c");
  let mut ts = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut ts);
  let expr = parser.parse_expression();
  assert!(expr.is_some(), "parser should recover");
  assert!(
    !parser.diagnostics().is_empty(),
    "should emit chaining comparison error"
  );
}

#[test]
fn parser_recovers_after_comparison_error() {
  let mut lexer = Lexer::new("a < b < c + d");
  let mut ts = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut ts);
  let expr = parser.parse_expression();
  assert!(expr.is_some());
  assert!(!parser.diagnostics().is_empty());
}

#[test]
fn let_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("let x = abc;");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..12);
}

#[test]
fn parse_let_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("let x = abc;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::LetBinding { var, initializer } => {
      assert_eq!(ast.expr(var), Expr::Var(VarId("x".into())));
      assert!(matches!(ast.expr(initializer), Expr::Var(_)));
    }
    _ => panic!("Expected Let"),
  }
}

#[test]
fn let_initializer_expression() {
  let (ast, stmt_id) = parse_stmt("let x = a + b;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::LetBinding { initializer, .. } => {
      assert!(matches!(
        ast.expr(initializer),
        Expr::Binary(BinaryExpr {
          op: BinaryOp::Add,
          lhs: _,
          rhs: _
        })
      ));
    }
    _ => panic!("Expected Let"),
  }
}

#[test]
fn assign_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("x = abc;");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..8);
}

#[test]
fn parse_assign_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("x = 18;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Assign {
      var,
      value_expr: expr,
    } => {
      assert_eq!(ast.expr(var), Expr::Var(VarId("x".into())));
      assert!(matches!(ast.expr(expr), Expr::Const(ConstValue::Int32(18))));
    }
    _ => panic!("Expected Let"),
  }
}

#[test]
fn print_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("print abc;");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..10);
}

#[test]
fn parse_print_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("print 123;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Print(expr_id) => {
      assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Int32(123)));
    }
    _ => panic!("Expected Print"),
  }
}

#[test]
fn print_expression() {
  let (ast, stmt_id) = parse_stmt("print a * b;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Print(expr_id) => {
      assert!(matches!(
        ast.expr(expr_id),
        Expr::Binary(BinaryExpr {
          op: BinaryOp::Mul,
          lhs: _,
          rhs: _
        })
      ));
    }
    _ => panic!("Expected Print"),
  }
}

#[test]
fn return_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("return abc;");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..11);
}

#[test]
fn parse_return_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("return false;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Return(expr_id) => {
      assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Bool(false)));
    }
    _ => panic!("Expected Return"),
  }
}

#[test]
fn return_expression() {
  let (ast, stmt_id) = parse_stmt("return a ^^ !b;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Return(expr_id) => {
      assert!(matches!(
        ast.expr(expr_id),
        Expr::Binary(BinaryExpr {
          op: BinaryOp::Xor,
          lhs: _,
          rhs: _
        })
      ));
    }
    _ => panic!("Expected Return"),
  }
}

#[test]
fn empty_block_span_is_correct() {
  let (ast, block_id) = parse_block("{}");
  let block_id = block_id.unwrap();
  assert_eq!(ast.block_span(block_id), 0..2);
}

#[test]
fn block_span_is_correct() {
  let (ast, block_id) = parse_block("{ a; b; }");
  let block_id = block_id.unwrap();
  assert_eq!(ast.block_span(block_id), 0..9);
}

#[test]
fn block_stmt_count() {
  let (ast, block_id) = parse_block("{ a; b; c; }");
  let block_id = block_id.unwrap();
  assert_eq!(ast.block(block_id).stmts().len(), 3);
}

#[test]
fn block_stmt_order_is_preserved() {
  let (ast, block_id) = parse_block("{ let a = 1; print b; return x; }");
  let block_id = block_id.unwrap();
  let block = ast.block(block_id);
  let stmts = block.stmts();
  assert!(matches!(ast.stmt(stmts[0]), Stmt::LetBinding { .. }));
  assert!(matches!(ast.stmt(stmts[1]), Stmt::Print(_)));
  assert!(matches!(ast.stmt(stmts[2]), Stmt::Return(_)));
}

#[test]
fn if_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("if abc { def; }");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..15);
}

#[test]
fn parse_if_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("if abc { def; fg; }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::If {
      condition,
      if_block,
    } => {
      assert!(matches!(ast.expr(condition), Expr::Var(VarId(_))));
      assert_eq!(ast.block(if_block).stmts().len(), 2);
    }
    _ => panic!("Expected If"),
  }
}

#[test]
fn if_condition_expression() {
  let (ast, stmt_id) = parse_stmt("if a != b { c; }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::If { condition, .. } => {
      assert!(matches!(
        ast.expr(condition),
        Expr::Binary(BinaryExpr {
          op: BinaryOp::Neq,
          lhs: _,
          rhs: _
        })
      ));
    }
    _ => panic!("Expected If"),
  }
}

#[test]
fn if_else_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("if abc { def; } else { ghi; }");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..29);
}

#[test]
fn parse_if_else_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("if !x { def; } else { ghi; jkl; mno; }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::IfElse {
      condition,
      if_block,
      else_block,
    } => {
      assert!(matches!(
        ast.expr(condition),
        Expr::Unary(UnaryExpr {
          op: UnaryOp::Not,
          operand: _
        })
      ));
      assert_eq!(ast.block(if_block).stmts().len(), 1);
      assert_eq!(ast.block(else_block).stmts().len(), 3);
    }
    _ => panic!("Expected IfElse"),
  }
}

#[test]
fn if_without_else_is_not_if_else() {
  let (ast, stmt_id) = parse_stmt("if abc { def; }");
  let stmt_id = stmt_id.unwrap();
  assert!(matches!(ast.stmt(stmt_id), Stmt::If { .. }));
}

#[test]
fn nested_if_parses_correctly() {
  let (ast, stmt_id) = parse_stmt("if a { if b { c; } }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::If { if_block, .. } => {
      let block = ast.block(if_block);
      let stmts = block.stmts();
      assert_eq!(stmts.len(), 1);
      assert!(matches!(ast.stmt(stmts[0]), Stmt::If { .. }));
    }
    _ => panic!("Expected If"),
  }
}

#[test]
fn parse_empty_program() {
  let (_ast, program) = parse_program("main {}");
  assert!(program.is_some());
}

#[test]
fn parse_program_with_statements() {
  let (ast, program) = parse_program("main { a; b; }");
  let block_id = program.unwrap().main_block();
  assert_eq!(ast.block(block_id).stmts().len(), 2);
}

#[test]
fn program_span_is_correct() {
  let (ast, program_opt) = parse_program("main { a; }");
  let program = program_opt.unwrap();
  assert_eq!(ast.block_span(program.main_block()), 5..11);
  assert_eq!(program.span(), 0..11);
}

#[test]
fn program_requires_main() {
  let (_ast, program) = parse_program("{ a; }");
  assert!(program.is_none());
}

#[test]
fn let_requires_identifier() {
  let source = "let 123 = 5;";
  let mut lexer = Lexer::new(source);
  let mut tokens = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut tokens);
  parser.parse_statement();
  assert!(parser.diagnostics().iter().any(|d: &Diagnostic| {
    d.msg()
      .contains("se esperaba un identificador de variable, pero se encontro '123'")
  }));
}

#[test]
fn unexpected_eof_in_block() {
  let source = "main { print 1; ";
  let mut lexer = Lexer::new(source);
  let mut tokens = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut tokens);
  parser.parse_program();
  assert!(
    parser
      .diagnostics()
      .iter()
      .any(|d| d.msg().contains("hubo un EOF inesperado"))
  );
}

#[test]
fn missing_semicolon_emits_error() {
  let source = "main { print 1 }";
  let mut lexer = Lexer::new(source);
  let mut tokens = TokenStream::new(&mut lexer);
  let mut parser = Parser::new(&mut tokens);
  parser.parse_program();
  assert!(
    parser
      .diagnostics
      .iter()
      .any(|d| d.msg().contains("hubo un token inesperado '}'")),
  );
}

proptest! {
    #[test]
    fn parser_never_panics_and_generates_valid_spans(bytes in proptest::collection::vec(0u8..=127u8, 0..80), a in 0..3) {
      let input = String::from_utf8(bytes).unwrap_or_default();
      if a == 0 {
        let (ast, stmt) = parse_stmt(&input);
        if let Some(stmt_id) = stmt {
          let span = ast.stmt_span(stmt_id);
          prop_assert!(span.start <= span.end);
        }
      } else if a == 1 {
        let (ast, expr) = parse_expr(&input);
        if let Some(expr_id) = expr {
          let span = ast.expr_span(expr_id);
          prop_assert!(span.start <= span.end);
        }
      } else {
        let (ast, block) = parse_block(&input);
        if let Some(block_id) = block {
          let span = ast.block_span(block_id);
          prop_assert!(span.start <= span.end);
        }
      }
    }

    #[test]
    fn binary_expressions_have_valid_spans(
      a in "[a-z]{1,5}",
      b in "[a-z]{1,5}"
    ) {
      let input = format!("{a} + {b}");
      let (ast, expr_id) = parse_expr(&input);
      if let Some(id) = expr_id {
        let span = ast.expr_span(id);
        prop_assert!(span.start == 0);
        prop_assert!(span.end == input.len());
      }
    }

    /// El span siempre cubre exactamente la expresion parseada
    /// Si el parser devuelve un ExprId, entonces el span debe estar contenido dentro del source
    /// y el substring del span NO debe estar vacío
    #[test]
    fn parsed_expression_span_is_never_out_of_bounds(
      bytes in proptest::collection::vec(0u8..=127u8, 0..80)
    ) {
      let input = String::from_utf8(bytes).unwrap_or_default();
      let (ast, expr_id) = parse_expr(&input);
      if let Some(id) = expr_id {
        let span = ast.expr_span(id);
        // Nunca fuera del source
        prop_assert!(span.start <= input.len());
        prop_assert!(span.end <= input.len());
        // Nunca invertido
        prop_assert!(span.start <= span.end);
        // Span no vacio
        prop_assert!(span.end - span.start > 0);
        // Deteccion de spans inutiles
        let snippet = &input[span.clone()];
        prop_assert!(!snippet.trim().is_empty());
      }
    }


    #[test]
    fn randomly_generated_programs_parse(stmt_count in 0usize..20) {
      let mut program = String::from("main {");
      for i in 0..stmt_count {
        let stmt = match i % 4 {
          0 => format!(" a{};", i),
          1 => format!(" let x{} = a{};", i, i),
          2 => format!(" print a{};", i),
          _ => format!(" return a{};", i),
        };
        program.push_str(&stmt);
      }
      program.push_str("}");
      let (_ast, parsed) = parse_program(&program);
      prop_assert!(parsed.is_some());
    }
}
