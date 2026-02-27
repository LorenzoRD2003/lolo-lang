// El Parser no deberia hablar directamente con el Lexer
// Responsabilidad:
// - Encapsular Lexer (una capa de abstraccion)
// - Manejar lookahead
// - API ergonomica: peek(), bump(), expect(kind), match(kind)
// Esto evita bugs de sincronizacion, centraliza el manejo de EOF y de errores

use crate::{
  lexer::{
    lexer::Lexer,
    token::{Token, TokenKind},
  },
  parser::error::ParserError,
};

#[derive(Debug)]
pub struct TokenStream<'a> {
  lexer: &'a mut Lexer<'a>,
  lookahead: Option<Token>,
}

impl<'a> TokenStream<'a> {
  pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
    Self {
      lexer,
      lookahead: None,
    }
  }

  /// Devuelve una referencia al token actual (lookahead), sin avanzar
  pub fn peek(&mut self) -> Option<&Token> {
    if self.lookahead.is_none() {
      self.lookahead = self.lexer.peek_token();
    }
    self.lookahead.as_ref()
  }

  /// Avanza al siguiente token y devuelve el anterior
  pub fn bump(&mut self) -> Option<Token> {
    if self.lookahead.is_some() {
      self.lookahead.take();
    }
    self.lexer.next().and_then(Result::ok)
  }

  /// Comprueba que el token actual sea del kind esperado. Siempre avanza.
  /// Devuelve Ok(token) si coincide, y Err(ParserError) si no coincide.
  pub fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
    match self.bump() {
      Some(token) => {
        if token.kind() == kind {
          Ok(token)
        } else {
          Err(ParserError::UnexpectedToken(token))
        }
      }
      None => Err(ParserError::UnexpectedEOF),
    }
  }

  /// Si el token actual es del kind indicado, lo consume y devuelve true; si no, devuelve false
  pub fn check_kind(&mut self, kind: TokenKind) -> bool {
    matches!(self.peek(), Some(token) if token.kind() == kind)
  }
}

#[cfg(test)]
mod tests;
