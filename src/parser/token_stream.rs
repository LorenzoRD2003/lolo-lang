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
pub(crate) struct TokenStream<'a> {
  lexer: &'a mut Lexer<'a>,
  lookahead: Option<Token>,
}

impl<'a> TokenStream<'a> {
  pub(crate) fn new(lexer: &'a mut Lexer<'a>) -> Self {
    Self {
      lexer,
      lookahead: None,
    }
  }

  /// Devuelve una referencia al token actual (lookahead), sin avanzar
  pub(crate) fn peek(&mut self) -> Option<&Token> {
    if self.lookahead.is_none() {
      self.lookahead = self.lexer.peek_token();
    }
    self.lookahead.as_ref()
  }

  /// Avanza al siguiente token y devuelve el anterior
  pub(crate) fn bump(&mut self) -> Option<Token> {
    if self.lookahead.is_some() {
      self.lookahead.take();
    }
    self.lexer.next().and_then(Result::ok)
  }

  /// Comprueba que el token actual sea del kind esperado. Siempre avanza.
  /// Devuelve Ok(token) si coincide, y Err(ParserError) si no coincide.
  pub(crate) fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
    match self.bump() {
      Some(token) => {
        if token.kind == kind {
          Ok(token)
        } else {
          Err(ParserError::UnexpectedToken(token))
        }
      }
      None => Err(ParserError::MissingToken),
    }
  }

  /// Si el token actual es del kind indicado, lo consume y devuelve true; si no, devuelve false
  pub(crate) fn check_kind(&mut self, kind: TokenKind) -> bool {
    matches!(self.peek(), Some(token) if token.kind == kind)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{lexer::Lexer, token::TokenKind};
  use proptest::prelude::*;

  // peek() consecutivos deben devolver el mismo token (una referencia)
  // luego, bump() debe devolver el token que peek() vio (tomandolo)
  #[test]
  fn peek_does_not_advance() {
    let mut lexer = Lexer::new("x + 1");
    let mut ts = TokenStream::new(&mut lexer);

    let first = ts.peek().unwrap().clone();
    let first_again = ts.peek().unwrap().clone();
    assert_eq!(first, first_again);

    let tok = ts.bump().unwrap();
    assert_eq!(tok, first, "bump() debe devolver el token que peek() vio");
  }

  // bump() consecutivos deberian devolver tokens diferentes
  #[test]
  fn bump_advances() {
    let mut lexer = Lexer::new("x + 1");
    let mut ts = TokenStream::new(&mut lexer);

    let first = ts.bump().unwrap();
    let second = ts.bump().unwrap();
    assert_ne!(first, second);
    assert_eq!(first.kind, TokenKind::Identifier);
    assert_eq!(second.kind, TokenKind::Plus); // el segundo token es +
  }

  #[test]
  fn match_kind_returns_true_and_peeks() {
    let mut lexer = Lexer::new("+");
    let mut ts = TokenStream::new(&mut lexer);
    assert!(ts.check_kind(TokenKind::Plus));
    // peek ahora debe seguir viendo el mismo token porque match_kind no avanza
    assert_eq!(ts.peek().unwrap().kind, TokenKind::Plus);
  }

  #[test]
  fn match_kind_returns_false_without_advancing() {
    let mut lexer = Lexer::new("-");
    let mut ts = TokenStream::new(&mut lexer);
    // peek sigue viendo el mismo token
    assert!(!ts.check_kind(TokenKind::BangEqual));
    assert_eq!(ts.peek().unwrap().kind, TokenKind::Minus);
  }

  #[test]
  fn expect_succeeds_and_advances() {
    let mut lexer = Lexer::new("+ -");
    let mut ts = TokenStream::new(&mut lexer);
    let token = ts.expect(TokenKind::Plus).unwrap();
    assert_eq!(token.kind, TokenKind::Plus);
    // peek() ahora debe ver el siguiente token
    assert_eq!(ts.peek().unwrap().kind, TokenKind::Minus);
  }

  #[test]
  fn expect_fails_and_advances_on_unexpected_token() {
    let mut lexer = Lexer::new("+ -");
    let mut ts = TokenStream::new(&mut lexer);
    let err = ts.expect(TokenKind::Minus).unwrap_err();
    match err {
      ParserError::UnexpectedToken(token) => assert_eq!(token.kind, TokenKind::Plus),
      _ => panic!("Expected UnexpectedToken"),
    }
    // peek() ahora debe ver el siguiente token
    assert_eq!(ts.peek().unwrap().kind, TokenKind::Minus);
  }

  #[test]
  fn expect_works_for_eof_on_empty_stream() {
    let mut lexer = Lexer::new("");
    let mut ts = TokenStream::new(&mut lexer);
    let token = ts.expect(TokenKind::EOF).unwrap();
    assert_eq!(token.kind, TokenKind::EOF);
  }

  #[test]
  fn expect_fails_when_there_are_no_more_tokens() {
    let mut lexer = Lexer::new("");
    let mut ts = TokenStream::new(&mut lexer);
    ts.expect(TokenKind::EOF).unwrap();
    let err = ts.expect(TokenKind::EOF).unwrap_err();
    assert!(matches!(err, ParserError::MissingToken));
  }

  proptest! {
    // El primer bump debe devolver exactamente el mismo token que peek
    #[test]
    fn peek_never_consumes(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
      let input = String::from_utf8(bytes).unwrap_or_default();
      let mut lexer = Lexer::new(&input);
      let mut ts = TokenStream::new(&mut lexer);
      for _ in 0..5 { // Llamamos varias veces a peek
        let _ = ts.peek();
      }
      let first_peek = ts.peek().cloned();
      let first_bump = ts.bump();
      assert_eq!(first_peek, first_bump);
    }

    // Property test: match_kind nunca avanza si no coincide
    #[test]
    fn match_kind_does_not_advance_unless_matching(bytes in proptest::collection::vec(0u8..=127u8, 0..20)) {
      let input = String::from_utf8(bytes).unwrap_or_default();
      let mut lexer = Lexer::new(&input);
      let mut ts = TokenStream::new(&mut lexer);
      let initial_peek = ts.peek().cloned();
      let _ = ts.check_kind(TokenKind::Plus); // tratar de coincidir con + arbitrario
      let after_peek = ts.peek().cloned();
      // si no coincide, peek sigue igual
      if let (Some(init), Some(after)) = (initial_peek, after_peek) {
        if init.kind != TokenKind::Plus {
          prop_assert_eq!(init, after);
        }
      }
    }

    // Property test: expect siempre avanza (peek cambia o se consume EOF)
    #[test]
    fn expect_always_advances(bytes in proptest::collection::vec(0u8..=127u8, 0..20)) {
      let input = String::from_utf8(bytes).unwrap_or_default();
      let mut lexer = Lexer::new(&input);
      let mut ts = TokenStream::new(&mut lexer);
      let initial_peek = ts.peek().cloned();
      let _ = ts.expect(TokenKind::Plus);
      let after_peek = ts.peek().cloned();
      if initial_peek.is_some() {
        prop_assert!(initial_peek != after_peek);
      }
    }
  }
}
