use crate::{
  lexer::token::TokenKind,
  parser::precedence::{ADD_BP, AND_BP, ASSIGN_BP, CMP_BP, MUL_BP, OR_BP, UNARY_BP, XOR_BP},
};

/// Para operadores infijos: devuelve (lo, hi), que son el binding power
/// para el lado izquierdo y derecho respectivamente. permite modelar asociatividad izquierda/derecha
fn infix_binding_power(kind: TokenKind) -> Option<(u8, u8)> {
  match kind {
    TokenKind::Star | TokenKind::Slash => Some((MUL_BP, MUL_BP + 1)),
    TokenKind::Plus | TokenKind::Minus => Some((ADD_BP, ADD_BP + 1)),
    TokenKind::EqualEqual
    | TokenKind::BangEqual
    | TokenKind::Greater
    | TokenKind::Less
    | TokenKind::GreaterEqual
    | TokenKind::LessEqual => Some((CMP_BP, CMP_BP)),
    TokenKind::AndAnd => Some((AND_BP, AND_BP + 1)),
    TokenKind::OrOr => Some((OR_BP, OR_BP + 1)),
    TokenKind::CaretCaret => Some((XOR_BP, XOR_BP + 1)),
    TokenKind::Equal => Some((ASSIGN_BP, ASSIGN_BP)),
    _ => None,
  }
}

/// Para operadores prefijos (not, neg, en un futuro ref, deref)
fn prefix_binding_power(kind: TokenKind) -> Option<u8> {
  match kind {
    TokenKind::Bang | TokenKind::Minus => Some(UNARY_BP),
    _ => None,
  }
}
