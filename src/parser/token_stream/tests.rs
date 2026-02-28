use crate::{
  lexer::{lexer::Lexer, token::TokenKind},
  parser::{error::ParserError, token_stream::TokenStream},
};
use proptest::prelude::*;

// peek_first() consecutivos deben devolver el mismo token (una referencia)
// luego, bump() debe devolver el token que peek_first() vio (tomandolo)
#[test]
fn peek_does_not_advance() {
  let mut lexer = Lexer::new("x + 1");
  let mut ts = TokenStream::new(&mut lexer);

  let first = ts.peek_first().unwrap().clone();
  let first_again = ts.peek_first().unwrap().clone();
  assert_eq!(first, first_again);
  
  let tok = ts.bump().unwrap();
  assert_eq!(
    tok, first,
    "bump() debe devolver el token que peek_first() vio"
  );
}

// bump() consecutivos deberian devolver tokens diferentes
#[test]
fn bump_advances() {
  let mut lexer = Lexer::new("x + 1");
  let mut ts = TokenStream::new(&mut lexer);

  let first = ts.bump().unwrap();
  let second = ts.bump().unwrap();
  assert_ne!(first, second);
  assert_eq!(first.kind(), TokenKind::Identifier);
  assert_eq!(second.kind(), TokenKind::Plus); // el segundo token es +
}

#[test]
fn match_kind_returns_true_and_peeks() {
  let mut lexer = Lexer::new("+");
  let mut ts = TokenStream::new(&mut lexer);
  assert!(ts.check_kind(0, TokenKind::Plus));
  // peek ahora debe seguir viendo el mismo token porque match_kind no avanza
  assert_eq!(ts.peek_first().unwrap().kind(), TokenKind::Plus);
}

#[test]
fn match_kind_returns_false_without_advancing() {
  let mut lexer = Lexer::new("-");
  let mut ts = TokenStream::new(&mut lexer);
  // peek sigue viendo el mismo token
  assert!(!ts.check_kind(0, TokenKind::BangEqual));
  assert_eq!(ts.peek_first().unwrap().kind(), TokenKind::Minus);
}

#[test]
fn expect_succeeds_and_advances() {
  let mut lexer = Lexer::new("+ -");
  let mut ts = TokenStream::new(&mut lexer);
  let token = ts.expect(TokenKind::Plus).unwrap();
  assert_eq!(token.kind(), TokenKind::Plus);
  // peek_first() ahora debe ver el siguiente token
  assert_eq!(ts.peek_first().unwrap().kind(), TokenKind::Minus);
}

#[test]
fn expect_fails_and_advances_on_unexpected_token() {
  let mut lexer = Lexer::new("+ -");
  let mut ts = TokenStream::new(&mut lexer);
  let err = ts.expect(TokenKind::Minus).unwrap_err();
  match err {
    ParserError::UnexpectedToken(token) => assert_eq!(token.kind(), TokenKind::Plus),
    _ => panic!("Expected UnexpectedToken"),
  }
  // peek_first() ahora debe ver el siguiente token
  assert_eq!(ts.peek_first().unwrap().kind(), TokenKind::Minus);
}

#[test]
fn expect_works_for_eof_on_empty_stream() {
  let mut lexer = Lexer::new("");
  let mut ts = TokenStream::new(&mut lexer);
  let token = ts.expect(TokenKind::EOF).unwrap();
  assert_eq!(token.kind(), TokenKind::EOF);
}

#[test]
fn expect_fails_when_there_are_no_more_tokens() {
  let mut lexer = Lexer::new("");
  let mut ts = TokenStream::new(&mut lexer);
  ts.expect(TokenKind::EOF).unwrap();
  let err = ts.expect(TokenKind::EOF).unwrap_err();
  assert!(matches!(err, ParserError::UnexpectedEOF));
}

proptest! {
  // El primer bump debe devolver exactamente el mismo token que peek
  #[test]
  fn peek_never_consumes(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap_or_default();
    let mut lexer = Lexer::new(&input);
    let mut ts = TokenStream::new(&mut lexer);
    for _ in 0..5 { // Llamamos varias veces a peek
      let _ = ts.peek_first();
    }
    let first_peek = ts.peek_first().cloned();
    let first_bump = ts.bump();
    assert_eq!(first_peek, first_bump);
  }

  // Property test: match_kind nunca avanza si no coincide
  #[test]
  fn match_kind_does_not_advance_unless_matching(bytes in proptest::collection::vec(0u8..=127u8, 0..20)) {
    let input = String::from_utf8(bytes).unwrap_or_default();
    let mut lexer = Lexer::new(&input);
    let mut ts = TokenStream::new(&mut lexer);
    let initial_peek = ts.peek_first().cloned();
    let _ = ts.check_kind(0, TokenKind::Plus); // tratar de coincidir con + arbitrario
    let after_peek = ts.peek_first().cloned();
    // si no coincide, peek sigue igual
    if let (Some(init), Some(after)) = (initial_peek, after_peek) {
      if init.kind() != TokenKind::Plus {
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
    let initial_peek = ts.peek_first().cloned();
    let _ = ts.expect(TokenKind::Plus);
    let after_peek = ts.peek_first().cloned();
    if initial_peek.is_some() {
      prop_assert!(initial_peek != after_peek);
    }
  }
}
