// Disenio para un parser extensible: basado en etapas/stages independientes

use crate::{
  ast::{Ast, Program},
  diagnostics::Diagnostic,
  semantic::SemanticResult,
};
pub(crate) type IrModule = ();

pub(crate) struct PipelineContext {
  pub(crate) source: String,
  pub(crate) ast: Option<Ast>,
  pub(crate) program: Option<Program>,
  pub(crate) semantic: Option<SemanticResult>,
  #[allow(dead_code)]
  pub(crate) ir: Option<IrModule>, // futuro
  pub(crate) diagnostics: Vec<Diagnostic>,
}

impl PipelineContext {
  pub(crate) fn start(source: String) -> Self {
    Self {
      source,
      ast: None,
      program: None,
      semantic: None,
      ir: None,
      diagnostics: Vec::new(),
    }
  }
}
