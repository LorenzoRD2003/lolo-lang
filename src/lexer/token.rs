// TokenKind: Describe que tipo de cosa es el token (estructura sintactica abstracta)

use crate::common::Span;

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
  Const,
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

impl TokenKind {
  pub fn is_literal(&self) -> bool {
    matches!(self, TokenKind::NumberLiteral | TokenKind::BooleanLiteral)
  }

  pub fn is_unary(&self) -> bool {
    matches!(self, TokenKind::Bang | TokenKind::Minus)
  }

  pub fn is_binary(&self) -> bool {
    matches!(
      self,
      TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::EqualEqual
        | TokenKind::BangEqual
        | TokenKind::Greater
        | TokenKind::Less
        | TokenKind::GreaterEqual
        | TokenKind::LessEqual
        | TokenKind::AndAnd
        | TokenKind::OrOr
        | TokenKind::CaretCaret
    )
  }

  pub fn is_comparison(&self) -> bool {
    matches!(
      self,
      TokenKind::EqualEqual
        | TokenKind::BangEqual
        | TokenKind::Greater
        | TokenKind::Less
        | TokenKind::GreaterEqual
        | TokenKind::LessEqual
    )
  }
}

// Token: Representa una ocurrencia concreta en el codigo fuente (contiene tipo + texto + span)
// Son la interfaz que va a ser generada por el lexer
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  kind: TokenKind,
  // El lexema es exactamente el texto que se ve
  lexeme: String,
  span: Span,
}

impl Token {
  pub fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
    Self { kind, lexeme, span }
  }

  pub fn kind(&self) -> TokenKind {
    self.kind
  }

  pub fn lexeme(&self) -> &str {
    &self.lexeme
  }

  pub fn span(&self) -> &Span {
    &self.span
  }

  pub fn is_eof(&self) -> bool {
    self.kind() == TokenKind::EOF
  }
}
