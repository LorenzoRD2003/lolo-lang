// El Parser en si mismo.
// Responsabilidad:
// - Implementar algoritmo Pratt parsing
// - Construir AST (o sea los ExprId)
// - Fusionar spans
// - Emitir errores sintacticos (luego el que los muestra es el Renderer de diagnostics)

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, UnaryExpr, UnaryOp, VarId},
    program::Program,
    stmt::{Block, Stmt},
  },
  common::span::Span,
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::token::{Token, TokenKind},
  parser::{
    error::ParserError,
    precedence::ASSIGN_BP,
    token_binding::{infix_binding_power, prefix_binding_power},
    token_stream::TokenStream,
  },
};

#[derive(Debug)]
pub(crate) struct Parser<'a> {
  /// Parser NO habla con Lexer. El lookahead esta centralizado en el TokenStream. Handlea EOF.
  tokens: &'a mut TokenStream<'a>,
  /// El parser es quien construye nodos, fusiona coherementemente los spans del Ast arena-based
  ast: Ast,
  /// Para acumular los errores. Podemos devolver un AST parcial valido
  diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
  pub(crate) fn new(tokens: &'a mut TokenStream<'a>) -> Self {
    Self {
      tokens,
      ast: Ast::empty(),
      diagnostics: Vec::new(),
    }
  }

  /// Esta funcion es el punto de entrada cuando queremos parsear una expresion
  /// Luego de generarse la expresion, se guarda en el AST obteniendose un ExprId, y se lo devuelve.
  pub(crate) fn parse_expression(&mut self) -> Option<ExprId> {
    // Internamente llama a parse_expr_bp(...)
    self.parse_expr_bp(0)
  }

  /// Funcion de punto de entrada para parsear un Statement. Se introduce la expresion en el AST mediante su StmtId
  pub(crate) fn parse_statement(&mut self) -> Option<StmtId> {
    // decidir que tipo de statement es.
    // uso peek() en vez de bump() porque el bump() lo uso al parsear en cada branch
    let token = self.tokens.peek()?;
    let (stmt, span) = match token.kind() {
      TokenKind::Let => self.parse_let_stmt()?,
      TokenKind::Return => self.parse_return_stmt()?,
      TokenKind::If => self.parse_if_stmt()?,
      TokenKind::Print => self.parse_print_stmt()?,
      _ => {
        let expr_id = self.parse_expression()?;
        self.expect_token(TokenKind::Semicolon);
        (Stmt::Expr(expr_id), self.ast.expr_span(expr_id))
      }
    };
    Some(self.ast.add_stmt(stmt, span))
  }

  pub(crate) fn parse_block(&mut self) -> Option<BlockId> {
    // block ::== { stmt* }
    let mut block = Block::new();
    let lbrace = self.tokens.expect(TokenKind::LCurlyBrace).ok()?;
    let span_start = lbrace.span().start;
    loop {
      // el bloque termina cuando encontramos el `}` correspondiente, o con un error si se teremina el archivo
      let token = self.tokens.peek()?.clone();
      let token_kind = token.kind();
      if matches!(token_kind, TokenKind::EOF) {
        self.emit_error(&ParserError::UnexpectedEOF(token));
      }
      if matches!(token_kind, TokenKind::RCurlyBrace | TokenKind::EOF) {
        break;
      }
      // hay que parsear un statement
      let stmt = self.parse_statement()?;
      block.add_stmt(stmt);
    }
    // hay que avanzar una ultima vez porque estabamos haciendo peek()
    let rbrace = self.tokens.expect(TokenKind::RCurlyBrace).ok()?;
    let span_end = rbrace.span().end;
    Some(self.ast.add_block(block, span_start..span_end))
  }

  pub(crate) fn parse_program(&mut self) -> Option<Program> {
    // program ::= main block
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::Main);
    let block = self.parse_block()?;
    let span_end = self.ast.block_span(block).end;
    Some(Program::new(block, span_start..span_end))
  }

  /// El metodo esencial del Pratt parsing. Responsable de:
  /// - Prefix (nud)
  /// - Loop infix (led) -> Binding powers, Mergear los spans
  /// - Error handling
  /// "Parseame una expresion cuya precedencia minima sea `min_bp`"
  fn parse_expr_bp(&mut self, min_bp: u8) -> Option<ExprId> {
    let mut lhs = self.parse_prefix()?;
    // Mientras el siguiente token pueda extender la expresion...
    // o el token no es operador, o el binding power es insuficiente
    loop {
      let token = self.tokens.peek()?;
      let Some((lbp, _)) = infix_binding_power(token.kind()) else {
        break;
      };
      // este operador puede pegarse al lhs?
      if lbp < min_bp {
        break;
      }
      let token = self.tokens.bump()?;
      lhs = self.parse_infix(lhs, token)?;
    }
    Some(lhs)
  }

  /// Es el nud. Construye el LHS inicial.
  /// Maneja literales, identifiers, expresiones con parentesis, operadores unarios.
  fn parse_prefix(&mut self) -> Option<ExprId> {
    let token = self.tokens.bump()?;
    let (kind, lexeme, span) = (token.kind(), token.lexeme(), token.span());
    match kind {
      TokenKind::NumberLiteral => {
        let number = lexeme.parse::<i32>().ok()?;
        let expr = Expr::Const(ConstValue::Int32(number));
        Some(self.ast.add_expr(expr, span.clone()))
      }
      TokenKind::BooleanLiteral => {
        let boolean = lexeme.parse::<bool>().ok()?;
        let expr = Expr::Const(ConstValue::Bool(boolean));
        Some(self.ast.add_expr(expr, span.clone()))
      }
      TokenKind::Identifier => {
        let expr = Expr::Var(VarId(lexeme.into()));
        Some(self.ast.add_expr(expr, span.clone()))
      }
      TokenKind::LParen => {
        // Consumir `(` (con el bump ya lo hicimos)
        // Llamar recursivamente
        let inner_expr_id = self.parse_expr_bp(ASSIGN_BP)?;
        // Esperar hasta `)` OBLIGATORIAMENTE
        match self.tokens.expect(TokenKind::RParen) {
          Ok(rparen_token) => {
            // span(open_paren.start -> close_paren.end)
            let open_paren_start = span.start;
            let close_paren_end = rparen_token.span().end;
            // debo actualizar el span del nodo existente
            Some(
              self
                .ast
                .update_expr_span(inner_expr_id, open_paren_start..close_paren_end),
            )
          }
          Err(err) => {
            // Si falta `)`, emito un error y retorno inner_expr igualmente (recovery minima)
            self.emit_error(&err);
            Some(inner_expr_id)
          }
        }
      }
      kind if kind.is_unary() => {
        // Consumir el operador `!` o `-` (con el bump ya lo hicimos)
        let rhs_id = self.parse_expr_bp(prefix_binding_power(kind)?)?;
        // el span es op.start .. rhs.end
        let op_start = span.start;
        let expr = Expr::Unary(UnaryExpr {
          op: UnaryOp::from_token(&token)?,
          operand: rhs_id,
        });
        let rhs_end = self.ast.expr_span(rhs_id).end;
        Some(self.ast.add_expr(expr, op_start..rhs_end))
      }
      _ => {
        self.emit_error(&ParserError::UnexpectedToken(token));
        None
      }
    }
  }

  /// Es el led. Responsable de construir los nodos binarios, mergear los spans, parsear RHS.
  fn parse_infix(&mut self, lhs: ExprId, token: Token) -> Option<ExprId> {
    let (_, rbp) = infix_binding_power(token.kind())?;
    let rhs = self.parse_expr_bp(rbp)?;
    let span_start = self.ast.expr_span(lhs).start;
    let span_end = self.ast.expr_span(rhs).end;

    // los operadores de comparacion son no asociativos en lolo-lang. prohibir a < b < c
    // lo tengo que hacer a mano porque Pratt parsing no lo soporta
    match token.kind() {
      kind if kind.is_comparison() => {
        // a < b < c tendriamos `(a < b)`` como expresion de comparacion, y `<` como token de comparacion
        if self.ast.expr(lhs).is_comparison() {
          self.emit_error(&ParserError::ChainedAssociativeOperator(token.clone()))
        }
      }
      _ => {}
    };

    // En cualquier caso, sigo parseando. Esto para mejorar el recovery.
    match token.kind() {
      kind if kind.is_binary() => {
        let expr = Expr::Binary(BinaryExpr {
          op: BinaryOp::from_token(&token)?,
          lhs,
          rhs,
        });
        Some(self.ast.add_expr(expr, span_start..span_end))
      }
      _ => return None,
    }
  }

  /// Convierte el error a Diagnostic, lo acumula en la lista de errores. El parser continua.
  fn emit_error(&mut self, err: &ParserError) {
    self.diagnostics.push(err.to_diagnostic())
  }

  fn expect_token(&mut self, kind: TokenKind) {
    if let Err(err) = self.tokens.expect(kind) {
      self.emit_error(&err);
    }
  }

  fn parse_let_stmt(&mut self) -> Option<(Stmt, Span)> {
    // let <var> = <expr>;
    // Consumimos el `let`
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::Let);
    // Ahora deberiamos consumir un identificador
    // Igualmente, vamos a seguir parseando en vez de panickear
    let token = self.tokens.peek()?;
    let var = VarId(token.lexeme().into());
    self.expect_token(TokenKind::Identifier);
    // Ahora deberiamos consumir un `=`
    self.expect_token(TokenKind::Equal);
    // Ahora vamos a parsear una expresion
    let expr_id = self.parse_expression()?;
    // Ahora deberiamos consumir un `;`
    let span_end = self.tokens.peek()?.span().end;
    self.expect_token(TokenKind::Semicolon);
    Some((
      Stmt::Let {
        name: var,
        initializer: expr_id,
      },
      span_start..span_end,
    ))
  }

  fn parse_return_stmt(&mut self) -> Option<(Stmt, Span)> {
    // return <expr>;
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::Return);
    let expr_id = self.parse_expression()?;
    let span_end = self.tokens.peek()?.span().end;
    self.expect_token(TokenKind::Semicolon);
    Some((Stmt::Return(expr_id), span_start..span_end))
  }

  fn parse_if_stmt(&mut self) -> Option<(Stmt, Span)> {
    // if expr { block } [ else { block } ]
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::If);
    let condition = self.parse_expression()?;
    // parsear el bloque if
    let if_block = self.parse_block()?;
    let mut span_end = self.ast.block_span(if_block).end;
    // si hay un bloque else, lo tengo que parsear
    if let Some(token) = self.tokens.peek()
      && matches!(token.kind(), TokenKind::Else)
    {
      self.expect_token(TokenKind::Else);
      // parsear el bloque else
      let else_block = self.parse_block()?;
      span_end = self.ast.block_span(else_block).end;
      Some((
        Stmt::IfElse {
          condition,
          if_block,
          else_block,
        },
        span_start..span_end,
      ))
    } else {
      Some((
        Stmt::If {
          condition,
          if_block,
        },
        span_start..span_end,
      ))
    }
  }

  fn parse_print_stmt(&mut self) -> Option<(Stmt, Span)> {
    // print <expr>;
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::Print);
    let expr_id = self.parse_expression()?;
    let span_end = self.tokens.peek()?.span().end;
    self.expect_token(TokenKind::Semicolon);
    Some((Stmt::Print(expr_id), span_start..span_end))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::lexer::Lexer;
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
      !parser.diagnostics.is_empty(),
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
    assert!(!parser.diagnostics.is_empty());
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
      Stmt::Let { name, initializer } => {
        assert_eq!(name, VarId("x".into()));
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
      Stmt::Let { initializer, .. } => {
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
    let stmts = ast.block(block_id).stmts();
    assert!(matches!(ast.stmt(stmts[0]), Stmt::Let { .. }));
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
        let stmts = ast.block(if_block).stmts();
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
}
