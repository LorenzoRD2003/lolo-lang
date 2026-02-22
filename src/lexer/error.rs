use std::error::Error;
use std::fmt;

use crate::common::span::Span;

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
  InvalidCharacter(char),
  IllFormedLiteral(String),
  UnterminatedToken,
  UnexpectedEOF,
}

// La linea/columna al reportar el error se deriva del Span.
#[derive(Debug, Clone)]
pub struct LexerError {
  pub(crate) kind: LexerErrorKind,
  pub(crate) span: Span,
}

// Esto va a haber que cambiarlo. Porque LexerError describe que salio mal, pero en el modulo diagnostics decidimos como mostrarlo
impl fmt::Display for LexerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.kind {
      LexerErrorKind::InvalidCharacter(c) => {
        write!(f, "Caracter inválido {c}, en la línea, columna.")
      }
      LexerErrorKind::IllFormedLiteral(literal) => {
        write!(f, "Literal mal formado {literal}, en la línea, columna.")
      }
      LexerErrorKind::UnterminatedToken => write!(f, "Token no terminado, en la línea, columna."),
      LexerErrorKind::UnexpectedEOF => write!(f, "EOF inesperado, en la línea, columna."),
    }
  }
}

impl Error for LexerError {}
