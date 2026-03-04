// Disenio para un parser extensible: basado en etapas/stages independientes

use crate::{
  ast::{Ast, Program},
  diagnostics::Diagnostic,
  semantic::result::SemanticResult,
};
pub type IrModule = ();

pub struct PipelineContext {
  pub source: String,
  pub ast: Option<Ast>,
  pub program: Option<Program>,
  pub semantic: Option<SemanticResult>,
  pub ir: Option<IrModule>, // futuro
  pub diagnostics: Vec<Diagnostic>,
}

impl PipelineContext {
  pub fn start(source: String) -> Self {
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
