// Responsabilidad: Mantener el vinculo entre IR y codigo fuente.
// Es util para errores posteriores, trazabilidad y debugging.

use std::collections::HashMap;

use crate::{
  ast::ExprId,
  common::Span,
  ir::ids::{InstId, ValueId},
  semantic::SymbolId,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IrSourceMap {
  /// Vinculo entre instrucciones y el span que ocupaba en el codigo fuente original.
  inst_to_span: HashMap<InstId, Span>,
  /// Vinculo entre valores SSA y expresiones del AST
  value_to_expr: HashMap<ValueId, ExprId>,
  /// Vinculo entre valores SSA y simbolos de la fuente
  value_to_symbol: HashMap<ValueId, SymbolId>,
}

impl IrSourceMap {
  pub(crate) fn new() -> Self {
    Self {
      inst_to_span: HashMap::new(),
      value_to_expr: HashMap::new(),
      value_to_symbol: HashMap::new(),
    }
  }
}
