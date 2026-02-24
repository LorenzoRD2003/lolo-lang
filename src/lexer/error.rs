use crate::{common::span::Span, diagnostics::diagnostic::Diagnostic};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LexerErrorKind {
  InvalidCharacter(char),
  IllFormedLiteral(String),
  // Para el lexer, un UnexpectedEOF es cuando necesita mas caracteres para terminar un Token. por ahora no tenemos ningun caso
  // UnexpectedEOF,
  // UnterminatedToken,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
  pub(crate) kind: LexerErrorKind,
  pub(crate) span: Span,
}

impl LexerError {
  pub(crate) fn to_diagnostic(&self) -> Diagnostic {
    match &self.kind {
      LexerErrorKind::InvalidCharacter(c) => {
        Diagnostic::error(format!("el lexer detecto un caracter invalido {c}"))
      }
      LexerErrorKind::IllFormedLiteral(lit) => {
        Diagnostic::error(format!("el lexer detecto un literal mal formado {lit}"))
      } // LexerErrorKind::UnexpectedEOF => {
        //   Diagnostic::error("el lexer detecto un EOF inesperado".into())
        // } // LexerErrorKind::UnterminatedToken => Diagnostic::error("token inconcluso".into()),
    }
    .with_span(self.span.clone())
  }
}
