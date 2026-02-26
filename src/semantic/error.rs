// Responsable de los errores semanticos.

use crate::{
  ast::expr::VarId,
  common::span::Span,
  diagnostics::{
    diagnostic::{Diagnosable, Diagnostic},
    label::Label,
  },
  semantic::types::Type,
};

#[derive(Debug, Clone)]
pub(crate) enum SemanticError {
  /// Uso de variable inexistente
  UndefinedVariable { name: VarId, span: Span },
  /// Redeclaracion ilegal en el mismo scope
  RedeclaredVariable {
    name: VarId,
    span: Span,
    previous_span: Span,
  },
  /// Intento de asignar a algo no asignable
  InvalidAssignmentTarget { span: Span },
  /// Se espera un tipo y se recibe otro
  TypeMismatch {
    expected: Type,
    found: Type,
    span: Span,
  },
}

impl Diagnosable for SemanticError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::UndefinedVariable { name, span } => {
        Diagnostic::error(format!("variable '{}' indefinida", name.0)).with_span(span.clone())
      }
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
          Some("declaración original aca".into()),
        ))
      }
      Self::InvalidAssignmentTarget { span } => {
        Diagnostic::error("target de asignacion invalido".into()).with_span(span.clone())
      }
      Self::TypeMismatch {
        expected,
        found,
        span,
      } => Diagnostic::error(format!(
        "mismatch de tipos: se esperaba {}, pero se encontro {}",
        expected.to_string(),
        found.to_string()
      ))
      .with_span(span.clone()),
    }
  }
}
