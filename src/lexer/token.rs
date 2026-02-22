// TokenKind: Describe que tipo de cosa es el token (estructura sintactica abstracta)

use crate::common::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
  // Constants
  NumberLiteral,  // i32
  BooleanLiteral, // true/false
  // Variables
  Identifier,
  // Unary operators
  Neg,
  Not,
  // Binary operators
  Add,
  Sub,
  Mul,
  Div,
  Eq,
  Neq,
  Lt,
  Gt,
  Lte,
  Gte,
  And,
  Or,
  Xor,
  // Statements
  Let,
  Return,
  If,
  Else,
  Print,
  // Delimiters
  LParen,
  RParen,
  LCurlyBrace,
  RCurlyBrace,
  Semicolon,
  // End-Of-File
  EOF,
}

// Token: Representa una ocurrencia concreta en el codigo fuente (contiene tipo + texto + span)
// Son la interfaz que va a ser generada por el lexer
pub struct Token {
  kind: TokenKind,
  // El lexema es exactamente el texto que se ve
  lexeme: String,
  span: Span,
}
