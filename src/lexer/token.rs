// TokenKind: Describe que tipo de cosa es el token (estructura sintactica abstracta)

use crate::common::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind {
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
  Eof,
}

impl TokenKind {
  // pub(crate) fn is_literal(&self) -> bool {
  //   matches!(self, TokenKind::NumberLiteral | TokenKind::BooleanLiteral)
  // }

  pub(crate) fn is_unary(&self) -> bool {
    matches!(self, TokenKind::Bang | TokenKind::Minus)
  }

  pub(crate) fn is_binary(&self) -> bool {
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

  pub(crate) fn is_comparison(&self) -> bool {
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
pub(crate) struct Token {
  kind: TokenKind,
  // El lexema es exactamente el texto que se ve
  lexeme: String,
  span: Span,
}

impl Token {
  pub(crate) fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
    Self { kind, lexeme, span }
  }

  pub(crate) fn kind(&self) -> TokenKind {
    self.kind
  }

  pub(crate) fn lexeme(&self) -> &str {
    &self.lexeme
  }

  pub(crate) fn span(&self) -> &Span {
    &self.span
  }

  pub(crate) fn is_eof(&self) -> bool {
    self.kind() == TokenKind::Eof
  }
}
