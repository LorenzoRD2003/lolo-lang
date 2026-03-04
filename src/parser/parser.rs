// El Parser en si mismo.
// Responsabilidad:
// - Implementar algoritmo Pratt parsing
// - Construir AST (o sea los ExprId)
// - Fusionar spans
// - Emitir errores sintacticos (luego el que los muestra es el Renderer de diagnostics)

use crate::{
  ast::{
    Ast, BinaryExpr, BinaryOp, Block, BlockId, ConstValue, Expr, ExprId, Program, Stmt, StmtId,
    UnaryExpr, UnaryOp,
  },
  common::Span,
  diagnostics::{Diagnosable, Diagnostic},
  lexer::{Token, TokenKind},
  parser::{
    error::ParserError,
    precedence::ASSIGN_BP,
    token_binding::{infix_binding_power, prefix_binding_power},
    token_stream::TokenStream,
  },
};

type Spanned<T> = (T, Span);
type ParseResult<T> = Option<Spanned<T>>;

#[derive(Debug)]
pub(crate) struct Parser<'a> {
  /// Parser NO habla con Lexer. El lookahead esta centralizado en el TokenStream. Handlea EOF.
  tokens: &'a mut TokenStream<'a>,
  /// El parser es quien construye nodos, fusiona coherementemente los spans del Ast arena-based
  ast: Ast,
  /// Para acumular los errores. Podemos devolver un AST parcial valido
  diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
  pub(crate) fn new(tokens: &'a mut TokenStream<'a>, diagnostics: &'a mut Vec<Diagnostic>) -> Self {
    Self {
      tokens,
      ast: Ast::empty(),
      diagnostics,
    }
  }

  /// Esta funcion es el punto de entrada cuando queremos parsear una expresion
  /// Luego de generarse la expresion, se guarda en el AST obteniendose un ExprId, y se lo devuelve.
  pub(crate) fn parse_expression(&mut self) -> Option<ExprId> {
    // Internamente llama a parse_expr_bp(...)
    self.parse_expr_bp(0)
  }

  /// El metodo esencial del Pratt parsing. Responsable de:
  /// - Prefix (nud)
  /// - Loop infix (led) -> Binding powers, Mergear los spans
  /// - Error handling
  /// "Parseame una expresion cuya precedencia minima sea `min_bp`"
  fn parse_expr_bp(&mut self, min_bp: u8) -> Option<ExprId> {
    let mut lhs = self.parse_prefix()?;
    // si es una asignacion, la devolvemos
    if self.tokens.peek_first(self.diagnostics)?.kind() == TokenKind::Equal {
      return Some(lhs);
    }
    // Mientras el siguiente token pueda extender la expresion...
    // o el token no es operador, o el binding power es insuficiente
    let mut i = 5;
    loop {
      i = i + 1;
      let token = self.tokens.peek_first(self.diagnostics)?;
      let Some((lbp, _)) = infix_binding_power(token.kind()) else {
        break;
      };
      // este operador puede pegarse al lhs?
      if lbp < min_bp {
        break;
      }
      let token = self.tokens.bump(self.diagnostics)?;
      lhs = self.parse_infix(lhs, token)?;
    }
    Some(lhs)
  }

  /// Es el nud. Construye el LHS inicial.
  /// Maneja literales, identifiers, expresiones con parentesis, operadores unarios.
  fn parse_prefix(&mut self) -> Option<ExprId> {
    let token = self.tokens.bump(self.diagnostics)?;
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
        let expr = Expr::Var(lexeme.into());
        Some(self.ast.add_expr(expr, span.clone()))
      }
      TokenKind::LParen => {
        // Consumir `(` (con el bump ya lo hicimos)
        // Llamar recursivamente
        let inner_expr_id = self.parse_expr_bp(ASSIGN_BP)?;
        // Esperar hasta `)` OBLIGATORIAMENTE
        match self.tokens.expect(TokenKind::RParen, self.diagnostics) {
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
      // inicio de un bloque. // Consumir `{` (con el bump ya lo hicimos)
      TokenKind::LCurlyBrace => self.parse_block_expression(span),
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
      // el chequeo de infix_binding_power(token.kind()) hace que no se pueda alcanzar esta rama
      _ => unreachable!(),
    }
  }

  /// Convierte el error a Diagnostic, lo acumula en la lista de errores. El parser continua.
  fn emit_error(&mut self, err: &ParserError) {
    self.diagnostics.push(err.to_diagnostic())
  }

  fn expect_token(&mut self, kind: TokenKind) {
    if let Err(err) = self.tokens.expect(kind, self.diagnostics) {
      self.emit_error(&err);
    }
  }

  /// Funcion de punto de entrada para parsear un Statement. Se introduce la expresion en el AST mediante su StmtId
  pub(crate) fn parse_statement(&mut self) -> Option<StmtId> {
    // decidir que tipo de statement es.
    // uso peek_first() en vez de bump() porque el bump() lo uso al parsear en cada branch
    let token = self.tokens.peek_first(self.diagnostics)?;
    let (stmt, span) = match token.kind() {
      TokenKind::Let => self.parse_let_stmt()?,
      TokenKind::Const => self.parse_const_stmt()?,
      TokenKind::Return => self.parse_return_stmt()?,
      TokenKind::If => self.parse_if_stmt()?,
      TokenKind::Print => self.parse_print_stmt()?,
      _ => {
        // Ahora depende si el segundo token es un `=` o no
        if self
          .tokens
          .check_kind(1, TokenKind::Equal, self.diagnostics)
        {
          self.parse_assign_stmt()?
        } else {
          let expr_id = self.parse_expression()?;
          self.expect_token(TokenKind::Semicolon);
          (Stmt::Expr(expr_id), self.ast.expr_span(expr_id))
        }
      }
    };
    Some(self.ast.add_stmt(stmt, span))
  }

  /// let <var_expr> = <expr>;
  fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
    // Consumimos el `let`
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    self.expect_token(TokenKind::Let);
    // corregimos el span_start porque el let fue consumido antes
    self
      .parse_assignment_like(|var, initializer| Stmt::LetBinding { var, initializer })
      .map(|(stmt, span)| (stmt, span_start..span.end))
  }

  /// const <var_expr> = <expr>
  fn parse_const_stmt(&mut self) -> ParseResult<Stmt> {
    // Consumimos el `const`
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    self.expect_token(TokenKind::Const);
    self
      .parse_assignment_like(|var, initializer| Stmt::ConstBinding { var, initializer })
      .map(|(stmt, span)| (stmt, span_start..span.end))
  }

  /// <var> = <expr>
  fn parse_assign_stmt(&mut self) -> ParseResult<Stmt> {
    self.parse_assignment_like(|var, value_expr| Stmt::Assign { var, value_expr })
  }

  /// Helper para parse_assign, parse_let, y en un futuro parse_const
  fn parse_assignment_like<F>(&mut self, constructor: F) -> ParseResult<Stmt>
  where
    F: FnOnce(ExprId, ExprId) -> Stmt,
  {
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    // Ahora deberiamos consumir un identificador de variable
    // Igualmente, vamos a seguir parseando en vez de panickear
    let var = self.parse_identifier_expr()?;
    // Ahora deberiamos consumir un `=`
    self.expect_token(TokenKind::Equal);
    // Luego debemos parsear una expresion (que semanticamente es ValueExpr)
    let value = self.parse_expression()?;
    let span_end = self.tokens.peek_first(self.diagnostics)?.span().end;
    // Ahora deberiamos consumir un `;`
    self.expect_token(TokenKind::Semicolon);
    Some((constructor(var, value), span_start..span_end))
  }

  fn parse_identifier_expr(&mut self) -> Option<ExprId> {
    match self.tokens.expect(TokenKind::Identifier, self.diagnostics) {
      Ok(token) => {
        let expr_id = self
          .ast
          .add_expr(Expr::Var(token.lexeme().into()), token.span().clone());
        Some(expr_id)
      }
      Err(err) => {
        self.emit_error(&err);
        None
      }
    }
  }

  fn parse_return_stmt(&mut self) -> ParseResult<Stmt> {
    // return <expr>; o return;
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    self.expect_token(TokenKind::Return);
    if self
      .tokens
      .check_kind(0, TokenKind::Semicolon, self.diagnostics)
    {
      let span_end = self.tokens.peek_first(self.diagnostics)?.span().end;
      self.expect_token(TokenKind::Semicolon);
      return Some((Stmt::Return(None), span_start..span_end));
    }
    let expr_id = self.parse_expression()?;
    let span_end = self.tokens.peek_first(self.diagnostics)?.span().end;
    self.expect_token(TokenKind::Semicolon);
    Some((Stmt::Return(Some(expr_id)), span_start..span_end))
  }

  fn parse_if_stmt(&mut self) -> ParseResult<Stmt> {
    // if expr { block } [ else { block } ]
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    self.expect_token(TokenKind::If);
    let condition = self.parse_expression()?;
    // parsear el bloque if
    let if_block = self.parse_block()?;
    let mut span_end = self.ast.block_span(if_block).end;
    // si hay un bloque else, lo tengo que parsear
    if let Some(token) = self.tokens.peek_first(self.diagnostics)
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

  fn parse_print_stmt(&mut self) -> ParseResult<Stmt> {
    // print <expr>;
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    self.expect_token(TokenKind::Print);
    let expr_id = self.parse_expression()?;
    let span_end = self.tokens.peek_first(self.diagnostics)?.span().end;
    self.expect_token(TokenKind::Semicolon);
    Some((Stmt::Print(expr_id), span_start..span_end))
  }

  pub(crate) fn parse_block(&mut self) -> Option<BlockId> {
    let lbrace = self
      .tokens
      .expect(TokenKind::LCurlyBrace, self.diagnostics)
      .ok()?;

    self.parse_block_after_lbrace(lbrace.span())
  }

  fn parse_block_expression(&mut self, lbrace_span: &Span) -> Option<ExprId> {
    let block_id = self.parse_block_after_lbrace(lbrace_span)?;
    Some(self.ast.add_block_expr(block_id))
  }

  fn parse_block_after_lbrace(&mut self, lbrace_span: &Span) -> Option<BlockId> {
    // block ::== { stmt* }
    let mut block = Block::new();
    let span_start = lbrace_span.start;
    loop {
      // el bloque termina cuando encontramos el `}` correspondiente, o con un error si se teremina el archivo
      let token = self.tokens.peek_first(self.diagnostics)?.clone();
      let token_kind = token.kind();
      if token.is_eof() {
        self.emit_error(&ParserError::UnexpectedEOF);
      }
      if matches!(token_kind, TokenKind::RCurlyBrace) || token.is_eof() {
        break;
      }
      // hay que parsear un statement
      let stmt_id = self.parse_statement()?;
      // Si el statement es un return, lo marcamos como terminator
      if matches!(self.ast.stmt(stmt_id), Stmt::Return(_)) {
        block.add_stmt(stmt_id);
        block.set_terminator(Some(stmt_id));
        // Después de un return no debería haber más statements
        // pero dejamos que el loop detecte el '}'
        continue;
      }
      if block.terminator().is_some() {
        self.emit_error(&ParserError::StatementAfterReturn(
          self.ast.stmt_span(stmt_id),
        ));
      }
      block.add_stmt(stmt_id);
    }
    // hay que avanzar una ultima vez porque estabamos haciendo peek_first()
    let rbrace = self
      .tokens
      .expect(TokenKind::RCurlyBrace, self.diagnostics)
      .ok()?;
    let span_end = rbrace.span().end;
    Some(self.ast.add_block(block, span_start..span_end))
  }

  pub(crate) fn parse_program(&mut self) -> Option<Program> {
    // program ::= main <block_expr>
    let span_start = self.tokens.peek_first(self.diagnostics)?.span().start;
    self.expect_token(TokenKind::Main);
    let main_block_expr = self.parse_expression()?;
    if !matches!(self.ast.expr(main_block_expr), Expr::Block(_)) {
      self.emit_error(&ParserError::MainMustBeBlock(
        self.ast.expr_span(main_block_expr),
      ));
      return None;
    }
    let span_end = self.ast.expr_span(main_block_expr).end;
    Some(Program::new(main_block_expr, span_start..span_end))
  }

  #[cfg(test)]
  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Devuelve el Ast generado, consumiendose.
  pub(crate) fn into_ast(self) -> Ast {
    self.ast
  }
}

#[cfg(test)]
mod tests;
