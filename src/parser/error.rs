use crate::{
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::token::Token,
};

#[derive(Debug, Clone)]
pub(crate) enum ParserError {
  UnexpectedToken(Token),
  MissingToken,
  ChainedAssociativeOperator(Token),
}

impl Diagnosable for ParserError {
  fn to_diagnostic(&self) -> Diagnostic {
    match &self {
      Self::UnexpectedToken(token) => {
        let lexeme = &token.lexeme;
        Diagnostic::error(format!("se detecto un token inesperado '{lexeme}'"))
          .with_span(token.span.clone())
      }
      Self::MissingToken => Diagnostic::error("hay un token faltante en el token stream".into()),
      Self::ChainedAssociativeOperator(token) => {
        let lexeme = &token.lexeme;
        Diagnostic::error(format!(
          "los operadores de comparacion no son asociativos, como '{lexeme}'"
        ))
        .with_span(token.span.clone())
      }
    }
  }
}
