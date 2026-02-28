// El Parser no deberia hablar directamente con el Lexer
// Responsabilidad:
// - Encapsular Lexer (una capa de abstraccion)
// - Manejar lookahead
// - API ergonomica: peek(), bump(), expect(kind), match(kind)
// Esto evita bugs de sincronizacion, centraliza el manejo de EOF y de errores

use crate::{
  lexer::{
    cache::{CACHE_LEN, TokenCache},
    lexer::Lexer,
    token::{Token, TokenKind},
  },
  parser::error::ParserError,
};

#[derive(Debug)]
pub struct TokenStream<'a> {
  lexer: &'a mut Lexer<'a>,
}

impl<'a> TokenStream<'a> {
  pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
    Self { lexer }
  }

  /// Devuelve una referencia al token indicado del cache (lookahead), sin avanzar
  pub fn peek(&mut self, index: usize) -> Option<&Token> {
    debug_assert!(index < CACHE_LEN);
    if self.cache().is_empty() {
      self.lexer.update_token_cache();
    }
    // dbg!(self.cache());
    self.cache().get_at(index)
  }

  pub fn peek_first(&mut self) -> Option<&Token> {
    self.peek(0)
  }

  /// Avanza al siguiente token y devuelve el anterior
  pub fn bump(&mut self) -> Option<Token> {
    if !self.cache().is_empty() {
      self.lexer.delete_cache();
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

  /// Si el token de posicion indicada desde el lugar actual es del kind indicado,
  /// lo consume y devuelve true; si no, devuelve false.
  pub fn check_kind(&mut self, index: usize, kind: TokenKind) -> bool {
    debug_assert!(index < CACHE_LEN);
    matches!(self.peek(index), Some(token) if token.kind() == kind)
  }

  fn cache(&self) -> &TokenCache {
    self.lexer.token_cache()
  }
}

#[cfg(test)]
mod tests;
