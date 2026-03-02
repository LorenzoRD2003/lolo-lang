use crate::{
  common::span::Span,
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
  InvalidCharacter(char, Span),
  IllFormedLiteral(String, Span),
}

impl Diagnosable for LexerError {
  fn to_diagnostic(&self) -> Diagnostic {
    match &self {
      Self::InvalidCharacter(c, span) => {
        Diagnostic::error(format!("se detecto un caracter invalido '{c}'")).with_span(span.clone())
      }
      Self::IllFormedLiteral(lit, span) => {
        Diagnostic::error(format!("se detecto un literal mal formado {lit}"))
          .with_span(span.clone())
      }
    }
  }
}
