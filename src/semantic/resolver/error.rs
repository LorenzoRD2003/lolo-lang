// Responsable de los errores del name resolver.

use crate::{
  ast::expr::VarId,
  common::span::Span,
  diagnostics::{
    diagnostic::{Diagnosable, Diagnostic},
    label::Label,
  },
};

#[derive(Debug, Clone)]
pub enum ResolverError {
  /// Redeclaracion ilegal en el mismo scope
  RedeclaredVariable {
    name: VarId,
    span: Span,
    previous_span: Span,
  },
  /// Uso de variable inexistente
  UndefinedVariable { name: VarId, span: Span },
}

impl Diagnosable for ResolverError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::RedeclaredVariable {
        name,
        span,
        previous_span,
      } => {
        Diagnostic::error(format!(
          "la variable '{}' ya fue declarada en este scope",
          name.0
        ))
        // primary_span apunta al span de la nueva declaracion, igual que el label principal
        .with_span(span.clone())
        .with_label(Label::primary(
          span.clone(),
          Some(format!("redeclara '{}'", name.0)),
        ))
        // label secundaria para la declaracion de variable original
        .with_label(Label::secondary(
          previous_span.clone(),
          Some("declaracion original en".into()),
        ))
      }
      Self::UndefinedVariable { name, span } => {
        Diagnostic::error(format!("variable '{}' indefinida", name.0)).with_span(span.clone())
      }
    }
  }
}
