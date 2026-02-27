// Responsable de los errores semanticos.

use crate::{
  ast::expr::{BinaryOp, ConstValue, VarId},
  common::span::Span,
  diagnostics::{
    diagnostic::{Diagnosable, Diagnostic},
    label::Label,
  },
  semantic::types::Type,
};

#[derive(Debug, Clone)]
pub enum SemanticError {
  /// Una operacion de suma/resta/multiplicacion hizo overflow en 32 bits
  ArithmeticOverflow {
    span: Span,
    op: BinaryOp,
    lhs: ConstValue,
    rhs: ConstValue,
  },
  /// Se esperaba una PlaceExpr (a la hora de recibir un valor).
  ExpectedPlaceExpression { span: Span },
  /// Se esperaba una ValueExpr (a la hora de emitir un valor).
  ExpectedValueExpression { span: Span },
  /// Redeclaracion ilegal en el mismo scope
  RedeclaredVariable {
    name: VarId,
    span: Span,
    previous_span: Span,
  },
  /// Se espera un tipo y se recibe otro
  TypeMismatch {
    expected: Type,
    found: Type,
    span: Span,
  },
  /// Uso de variable inexistente
  UndefinedVariable { name: VarId, span: Span },
  /// Intento de division por cero
  ZeroDivision { span: Span },
}

impl Diagnosable for SemanticError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::ArithmeticOverflow { span, op, lhs, rhs } => {
        Diagnostic::error(format!("overflow evaluando {} {} {}", lhs, op, rhs))
          .with_span(span.clone())
      }
      Self::ExpectedPlaceExpression { span } => {
        Diagnostic::error("se esperaba una place expression (para recibir un valor)".into())
          .with_span(span.clone())
      }
      Self::ExpectedValueExpression { span } => {
        Diagnostic::error("se esperaba una value expression (para emitir un valor)".into())
          .with_span(span.clone())
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
      Self::UndefinedVariable { name, span } => {
        Diagnostic::error(format!("variable '{}' indefinida", name.0)).with_span(span.clone())
      }
      Self::ZeroDivision { span } => {
        Diagnostic::error(format!("division por cero encontrada")).with_span(span.clone())
      }
    }
  }
}
