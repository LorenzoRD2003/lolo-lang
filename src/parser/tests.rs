use crate::{
  ast::{BinaryExpr, BinaryOp, ConstValue, Expr, Stmt, UnaryExpr, UnaryOp},
  diagnostics::Diagnostic,
  lexer::Lexer,
  parser::{
    Parser,
    program_parsing::{parse_block, parse_expr, parse_program, parse_stmt},
    token_stream::TokenStream,
  },
};
use proptest::prelude::*;

fn source_to_parser_diagnostics(source: &str) -> Vec<Diagnostic> {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new(source);
  let mut tokens = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut tokens, &mut diagnostics);
  parser.parse_program();
  diagnostics
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
  assert_eq!(ast.expr(expr_id), Expr::Var("x".into()));
}

#[test]
fn parses_unary_minus() {
  let (ast, expr_id) = parse_expr("-x");
  let expr_id = expr_id.expect("expression expected");
  match ast.expr(expr_id) {
    Expr::Unary(unary) => {
      assert_eq!(unary.op, UnaryOp::Neg);
      assert_eq!(ast.expr(unary.operand), Expr::Var("x".into()));
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
  assert_eq!(ast.expr(expr_id), Expr::Var("x".into()));
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
      assert_eq!(ast.expr(or.lhs), Expr::Var("a".into()));
      match ast.expr(or.rhs) {
        Expr::Binary(and) => {
          assert_eq!(and.op, BinaryOp::And);
          assert_eq!(ast.expr(and.lhs), Expr::Var("b".into()));
          assert_eq!(ast.expr(and.rhs), Expr::Var("c".into()));
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
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("a < b < c");
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts, &mut diagnostics);
  let expr = parser.parse_expression();
  assert!(expr.is_some(), "parser should recover");
  assert!(
    !parser.diagnostics().is_empty(),
    "should emit chaining comparison error"
  );
}

#[test]
fn parser_recovers_after_comparison_error() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("a < b < c + d");
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts, &mut diagnostics);
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
      assert!(ast.expr(var).is_var());
      assert!(ast.expr(initializer).is_var());
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
      assert_eq!(ast.expr(var), Expr::Var("x".into()));
      assert!(matches!(ast.expr(expr), Expr::Const(ConstValue::Int32(18))));
    }
    _ => panic!("Expected Let"),
  }
}

#[test]
fn const_span_is_correct() {
  let (ast, stmt_id) = parse_stmt("const x = abc;");
  let stmt_id = stmt_id.unwrap();
  assert_eq!(ast.stmt_span(stmt_id), 0..14);
}

#[test]
fn parse_const_stmt_structure() {
  let (ast, stmt_id) = parse_stmt("const x = abc;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::ConstBinding { var, initializer } => {
      assert!(ast.expr(var).is_var());
      assert!(ast.expr(initializer).is_var());
    }
    _ => panic!("Expected Const"),
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
    Stmt::Return(Some(expr_id)) => {
      assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Bool(false)));
    }
    _ => panic!("Expected Return with some value"),
  }
}

#[test]
fn return_expression() {
  let (ast, stmt_id) = parse_stmt("return a ^^ !b;");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Return(Some(expr_id)) => {
      assert!(matches!(
        ast.expr(expr_id),
        Expr::Binary(BinaryExpr {
          op: BinaryOp::Xor,
          lhs: _,
          rhs: _
        })
      ));
    }
    _ => panic!("Expected Return with some value"),
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
    Stmt::Expr(expr_id) => match ast.expr(expr_id) {
      Expr::If(if_expr) => {
        assert!(ast.expr(if_expr.condition).is_var());
        assert_eq!(ast.block(if_expr.if_block).stmts().len(), 2);
      }
      _ => panic!("Expected If expression"),
    },
    _ => panic!("Expected Expr statement"),
  }
}

#[test]
fn if_condition_expression() {
  let (ast, stmt_id) = parse_stmt("if a != b { c; }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Expr(expr_id) => match ast.expr(expr_id) {
      Expr::If(if_expr) => {
        assert!(matches!(
          ast.expr(if_expr.condition),
          Expr::Binary(BinaryExpr {
            op: BinaryOp::Neq,
            lhs: _,
            rhs: _
          })
        ));
      }
      _ => panic!("Expected If expression"),
    },
    _ => panic!("Expected Expr statement"),
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
    Stmt::Expr(expr_id) => match ast.expr(expr_id) {
      Expr::If(if_expr) => {
        assert!(matches!(
          ast.expr(if_expr.condition),
          Expr::Unary(UnaryExpr {
            op: UnaryOp::Not,
            operand: _
          })
        ));
        assert_eq!(ast.block(if_expr.if_block).stmts().len(), 1);
        let else_expr = if_expr.else_branch.expect("else branch expected");
        match ast.expr(else_expr) {
          Expr::Block(else_block) => assert_eq!(ast.block(else_block).stmts().len(), 3),
          _ => panic!("Expected else block expression"),
        }
      }
      _ => panic!("Expected If expression"),
    },
    _ => panic!("Expected Expr statement"),
  }
}

#[test]
fn parse_else_if_chain_structure() {
  let (ast, stmt_id) = parse_stmt("if a { b; } else if c { d; }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Expr(expr_id) => match ast.expr(expr_id) {
      Expr::If(if_expr) => {
        let else_expr = if_expr.else_branch.expect("else-if branch expected");
        assert!(matches!(ast.expr(else_expr), Expr::If(_)));
      }
      _ => panic!("Expected If expression"),
    },
    _ => panic!("Expected Expr statement"),
  }
}

#[test]
fn parse_if_expression_with_deep_else_if_and_final_else() {
  let (ast, expr_id) = parse_expr("if a { b; } else if c { d; } else { e; }");
  let expr_id = expr_id.expect("if expression expected");
  let Expr::If(root_if) = ast.expr(expr_id) else {
    panic!("Expected If expression")
  };
  let Some(else_if_expr_id) = root_if.else_branch else {
    panic!("Expected else-if branch")
  };
  let Expr::If(else_if) = ast.expr(else_if_expr_id) else {
    panic!("Expected nested If expression")
  };
  let Some(final_else_expr_id) = else_if.else_branch else {
    panic!("Expected final else branch")
  };
  assert!(matches!(ast.expr(final_else_expr_id), Expr::Block(_)));
}

#[test]
fn if_expression_statement_allows_optional_semicolon() {
  let (ast, stmt_id) = parse_stmt("if cond { x; };");
  let stmt_id = stmt_id.expect("statement expected");
  match ast.stmt(stmt_id) {
    Stmt::Expr(expr_id) => assert!(matches!(ast.expr(expr_id), Expr::If(_))),
    _ => panic!("Expected Expr statement"),
  }
}

#[test]
fn invalid_else_branch_in_if_expression_emits_error() {
  let source = r#"
    main {
      if true { a; } else 123;
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics
      .iter()
      .any(|d| d.msg().contains("hubo un token inesperado '123'"))
  );
}

#[test]
fn if_without_else_is_not_if_else() {
  let (ast, stmt_id) = parse_stmt("if abc { def; }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Expr(expr_id) => match ast.expr(expr_id) {
      Expr::If(if_expr) => assert!(if_expr.else_branch.is_none()),
      _ => panic!("Expected If expression"),
    },
    _ => panic!("Expected Expr statement"),
  }
}

#[test]
fn nested_if_parses_correctly() {
  let (ast, stmt_id) = parse_stmt("if a { if b { c; } }");
  let stmt_id = stmt_id.unwrap();
  match ast.stmt(stmt_id) {
    Stmt::Expr(expr_id) => match ast.expr(expr_id) {
      Expr::If(if_expr) => {
        let block = ast.block(if_expr.if_block);
        let stmts = block.stmts();
        assert_eq!(stmts.len(), 1);
        match ast.stmt(stmts[0]) {
          Stmt::Expr(inner_expr) => assert!(matches!(ast.expr(inner_expr), Expr::If(_))),
          _ => panic!("Expected nested If expression statement"),
        }
      }
      _ => panic!("Expected If expression"),
    },
    _ => panic!("Expected Expr statement"),
  }
}

#[test]
fn parse_empty_program() {
  parse_program("main {}");
}

#[test]
fn parse_program_with_statements() {
  let (ast, program) = parse_program("main { a; b; }");
  let block_id = program.main_block(&ast);
  assert_eq!(ast.block(block_id).stmts().len(), 2);
}

#[test]
fn program_span_is_correct() {
  let (ast, program) = parse_program("main { a; }");
  assert_eq!(ast.block_span(program.main_block(&ast)), 5..11);
  assert_eq!(program.span(), 0..11);
}

#[test]
#[should_panic]
fn program_requires_main() {
  parse_program("{ a; }");
}

#[test]
fn let_requires_identifier() {
  let source = "let 123 = 5;";
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new(source);
  let mut tokens = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut tokens, &mut diagnostics);
  parser.parse_statement();
  assert!(
    parser.diagnostics()[0]
      .msg()
      .contains("hubo un token inesperado '123'")
  );
}

#[test]
fn unexpected_eof_in_block() {
  let source = "main { print 1; ";
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics[0].msg().contains("hubo un EOF inesperado"));
}

#[test]
fn missing_semicolon_emits_error() {
  let source = "main { print 1 }";
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(
    diagnostics[0]
      .msg()
      .contains("hubo un token inesperado '}'")
  );
}

#[test]
fn empty_block_expression() {
  let source = r#"
    main {
      let x = {};
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn return_without_expression_in_block_expression() {
  let source = r#"
    main {
      let x = {
        return;
      };
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn statement_after_return_in_block_expression() {
  let source = r#"
    main {
      let x = {
        return 5;
        const y = 10;
      };
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(
    diagnostics[0]
      .msg()
      .contains("se detecto un statement luego de un terminador de bloque")
  )
}

#[test]
fn block_expression_in_binary_operation() {
  let source = r#"
    main {
      let x = { return 5; } + 3;
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn unary_operator_with_block_expression() {
  let source = r#"
    main {
      let x = -{ return 5; };
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn parenthesized_block_expression() {
  let source = r#"
    main {
      let x = ({ return 5; });
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn missing_closing_brace_in_block_expression() {
  let source = r#"
    main {
      let x = {
        return 5;
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(!diagnostics.is_empty());
}

#[test]
fn block_expression_as_condition() {
  let source = r#"
    main {
      if { return true; } {
        return 1;
      }
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn two_consecutive_blocks_in_expression_should_fail() {
  let source = r#"
    main {
      let x = {} {};
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(!diagnostics.is_empty());
}

#[test]
fn deeply_nested_block_expressions() {
  let source = r#"
    main {
      let x = {
        return {
          return {
            return 5;
          };
        };
      };
    }
  "#;
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn main_requires_block_error() {
  let source = "main 42";
  let diagnostics = source_to_parser_diagnostics(source);
  assert!(!diagnostics.is_empty());
  assert!(diagnostics[0].msg().contains("main debe ser un bloque"));
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
      program.push('}');
      parse_program(&program);
    }
}
