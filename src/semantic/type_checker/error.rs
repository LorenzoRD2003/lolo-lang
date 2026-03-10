// Responsable de los errores semanticos.

use crate::{
  ast::{BinaryOp, UnaryOp},
  common::Span,
  diagnostics::{Diagnosable, Diagnostic},
  semantic::types::SemanticType,
};

#[derive(Debug, Clone)]
pub(crate) enum TypeError {
  /// Una operacion binaria es invalida
  InvalidBinaryOperation {
    op: BinaryOp,
    lhs: SemanticType,
    rhs: SemanticType,
    span: Span,
  },
  /// Una operacion unaria es invalida
  InvalidUnaryOperation {
    op: UnaryOp,
    operand: SemanticType,
    span: Span,
  },
  /// Se espera un tipo y se recibe otro
  MismatchedTypes {
    expected: SemanticType,
    found: SemanticType,
    span: Span,
  },
  /// Se encuentra una condicion no booleana en un If
  NonBooleanCondition { found: SemanticType, span: Span },
}

impl Diagnosable for TypeError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::MismatchedTypes {
        expected,
        found,
        span,
      } => Diagnostic::error(format!(
        "mismatch de tipos: se esperaba {}, pero se encontro {}",
        expected, found
      ))
      .with_span(span.clone()),
      Self::InvalidBinaryOperation { op, lhs, rhs, span } => Diagnostic::error(format!(
        "operacion binaria invalida: {}, el LHS es de tipo {} y el RHS es de tipo {}",
        op, lhs, rhs
      ))
      .with_span(span.clone()),
      Self::InvalidUnaryOperation { op, operand, span } => Diagnostic::error(format!(
        "operacion unaria invalida: {}, el operando es de tipo {}",
        op, operand
      ))
      .with_span(span.clone()),
      Self::NonBooleanCondition { found, span } => Diagnostic::error(format!(
        "se encontro una condicion no booleana, de tipo {}",
        found
      ))
      .with_span(span.clone()),
    }
  }
}
