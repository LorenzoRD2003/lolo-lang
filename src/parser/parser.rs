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
pub struct Parser<'a> {
  /// Parser NO habla con Lexer. El lookahead esta centralizado en el TokenStream. Handlea EOF.
  tokens: &'a mut TokenStream<'a>,
  /// El parser es quien construye nodos, fusiona coherementemente los spans del Ast arena-based
  ast: Ast,
  /// Para acumular los errores. Podemos devolver un AST parcial valido
  diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
  pub fn new(tokens: &'a mut TokenStream<'a>) -> Self {
    Self {
      tokens,
      ast: Ast::empty(),
      diagnostics: Vec::new(),
    }
  }

  /// Esta funcion es el punto de entrada cuando queremos parsear una expresion
  /// Luego de generarse la expresion, se guarda en el AST obteniendose un ExprId, y se lo devuelve.
  pub fn parse_expression(&mut self) -> Option<ExprId> {
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
    if self.tokens.peek()?.kind() == TokenKind::Equal {
      return Some(lhs);
    }
    // Mientras el siguiente token pueda extender la expresion...
    // o el token no es operador, o el binding power es insuficiente
    let mut i = 5;
    loop {
      i = i + 1;
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
      // el chequeo de infix_binding_power(token.kind()) hace que no se pueda alcanzar esta rama
      _ => unreachable!(),
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

  /// Funcion de punto de entrada para parsear un Statement. Se introduce la expresion en el AST mediante su StmtId
  pub fn parse_statement(&mut self) -> Option<StmtId> {
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

  fn parse_let_stmt(&mut self) -> Option<(Stmt, Span)> {
    // let <var_expr> = <expr>;
    // Consumimos el `let`
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::Let);
    // Ahora deberiamos consumir un identificador
    // Igualmente, vamos a seguir parseando en vez de panickear
    let token = self.tokens.peek()?.clone();
    let var_expr_id = self.parse_expression()?;
    if !self.ast.expr(var_expr_id).is_var() {
      self.emit_error(&ParserError::IdentifierExpected(token));
    }
    // Ahora deberiamos consumir un `=`
    self.expect_token(TokenKind::Equal);
    // Ahora vamos a parsear una expresion
    let expr_id = self.parse_expression()?;
    // Ahora deberiamos consumir un `;`
    let span_end = self.tokens.peek()?.span().end;
    self.expect_token(TokenKind::Semicolon);
    Some((
      Stmt::Let {
        var: var_expr_id,
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

  pub fn parse_block(&mut self) -> Option<BlockId> {
    // block ::== { stmt* }
    let mut block = Block::new();
    let lbrace = self.tokens.expect(TokenKind::LCurlyBrace).ok()?;
    let span_start = lbrace.span().start;
    loop {
      // el bloque termina cuando encontramos el `}` correspondiente, o con un error si se teremina el archivo
      let token = self.tokens.peek()?.clone();
      let token_kind = token.kind();
      if matches!(token_kind, TokenKind::EOF) {
        self.emit_error(&ParserError::UnexpectedEOF);
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

  pub fn parse_program(&mut self) -> Option<Program> {
    // program ::= main block
    let span_start = self.tokens.peek()?.span().start;
    self.expect_token(TokenKind::Main);
    let block = self.parse_block()?;
    let span_end = self.ast.block_span(block).end;
    Some(Program::new(block, span_start..span_end))
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }
}

#[cfg(test)]
mod tests;
