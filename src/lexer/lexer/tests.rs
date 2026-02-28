use proptest::prelude::*;

use crate::lexer::{error::LexerError, lexer::Lexer, token::TokenKind};

#[test]
fn eof_is_emitted() {
  let mut lexer = Lexer::new("");
  let tok = lexer.next().unwrap().unwrap();
  assert_eq!(tok.kind(), TokenKind::EOF);
}

#[test]
fn lex_simple_delimiters_tokens() {
  let src = "( ) { } ;";
  let tokens: Vec<TokenKind> = Lexer::new(src).map(|res| res.unwrap().kind()).collect();

  assert_eq!(
    tokens,
    vec![
      TokenKind::LParen,
      TokenKind::RParen,
      TokenKind::LCurlyBrace,
      TokenKind::RCurlyBrace,
      TokenKind::Semicolon,
      TokenKind::EOF,
    ]
  );
}

#[test]
fn lex_keywords() {
  let tokens: Vec<TokenKind> = Lexer::new("let true false if else return")
    .map(|res| res.unwrap().kind())
    .collect();

  assert_eq!(
    tokens,
    vec![
      TokenKind::Let,
      TokenKind::BooleanLiteral,
      TokenKind::BooleanLiteral,
      TokenKind::If,
      TokenKind::Else,
      TokenKind::Return,
      TokenKind::EOF
    ]
  );
}

#[test]
fn lex_identifier() {
  let src = "hello world";
  let tokens: Vec<_> = Lexer::new(src).map(Result::unwrap).collect();
  // recordar que aun no hay strings en el lenguaje. esas deberian estar entre comillas en un futuro
  assert_eq!(tokens[0].lexeme(), "hello");
  assert_eq!(tokens[0].kind(), TokenKind::Identifier);
  assert_eq!(tokens[1].lexeme(), "world");
  assert_eq!(tokens[1].kind(), TokenKind::Identifier);
}

#[test]
fn lex_number_literal() {
  let src = "12345";
  let token = Lexer::new(src).next().unwrap().unwrap();
  assert_eq!(token.kind(), TokenKind::NumberLiteral);
  assert_eq!(token.lexeme(), "12345");
}

#[test]
fn lex_operators() {
  let src = "+ == ! != > >= !$ ^/ ^^";
  let lexer = Lexer::new(src);
  let tokens: Vec<TokenKind> = lexer
    .filter_map(|res| res.ok().map(|tok| tok.kind()))
    .collect();
  assert_eq!(
    tokens,
    vec![
      TokenKind::Plus,
      TokenKind::EqualEqual,
      TokenKind::Bang,
      TokenKind::BangEqual,
      TokenKind::Greater,
      TokenKind::GreaterEqual,
      TokenKind::Bang,
      TokenKind::Slash,
      TokenKind::CaretCaret,
      TokenKind::EOF,
    ]
  );
}

#[test]
fn lex_ill_formed_literal() {
  let src = "123abc";
  let result = Lexer::new(src).next().unwrap();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err, LexerError::IllFormedLiteral(src.into(), 0..6));
}

#[test]
fn lex_invalid_character() {
  let src = "@";
  let result = Lexer::new(src).next().unwrap();
  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(err, LexerError::InvalidCharacter('@', 0..1));
}

#[test]
fn lex_operators_find_invalid_characters() {
  let src = "+ == ! != > >= !$ ^/ ^^";
  let lexer = Lexer::new(src);
  // chequeo que encuentre el InvalidCharacter('$')
  let errors: Vec<LexerError> = lexer.filter_map(|res| res.err()).collect();
  assert_eq!(errors.len(), 2);
  assert_eq!(errors[0], LexerError::InvalidCharacter('$', 16..17));
  assert_eq!(errors[1], LexerError::InvalidCharacter('^', 18..19));
}

// test unitario: mezcla operadores + números + identificadores
#[test]
fn lex_mixed_input() {
  let input = "a + b * 42 = false";
  let tokens: Vec<TokenKind> = Lexer::new(input)
    .filter_map(|res| res.ok().map(|tok| tok.kind()))
    .collect();

  assert_eq!(
    tokens,
    vec![
      TokenKind::Identifier,
      TokenKind::Plus,
      TokenKind::Identifier,
      TokenKind::Star,
      TokenKind::NumberLiteral,
      TokenKind::Equal,
      TokenKind::BooleanLiteral,
      TokenKind::EOF,
    ]
  );
}

// necesito que las entradas sean ASCII para que no se rompan los proptest
proptest! {
#[test]
  fn lexer_never_panics(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap();
    let lexer = Lexer::new(&input);
    for tok in lexer { // iteracion completa
      let _ = tok;
    }
  }
}

proptest! {
  #[test]
  fn spans_are_always_valid(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap();
    let lexer = Lexer::new(&input);
    for tok in lexer {
      match tok {
        Ok(tok) => {
          let tok_span = tok.span();
          prop_assert!(tok_span.start <= tok_span.end);
          prop_assert!(tok_span.end <= input.len());
        }
        Err(LexerError::InvalidCharacter(_, span)) | Err(LexerError::IllFormedLiteral(_, span)) => {
          prop_assert!(span.start <= span.end);
          prop_assert!(span.end <= input.len());
        }
      }
    }
  }
}

// la concatenacion de los lexemas reconstruye el output (excepto por los whitespaces)
proptest! {
  #[test]
  fn lexemes_reconstruct_input(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap();
    let mut lexer = Lexer::new(&input);
    let reconstructed: String = lexer
        .by_ref()
        .filter_map(Result::ok)
        .filter(|t| t.kind() != TokenKind::EOF)
        .map(|t| t.lexeme().to_string())
        .collect();
    let no_ws: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    // esto funciona en caso de que no haya habido diagnostics durante la ejecucion (por ejemplo un invalid character)
    if lexer.diagnostics.is_empty() {
      prop_assert_eq!(reconstructed, no_ws);
    }
  }

  // peek_token() no cambia estado
  #[test]
  fn peek_token_is_pure(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap();
    let mut lexer = Lexer::new(&input);
    let pos_before = lexer.position;
    let diagnostics_before = lexer.diagnostics.len();
    let emitted_eof = lexer.emitted_eof;
    // Peek arbitrarias veces
    for _ in 0..10 {
      let _ = lexer.update_token_cache();
    }
    prop_assert_eq!(lexer.position, pos_before);
    prop_assert_eq!(lexer.diagnostics.len(), diagnostics_before);
    prop_assert_eq!(lexer.emitted_eof, emitted_eof);
  }

// en particular, peek_token() no altera la siguiente ejecucion de next()
  #[test]
  fn peek_does_not_change_next(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
    let input = String::from_utf8(bytes).unwrap();
    let mut lexer1 = Lexer::new(&input);
    let mut lexer2 = Lexer::new(&input);

    for _ in 0..10 {
      lexer1.update_token_cache(); // lexer1 usa peek
    }
    loop {
      let tok1 = lexer1.next();
      let tok2 = lexer2.next();
      assert_eq!(tok1.is_some(), tok2.is_some());
      if tok1.is_none() { break; }
      assert_eq!(tok1.unwrap().ok().map(|tok| tok.kind()), tok2.unwrap().ok().map(|tok| tok.kind()));
    }
  }
}
