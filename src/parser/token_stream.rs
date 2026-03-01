// El Parser no deberia hablar directamente con el Lexer
// Responsabilidad:
// - Encapsular Lexer (una capa de abstraccion)
// - Manejar lookahead
// - API ergonomica: peek(), bump(), expect(kind), match(kind)
// Esto evita bugs de sincronizacion, centraliza el manejo de EOF y de errores

use std::collections::VecDeque;

use crate::{
  lexer::{
    lexer::Lexer,
    token::{Token, TokenKind},
  },
  parser::error::ParserError,
};

#[derive(Debug)]
pub struct TokenStream<'a> {
  /// El token stream va a poseer al lexer
  lexer: Lexer<'a>,
  /// Buffer de tokens
  buffer: VecDeque<Token>,
}

impl<'a> TokenStream<'a> {
  pub fn new(lexer: Lexer<'a>) -> Self {
    Self {
      lexer,
      buffer: VecDeque::new(),
    }
  }

  // =========================
  // Buffer management
  // =========================

  /// Asegura que haya al menos `n + 1` tokens en el buffer
  fn ensure_buffered(&mut self, n: usize) {
    while self.buffer.len() <= n {
      match self.lexer.next() {
        Some(Ok(token)) => self.buffer.push_back(token),
        Some(Err(_err)) => {
          // TODO posible: impl From<LexerError> for ParserError
          continue;
        }
        None => break,
      }
    }
  }

  // =========================
  // API publica del stream
  // =========================

  /// Lookahead sin consumir.
  pub fn peek(&mut self, n: usize) -> Option<&Token> {
    self.ensure_buffered(n);
    self.buffer.get(n)
  }

  /// peek(0) = token actual
  pub fn peek_first(&mut self) -> Option<&Token> {
    self.peek(0)
  }

  /// Consume y devuelve el siguiente token
  pub fn bump(&mut self) -> Option<Token> {
    if self.buffer.is_empty() {
      return self.lexer.next().and_then(Result::ok);
    }
    self.buffer.pop_front()
  }

  /// Siempre avanza. Devuelve error si el token no es del tipo esperado.
  pub fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
    match self.bump() {
      Some(token) if token.kind() == kind => Ok(token),
      Some(token) => Err(ParserError::UnexpectedToken(token)),
      None => Err(ParserError::UnexpectedEOF),
    }
  }

  /// Chequea el kind del token en posición `n` sin consumir.
  pub fn check_kind(&mut self, n: usize, kind: TokenKind) -> bool {
    matches!(self.peek(n), Some(tok) if tok.kind() == kind)
  }

  // /// Devuelve true si el token actual es EOF.
  // pub fn is_eof(&mut self) -> bool {
  //   matches!(self.peek(0), Some(tok) if tok.kind() == TokenKind::EOF)
  // }
}

#[cfg(test)]
mod tests;
