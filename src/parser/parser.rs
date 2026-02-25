// El Parser en si mismo.
// Responsabilidad:
// - Implementar algoritmo Pratt parsing
// - Construir AST (o sea los ExprId)
// - Fusionar spans
// - Emitir errores sintacticos (luego el que los muestra es el Renderer de diagnostics)

use crate::{
  ast::{
    ast::{Ast, ExprId, StmtId},
    expr::{BinaryExpr, BinaryOp, ConstValue, Expr, UnaryExpr, UnaryOp, VarId},
    stmt::Stmt,
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
    let (stmt, span) = match token.kind {
      TokenKind::Let => self.parse_let_stmt()?,
      TokenKind::Return => self.parse_return_stmt()?,
      TokenKind::If => self.parse_if_stmt()?,
      TokenKind::Print => self.parse_print_stmt()?,
      _ => {
        let expr_id = self.parse_expression()?;
        (Stmt::Expr(expr_id), self.ast.expr_span(expr_id))
      }
    };
    Some(self.ast.add_stmt(stmt, span))
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
      let Some((lbp, _)) = infix_binding_power(token.kind) else {
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
    let Token { kind, lexeme, span } = token.clone();
    match kind {
      TokenKind::NumberLiteral => {
        let number = lexeme.parse::<i32>().ok()?;
        let expr = Expr::Const(ConstValue::Int(number));
        Some(self.ast.add_expr(expr, span))
      }
      TokenKind::BooleanLiteral => {
        let boolean = lexeme.parse::<bool>().ok()?;
        let expr = Expr::Const(ConstValue::Bool(boolean));
        Some(self.ast.add_expr(expr, span))
      }
      TokenKind::Identifier => {
        let expr = Expr::Var(lexeme.clone());
        Some(self.ast.add_expr(expr, span))
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
            let close_paren_end = rparen_token.span.end;
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
    let (_, rbp) = infix_binding_power(token.kind)?;
    let rhs = self.parse_expr_bp(rbp)?;
    let span_start = self.ast.expr_span(lhs).start;
    let span_end = self.ast.expr_span(rhs).end;

    // los operadores de comparacion son no asociativos en lolo-lang. prohibir a < b < c
    // lo tengo que hacer a mano porque Pratt parsing no lo soporta
    match token.kind {
      kind if kind.is_comparison() => {
        // a < b < c tendriamos `(a < b)`` como expresion de comparacion, y `<` como token de comparacion
        if self.ast.expr(lhs).is_comparison() {
          self.emit_error(&ParserError::ChainedAssociativeOperator(token.clone()))
        }
      }
      _ => {}
    };

    // En cualquier caso, sigo parseando. Esto para mejorar el recovery.
    match token.kind {
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

  fn parse_let_stmt(&mut self) -> Option<(Stmt, Span)> {
    // Consumimos el `let`
    let let_token = self.tokens.bump()?;
    // Ahora deberiamos consumir un identificador. de otro modo estamos en problemas
    // Igualmente, vamos a seguir parseando en vez de panickear
    let token = self.tokens.peek()?;
    let var = match self.tokens.expect(TokenKind::Identifier) {
      Ok(token) => VarId(token.lexeme.clone()),
      Err(err) => {
        self.emit_error(&err);
        VarId("que pongo aca".into())
      }
    };
    // Ahora deberiamos consumir un =, de otro modo estamos en problemas
    if let Err(err) = self.tokens.expect(TokenKind::Equal) {
      self.emit_error(&err);
    }


    Some((Stmt::Expr(ExprId(1)), 1..2))
  }

  fn parse_return_stmt(&mut self) -> Option<(Stmt, Span)> {
    todo!()
  }

  fn parse_if_stmt(&mut self) -> Option<(Stmt, Span)> {
    todo!()
  }

  fn parse_print_stmt(&mut self) -> Option<(Stmt, Span)> {
    todo!()
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

  #[test]
  fn parses_number_literal() {
    let (ast, expr_id) = parse_expr("123");
    let expr_id = expr_id.expect("expression expected");
    assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Int(123)));
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
    assert_eq!(ast.expr(expr_id), Expr::Const(ConstValue::Int(123)));
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

  proptest! {
    #[test]
    fn parser_never_panics(bytes in proptest::collection::vec(0u8..=127u8, 0..50)) {
      let input = String::from_utf8(bytes).unwrap_or_default();
      let _ = parse_expr(&input);
    }
  }

  #[test]
  fn multiplication_has_higher_precedence_than_addition() {
    let (ast, expr_id) = parse_expr("1 + 2 * 3");
    let expr_id = expr_id.unwrap();

    match ast.expr(expr_id) {
      Expr::Binary(add) => {
        assert_eq!(add.op, BinaryOp::Add);
        assert_eq!(ast.expr(add.lhs), Expr::Const(ConstValue::Int(1)));
        match ast.expr(add.rhs) {
          Expr::Binary(mul) => {
            assert_eq!(mul.op, BinaryOp::Mul);
            assert_eq!(ast.expr(mul.lhs), Expr::Const(ConstValue::Int(2)));
            assert_eq!(ast.expr(mul.rhs), Expr::Const(ConstValue::Int(3)));
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

  proptest! {
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
  }
}
