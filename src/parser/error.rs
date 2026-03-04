use crate::{
  common::span::Span,
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::token::Token,
};

#[derive(Debug, Clone)]
pub enum ParserError {
  ChainedAssociativeOperator(Token),
  StatementAfterReturn(Span),
  MainMustBeBlock(Span),
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
    match self {
      Self::ChainedAssociativeOperator(token) => {
        Self::token_error_fmt(token, "operadores de comparacion '{}' no son asociativos")
      }
      Self::MainMustBeBlock(span) => {
        Diagnostic::error("main debe ser un bloque".into()).with_span(span.clone())
      }
      Self::StatementAfterReturn(span) => {
        Diagnostic::error("se detecto un statement luego de un terminador de bloque".into())
          .with_span(span.clone())
      }
      Self::UnexpectedEOF => Diagnostic::error("hubo un EOF inesperado".into()),
      Self::UnexpectedToken(token) => Self::token_error_fmt(token, "hubo un token inesperado '{}'"),
    }
  }
}
