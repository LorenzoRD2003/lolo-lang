// Responsable de los errores semanticos.

use crate::{
  ast::{BinaryOp, ConstValue},
  common::Span,
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
};

#[derive(Debug, Clone)]
pub enum CompileTimeConstantError {
  /// Una operacion de suma/resta/multiplicacion hizo overflow en 32 bits
  ArithmeticOverflow {
    op: BinaryOp,
    lhs: ConstValue,
    rhs: ConstValue,
    span: Span,
  },
  /// Intento de division por cero
  ZeroDivision { span: Span },
}

impl Diagnosable for CompileTimeConstantError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::ArithmeticOverflow { span, op, lhs, rhs } => {
        Diagnostic::error(format!("overflow evaluando {} {} {}", lhs, op, rhs))
          .with_span(span.clone())
      }
      Self::ZeroDivision { span } => {
        Diagnostic::error(format!("division por cero encontrada")).with_span(span.clone())
      }
    }
  }
}
