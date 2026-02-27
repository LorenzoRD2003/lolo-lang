use crate::{
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::token::Token,
};

#[derive(Debug, Clone)]
pub(crate) enum ParserError {
  ChainedAssociativeOperator(Token),
  IdentifierExpected(Token),
  MissingToken,
  UnexpectedEOF(Token),
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
      Self::IdentifierExpected(token) => Self::token_error_fmt(
        token,
        "se esperaba un identificador de variable, pero se encontro {}".into(),
      ),
      Self::MissingToken => Diagnostic::error("hay un token faltante en el token stream".into()),
      Self::UnexpectedEOF(token) => Self::token_error_fmt(token, "hubo un EOF inesperado '{}'"),
      Self::UnexpectedToken(token) => Self::token_error_fmt(token, "hubo un token inesperado '{}'"),
    }
  }
}
