use crate::{
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::token::Token,
};

#[derive(Debug, Clone)]
pub(crate) enum ParserError {
  UnexpectedToken(Token),
  MissingToken,
  ChainedAssociativeOperator(Token),
  UnexpectedEOF(Token),
  // IdentifierExpected(Token),
}

impl ParserError {
  fn token_error_fmt(token: &Token, template: &str) -> Diagnostic {
    Diagnostic::error(template.replace("{}", &token.lexeme())).with_span(token.span().clone())
  }
}

impl Diagnosable for ParserError {
  fn to_diagnostic(&self) -> Diagnostic {
    match &self {
      Self::UnexpectedToken(token) => Self::token_error_fmt(token, "hubo un token inesperado '{}'"),
      Self::MissingToken => Diagnostic::error("hay un token faltante en el token stream".into()),
      Self::ChainedAssociativeOperator(token) => {
        Self::token_error_fmt(token, "operadores de comparacion '{}' no son asociativos")
      }
      Self::UnexpectedEOF(token) => Self::token_error_fmt(token, "hubo un EOF inesperado"),
    }
  }
}
