// El Parser no deberia hablar directamente con el Lexer
// Responsabilidad:
// - Encapsular Lexer (una capa de abstraccion)
// - Manejar lookahead
// - API ergonomica: peek(), bump(), expect(kind), match(kind)
// Esto evita bugs de sincronizacion, centraliza el manejo de EOF y de errores

use std::collections::VecDeque;

use crate::{
  diagnostics::Diagnostic,
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
  fn ensure_buffered(&mut self, n: usize, diagnostics: &mut Vec<Diagnostic>) {
    while self.buffer.len() <= n {
      match self.lexer.next(diagnostics) {
        Some(token) => self.buffer.push_back(token),
        None => break,
      }
    }
  }

  // =========================
  // API publica del stream
  // =========================

  /// Lookahead sin consumir.
  pub fn peek(&mut self, n: usize, diagnostics: &mut Vec<Diagnostic>) -> Option<&Token> {
    self.ensure_buffered(n, diagnostics);
    self.buffer.get(n)
  }

  /// peek(0) = token actual
  pub fn peek_first(&mut self, diagnostics: &mut Vec<Diagnostic>) -> Option<&Token> {
    self.peek(0, diagnostics)
  }

  /// Consume y devuelve el siguiente token
  pub fn bump(&mut self, diagnostics: &mut Vec<Diagnostic>) -> Option<Token> {
    if self.buffer.is_empty() {
      return self.lexer.next(diagnostics);
    }
    self.buffer.pop_front()
  }

  /// Siempre avanza. Devuelve error si el token no es del tipo esperado.
  pub fn expect(
    &mut self,
    kind: TokenKind,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> Result<Token, ParserError> {
    match self.bump(diagnostics) {
      Some(token) if token.kind() == kind => Ok(token),
      Some(token) => Err(ParserError::UnexpectedToken(token)),
      None => Err(ParserError::UnexpectedEOF),
    }
  }

  /// Chequea el kind del token en posición `n` sin consumir.
  pub fn check_kind(
    &mut self,
    n: usize,
    kind: TokenKind,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> bool {
    matches!(self.peek(n, diagnostics), Some(tok) if tok.kind() == kind)
  }
}

#[cfg(test)]
mod tests;
