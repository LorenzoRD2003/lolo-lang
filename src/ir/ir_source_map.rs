// Responsabilidad: Mantener el vinculo entre IR y codigo fuente.
// Es util para errores posteriores, trazabilidad y debugging.

use std::collections::HashMap;

use crate::{
  ast::ExprId,
  common::Span,
  ir::ids::{InstId, LocalId, ValueId},
  semantic::SymbolId,
};

#[derive(Debug, Clone)]
pub(crate) struct IrSourceMap {
  /// Vinculo entre instrucciones y el span que ocupaba en el codigo fuente original.
  inst_to_span: HashMap<InstId, Span>,
  /// Vinculo entre valores de la IR y expresiones del AST
  value_to_expr: HashMap<ValueId, ExprId>,
  /// Vinculo entre locales de la IR y simbolos de la resolucion semantica.
  local_to_symbol: HashMap<LocalId, SymbolId>,
}

impl IrSourceMap {
  pub(crate) fn new() -> Self {
    Self {
      inst_to_span: HashMap::new(),
      value_to_expr: HashMap::new(),
      local_to_symbol: HashMap::new(),
    }
  }
}
