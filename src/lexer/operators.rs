// Responsabilidades: reconocer operadores multi-char, y definir precedencia lexica (longest match)

use crate::lexer::token::TokenKind;

pub(crate) fn match_operator(current: char, next: Option<char>) -> Option<(TokenKind, usize)> {
  let kind = match (current, next) {
    // ===== MULTI CHAR FIRST =====
    ('=', Some('=')) => TokenKind::EqualEqual,
    ('!', Some('=')) => TokenKind::BangEqual,
    ('>', Some('=')) => TokenKind::GreaterEqual,
    ('<', Some('=')) => TokenKind::LessEqual,
    ('&', Some('&')) => TokenKind::AndAnd,
    ('|', Some('|')) => TokenKind::OrOr,
    ('^', Some('^')) => TokenKind::CaretCaret,
    // ===== SINGLE CHAR =====
    ('=', _) => TokenKind::Equal,
    ('+', _) => TokenKind::Plus,
    ('-', _) => TokenKind::Minus,
    ('*', _) => TokenKind::Star,
    ('/', _) => TokenKind::Slash,
    ('>', _) => TokenKind::Greater,
    ('<', _) => TokenKind::Less,
    ('!', _) => TokenKind::Bang,
    // ===== DEFAULT =====
    _ => return None,
  };
  let width = match (current, next) {
    ('=', Some('='))
    | ('!', Some('='))
    | ('>', Some('='))
    | ('<', Some('='))
    | ('&', Some('&'))
    | ('|', Some('|'))
    | ('^', Some('^')) => 2,
    _ => 1,
  };
  Some((kind, width))
}

#[cfg(test)]
mod tests {
  use super::*;

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
}
