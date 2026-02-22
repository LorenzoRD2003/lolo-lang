use std::error::Error;
use std::fmt;

// TODO: indicar linea y posicion en los errores

#[derive(Debug)]
pub enum LexerError {
  InvalidCharacter {
    c: char,
    line: usize,
    column: usize,
  },
  IllFormedLiteral {
    literal: String,
    line: usize,
    column: usize,
  },
  UnexpectedEOF {
    line: usize,
    column: usize,
  },
}

impl fmt::Display for LexerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidCharacter { c, line, column } => write!(
        f,
        "Caracter inválido {c}, en la línea {line}, columna {column}."
      ),
      Self::IllFormedLiteral {
        literal,
        line,
        column,
      } => write!(
        f,
        "Literal mal formado {literal}, en la línea {line}, columna {column}."
      ),
      Self::UnexpectedEOF { line, column } => {
        write!(f, "EOF inesperado, en la línea {line}, columna {column}.")
      }
    }
  }
}

impl Error for LexerError {}
