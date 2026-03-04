use crate::{
  common::Span,
  diagnostics::{Diagnosable, Diagnostic},
};

#[derive(Debug, Clone)]
pub enum MutabilityError {
  /// Se intento modificar una variable inmutable
  ImmutableVariable { name: String, span: Span },
}

impl Diagnosable for MutabilityError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::ImmutableVariable { name, span } => Diagnostic::error(format!(
        "se intento modificar la variable inmutable '{}'",
        name
      ))
      .with_span(span.clone()),
    }
  }
}
