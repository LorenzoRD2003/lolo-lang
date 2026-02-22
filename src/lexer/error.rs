use std::error::Error;
use std::fmt;

// TODO: indicar linea y posicion en los errores

#[derive(Debug)]
pub enum LexerError {
  InvalidCharacter(char),
  IllFormedLiteral(String),
  UnexpectedEOF,
}

impl fmt::Display for LexerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LexerError::InvalidCharacter(c) => write!(f, "Caracter inválido {c}."),
      LexerError::IllFormedLiteral(literal) => write!(f, "Literal mal formado {literal}."),
      LexerError::UnexpectedEOF => write!(f, "EOF inesperado."),
    }
  }
}

impl Error for LexerError {}
