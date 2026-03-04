// Responsable de los errores de chequeo de categorias.

use crate::{
  common::Span,
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
};

#[derive(Debug, Clone)]
pub enum CategoryError {
  /// Se esperaba una PlaceExpr (a la hora de recibir un valor).
  ExpectedPlaceExpression { span: Span },
  /// Se esperaba una ValueExpr (a la hora de emitir un valor).
  ExpectedValueExpression { span: Span },
  /// Se esperaba una ConstantExpr (a la hora de emitir un valor).
  ExpectedConstantExpression { span: Span },
}

impl Diagnosable for CategoryError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::ExpectedPlaceExpression { span } => {
        Diagnostic::error("se esperaba una place expression".into()).with_span(span.clone())
      }
      Self::ExpectedValueExpression { span } => {
        Diagnostic::error("se esperaba una value expression".into()).with_span(span.clone())
      }
      Self::ExpectedConstantExpression { span } => {
        Diagnostic::error("se esperaba una constant expression".into()).with_span(span.clone())
      }
    }
  }
}
