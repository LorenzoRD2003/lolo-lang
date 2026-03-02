// Representa el resultado final del frontend.

use crate::{
  ast::ast::Ast, diagnostics::diagnostic::Diagnostic, semantic::semantic_analyzer::SemanticResult,
};

#[derive(Debug, Clone)]
pub struct FrontendResult {
  pub ast: Option<Ast>,
  pub semantic: Option<SemanticResult>,
  pub diagnostics: Vec<Diagnostic>,
}

impl FrontendResult {
  pub fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
    Self {
      ast: None,
      semantic: None,
      diagnostics,
    }
  }

  pub fn has_errors(&self) -> bool {
    self.diagnostics.is_empty()
  }

  pub fn into_diagnostics(self) -> Vec<Diagnostic> {
    self.diagnostics
  }

  pub fn expect_semantic(self) -> SemanticResult {
    todo!()
  }
}
