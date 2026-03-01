use crate::{
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::token::Token,
};

#[derive(Debug, Clone)]
pub enum ParserError {
  ChainedAssociativeOperator(Token),
  UnexpectedEOF,
  UnexpectedToken(Token),
}

impl ParserError {
  fn token_error_fmt(token: &Token, template: &str) -> Diagnostic {
    Diagnostic::error(template.replace("{}", &token.lexeme())).with_span(token.span().clone())
  }
}

impl Diagnosable for ParserError {
  fn to_diagnostic(&self) -> Diagnostic {
    match &self {
      Self::ChainedAssociativeOperator(token) => {
        Self::token_error_fmt(token, "operadores de comparacion '{}' no son asociativos")
      }
      Self::UnexpectedEOF => Diagnostic::error("hubo un EOF inesperado".into()),
      Self::UnexpectedToken(token) => Self::token_error_fmt(token, "hubo un token inesperado '{}'"),
    }
  }
}
