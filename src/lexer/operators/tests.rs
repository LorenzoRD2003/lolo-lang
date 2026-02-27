use crate::lexer::{operators::match_operator, token::TokenKind};

#[test]
fn longest_match_equality() {
  assert_eq!(
    match_operator('=', Some('=')),
    Some((TokenKind::EqualEqual, 2))
  );
}

#[test]
fn single_char_fallback() {
  assert_eq!(match_operator('&', Some('=')), None);
}

#[test]
fn longest_match_not_equal() {
  assert_eq!(
    match_operator('!', Some('=')),
    Some((TokenKind::BangEqual, 2))
  );
}

#[test]
fn single_char_not() {
  assert_eq!(match_operator('!', None), Some((TokenKind::Bang, 1)));
}

#[test]
fn greater_equal_vs_greater() {
  assert_eq!(
    match_operator('>', Some('=')),
    Some((TokenKind::GreaterEqual, 2))
  );
  assert_eq!(
    match_operator('>', Some('a')),
    Some((TokenKind::Greater, 1))
  );
}

#[test]
fn logical_operators() {
  assert_eq!(match_operator('&', Some('&')), Some((TokenKind::AndAnd, 2)));
  assert_eq!(match_operator('|', Some('|')), Some((TokenKind::OrOr, 2)));
}

#[test]
fn longest_match_preferred() {
  let op = match_operator('=', Some('='));
  assert_eq!(op, Some((TokenKind::EqualEqual, 2)));
}
