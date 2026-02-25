// TokenKind: Describe que tipo de cosa es el token (estructura sintactica abstracta)

use crate::common::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
  // Constants
  NumberLiteral,  // i32
  BooleanLiteral, // true/false
  // Variables
  Identifier,
  // operators
  Bang,         // ! (not)
  Plus,         // +
  Minus,        // - (sub, neg)
  Star,         // *
  Slash,        // /
  Equal,        // =
  EqualEqual,   // ==
  BangEqual,    // !=
  Greater,      // >
  Less,         // <
  GreaterEqual, // >=
  LessEqual,    // <=
  AndAnd,       // &&
  OrOr,         // ||
  CaretCaret,   // ^^
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
  // Start of Program
  Main,
  // End-Of-File
  EOF,
}

// Token: Representa una ocurrencia concreta en el codigo fuente (contiene tipo + texto + span)
// Son la interfaz que va a ser generada por el lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  pub(crate) kind: TokenKind,
  // El lexema es exactamente el texto que se ve
  pub(crate) lexeme: String,
  pub(crate) span: Span,
}
