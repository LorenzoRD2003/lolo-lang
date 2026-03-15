use crate::{
  lexer::{Lexer, TokenKind},
  parser::{error::ParserError, token_stream::TokenStream},
};
use proptest::prelude::*;

// peek_first() consecutivos deben devolver el mismo token (una referencia)
// luego, bump() debe devolver el token que peek_first() vio (tomandolo)
#[test]
fn peek_does_not_advance() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("x + 1");
  let mut ts = TokenStream::new(lexer);

  let first = ts.peek_first(&mut diagnostics).unwrap().clone();
  let first_again = ts.peek_first(&mut diagnostics).unwrap().clone();
  assert_eq!(first, first_again);

  let tok = ts.bump(&mut diagnostics).unwrap();
  assert_eq!(
    tok, first,
    "bump() debe devolver el token que peek_first() vio"
  );
}

// bump() consecutivos deberian devolver tokens diferentes
#[test]
fn bump_advances() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("x + 1");
  let mut ts = TokenStream::new(lexer);

  let first = ts.bump(&mut diagnostics).unwrap();
  let second = ts.bump(&mut diagnostics).unwrap();
  assert_ne!(first, second);
  assert_eq!(first.kind(), TokenKind::Identifier);
  assert_eq!(second.kind(), TokenKind::Plus); // el segundo token es +
}

#[test]
fn match_kind_returns_true_and_peeks() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("+");
  let mut ts = TokenStream::new(lexer);
  assert!(ts.check_kind(0, TokenKind::Plus, &mut diagnostics));
  // peek ahora debe seguir viendo el mismo token porque match_kind no avanza
  assert_eq!(
    ts.peek_first(&mut diagnostics).unwrap().kind(),
    TokenKind::Plus
  );
}

#[test]
fn match_kind_returns_false_without_advancing() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("-");
  let mut ts = TokenStream::new(lexer);
  // peek sigue viendo el mismo token
  assert!(!ts.check_kind(0, TokenKind::BangEqual, &mut diagnostics));
  assert_eq!(
    ts.peek_first(&mut diagnostics).unwrap().kind(),
    TokenKind::Minus
  );
}

#[test]
fn expect_succeeds_and_advances() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("+ -");
  let mut ts = TokenStream::new(lexer);
  let token = ts.expect(TokenKind::Plus, &mut diagnostics).unwrap();
  assert_eq!(token.kind(), TokenKind::Plus);
  // peek_first() ahora debe ver el siguiente token
  assert_eq!(
    ts.peek_first(&mut diagnostics).unwrap().kind(),
    TokenKind::Minus
  );
}

#[test]
fn expect_fails_and_advances_on_unexpected_token() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("+ -");
  let mut ts = TokenStream::new(lexer);
  let err = ts.expect(TokenKind::Minus, &mut diagnostics).unwrap_err();
  match err {
    ParserError::UnexpectedToken(token) => assert_eq!(token.kind(), TokenKind::Plus),
    _ => panic!("Expected UnexpectedToken"),
  }
  // peek_first() ahora debe ver el siguiente token
  assert_eq!(
    ts.peek_first(&mut diagnostics).unwrap().kind(),
    TokenKind::Minus
  );
}

#[test]
fn expect_works_for_eof_on_empty_stream() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("");
  let mut ts = TokenStream::new(lexer);
  let token = ts.expect(TokenKind::Eof, &mut diagnostics).unwrap();
  assert!(token.is_eof());
}

#[test]
fn expect_fails_when_there_are_no_more_tokens() {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new("");
  let mut ts = TokenStream::new(lexer);
  ts.expect(TokenKind::Eof, &mut diagnostics).unwrap();
  let err = ts.expect(TokenKind::Eof, &mut diagnostics).unwrap_err();
  assert!(matches!(err, ParserError::UnexpectedEOF));
}

proptest! {
  // El primer bump debe devolver exactamente el mismo token que peek
  #[test]
  fn peek_never_consumes(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap_or_default();
    let mut diagnostics = Vec::new();
    let lexer = Lexer::new(&input);
    let mut ts = TokenStream::new(lexer);
    for _ in 0..5 { // Llamamos varias veces a peek
      let _ = ts.peek_first(&mut diagnostics);
    }
    let first_peek = ts.peek_first(&mut diagnostics).cloned();
    let first_bump = ts.bump(&mut diagnostics);
    assert_eq!(first_peek, first_bump);
  }

  // Property test: match_kind nunca avanza si no coincide
  #[test]
  fn match_kind_does_not_advance_unless_matching(bytes in proptest::collection::vec(0u8..=127u8, 0..20)) {
    let input = String::from_utf8(bytes).unwrap_or_default();
    let mut diagnostics = Vec::new();
    let lexer = Lexer::new(&input);
    let mut ts = TokenStream::new(lexer);
    let initial_peek = ts.peek_first(&mut diagnostics).cloned();
    let _ = ts.check_kind(0, TokenKind::Plus, &mut diagnostics); // tratar de coincidir con + arbitrario
    let after_peek = ts.peek_first(&mut diagnostics).cloned();
    // si no coincide, peek sigue igual
    if let (Some(init), Some(after)) = (initial_peek, after_peek)
      && init.kind() != TokenKind::Plus {
        prop_assert_eq!(init, after);
      }
  }

  // Property test: expect siempre avanza (peek cambia o se consume EOF)
  #[test]
  fn expect_always_advances(bytes in proptest::collection::vec(0u8..=127u8, 0..20)) {
    let input = String::from_utf8(bytes).unwrap_or_default();
    let mut diagnostics = Vec::new();
    let lexer = Lexer::new(&input);
    let mut ts = TokenStream::new(lexer);
    let initial_peek = ts.peek_first(&mut diagnostics).cloned();
    let _ = ts.expect(TokenKind::Plus, &mut diagnostics);
    let after_peek = ts.peek_first(&mut diagnostics).cloned();
    if initial_peek.is_some() {
      prop_assert!(initial_peek != after_peek);
    }
  }
}
