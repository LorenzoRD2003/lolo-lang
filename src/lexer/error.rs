use crate::{
  common::span::Span,
  diagnostics::{
    diagnostic::{Diagnosable, Diagnostic},
    label::Label,
  },
};

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

impl Diagnosable for LexerError {
  fn to_diagnostic(&self) -> Diagnostic {
    let Span { start, end } = self.span;
    match &self.kind {
      LexerErrorKind::InvalidCharacter(c) => {
        Diagnostic::error(format!("el lexer detecto un caracter invalido {c}"))
          .with_label(Label::primary(start..end, None))
      }
      LexerErrorKind::IllFormedLiteral(lit) => {
        Diagnostic::error(format!("el lexer detecto un literal mal formado {lit}"))
          .with_label(Label::primary(start..end, None))
      } // LexerErrorKind::UnexpectedEOF => {
        //   Diagnostic::error("el lexer detecto un EOF inesperado".into())
        // } // LexerErrorKind::UnterminatedToken => Diagnostic::error("token inconcluso".into()),
    }
    .with_span(start..end)
  }
}
