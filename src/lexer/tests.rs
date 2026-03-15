use proptest::prelude::*;

use crate::lexer::{Lexer, token::TokenKind};

#[test]
fn eof_is_emitted() {
  let mut lexer = Lexer::new("");
  let mut diagnostics = Vec::new();
  let tok = lexer.next(&mut diagnostics).unwrap();
  assert!(tok.is_eof());
}

#[test]
fn lex_simple_delimiters_tokens() {
  let mut diagnostics = Vec::new();
  let src = "( ) { } ;";
  let mut lexer = Lexer::new(src);
  let expected_tokens = vec![
    TokenKind::LParen,
    TokenKind::RParen,
    TokenKind::LCurlyBrace,
    TokenKind::RCurlyBrace,
    TokenKind::Semicolon,
    TokenKind::Eof,
  ];
  for expected_token in expected_tokens {
    let token = lexer.next(&mut diagnostics).unwrap();
    assert_eq!(token.kind(), expected_token);
  }
}

#[test]
fn lex_keywords() {
  let mut diagnostics = Vec::new();
  let mut lexer = Lexer::new("let true false if const else return");
  let expected_tokens = vec![
    TokenKind::Let,
    TokenKind::BooleanLiteral,
    TokenKind::BooleanLiteral,
    TokenKind::If,
    TokenKind::Const,
    TokenKind::Else,
    TokenKind::Return,
    TokenKind::Eof,
  ];
  for expected_token in expected_tokens {
    let token = lexer.next(&mut diagnostics).unwrap();
    assert_eq!(token.kind(), expected_token);
  }
}

#[test]
fn lex_identifier() {
  let mut diagnostics = Vec::new();
  let mut lexer = Lexer::new("hello world");
  let first_token = lexer.next(&mut diagnostics).unwrap();
  assert_eq!(first_token.lexeme(), "hello");
  let second_token = lexer.next(&mut diagnostics).unwrap();
  assert_eq!(first_token.kind(), TokenKind::Identifier);
  assert_eq!(second_token.lexeme(), "world");
  assert_eq!(second_token.kind(), TokenKind::Identifier);
}

#[test]
fn lex_number_literal() {
  let mut diagnostics = Vec::new();
  let mut lexer = Lexer::new("12345");
  let token = lexer.next(&mut diagnostics).unwrap();
  assert_eq!(token.kind(), TokenKind::NumberLiteral);
  assert_eq!(token.lexeme(), "12345");
}

#[test]
fn lex_operators() {
  let mut diagnostics = Vec::new();
  let mut lexer = Lexer::new("+ == ! != > >= ! / ^^");
  let expected_tokens = vec![
    TokenKind::Plus,
    TokenKind::EqualEqual,
    TokenKind::Bang,
    TokenKind::BangEqual,
    TokenKind::Greater,
    TokenKind::GreaterEqual,
    TokenKind::Bang,
    TokenKind::Slash,
    TokenKind::CaretCaret,
    TokenKind::Eof,
  ];
  for expected_token in expected_tokens {
    let token = lexer.next(&mut diagnostics);
    assert_eq!(token.unwrap().kind(), expected_token);
  }
}

#[test]
fn lex_ill_formed_literal() {
  let mut diagnostics = Vec::new();
  let mut lexer = Lexer::new("123abc");
  assert!(lexer.next(&mut diagnostics).is_none());
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains(&"se detecto un literal mal formado 123abc".to_string())
  );
}

#[test]
fn lex_invalid_character() {
  let mut diagnostics = Vec::new();
  let mut lexer = Lexer::new("@");
  assert!(lexer.next(&mut diagnostics).is_some_and(|tok| tok.is_eof()));
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains(&"se detecto un caracter invalido '@'".to_string())
  );
}

#[test]
fn lex_operators_find_invalid_characters() {
  let mut diagnostics = Vec::new();
  let src = "+ == ! != > >= !$ ^/ ^^";
  let mut lexer = Lexer::new(src);
  loop {
    let token = lexer.next(&mut diagnostics);
    match token {
      Some(token) if token.is_eof() => break,
      _ => continue,
    }
  }
  assert_eq!(diagnostics.len(), 2);
}

// test unitario: mezcla operadores + números + identificadores
#[test]
fn lex_mixed_input() {
  let mut diagnostics = Vec::new();
  let input = "a + b * 42 = false";
  let mut lexer = Lexer::new(input);

  let expected_tokens = vec![
    TokenKind::Identifier,
    TokenKind::Plus,
    TokenKind::Identifier,
    TokenKind::Star,
    TokenKind::NumberLiteral,
    TokenKind::Equal,
    TokenKind::BooleanLiteral,
    TokenKind::Eof,
  ];
  for expected_token in expected_tokens {
    let token = lexer.next(&mut diagnostics).unwrap();
    assert_eq!(token.kind(), expected_token);
  }
}

// ===============================
// PROPERTY TESTS
// ===============================

proptest! {
  #[test]
  // necesito que las entradas sean ASCII para que no se rompan los proptest
  fn lexer_never_panics(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let mut diagnostics = Vec::new();
    let input = String::from_utf8(bytes).unwrap();
    let mut lexer = Lexer::new(&input);
    while lexer.next(&mut diagnostics).is_some() {}
  }
}

proptest! {
  #[test]
  fn spans_are_always_valid(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let mut diagnostics = Vec::new();
    let input = String::from_utf8(bytes).unwrap();
    let mut lexer = Lexer::new(&input);
    while let Some(tok) = lexer.next(&mut diagnostics) {
      let tok_span = tok.span();
      prop_assert!(tok_span.start <= tok_span.end);
      prop_assert!(tok_span.end <= input.len());
    }
  }
}

// la concatenacion de los lexemas reconstruye el output (excepto por los whitespaces)
proptest! {
  #[test]
  fn lexemes_reconstruct_input(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let mut diagnostics = Vec::new();
    let input = String::from_utf8(bytes).unwrap();
    let mut lexer = Lexer::new(&input);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next(&mut diagnostics) {
      tokens.push(token);
    }
    let reconstructed: String = tokens
        .iter()
        .filter(|t| t.kind() != TokenKind::Eof)
        .map(|t| t.lexeme().to_string())
        .collect();
    let no_ws: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    // esto funciona en caso de que no haya habido diagnostics durante la ejecucion (por ejemplo un invalid character)
    if diagnostics.is_empty() {
      prop_assert_eq!(reconstructed, no_ws);
    }
  }
}
