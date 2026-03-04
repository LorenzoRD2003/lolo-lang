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
mod tests;
