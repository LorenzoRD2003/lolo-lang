use crate::{
  ast::ExprId,
  common::Span,
  diagnostics::{Diagnosable, Diagnostic},
  semantic::SymbolId,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LoweringError {
  /// Falta el simbolo resuelto para una expresion de variable.
  MissingSymbol { expr_id: ExprId, span: Span },
  /// Existe simbolo semantico pero no tiene un valor SSA vivo en el punto actual del CFG.
  MissingSsaValueForSymbol { symbol: SymbolId, span: Span },
  /// El lowering recibio una expresion con tipo de error semantico y no puede generar IR valida para ella.
  CannotLowerErrorTypedExpr { expr_id: ExprId, span: Span },
}

impl Diagnosable for LoweringError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::MissingSymbol { expr_id, span } => Diagnostic::error(format!(
        "no se pudo bajar a IR: la expresion {:?} no tiene simbolo resuelto",
        expr_id
      ))
      .with_span(span.clone()),
      Self::MissingSsaValueForSymbol {
        symbol: symbol_id,
        span,
      } => Diagnostic::error(format!(
        "no se pudo bajar a IR: el simbolo {:?} no tiene un valor SSA disponible",
        symbol_id
      ))
      .with_span(span.clone()),
      Self::CannotLowerErrorTypedExpr { expr_id, span } => Diagnostic::error(format!(
        "no se pudo bajar a IR: la expresion {:?} tiene un tipo semantico invalido",
        expr_id
      ))
      .with_span(span.clone()),
    }
  }
}
