// Representa el resultado final del frontend.

use crate::{
  ast::ast::Ast, diagnostics::diagnostic::Diagnostic, semantic::semantic_analyzer::SemanticResult,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FrontendResult {
  ast: Option<Ast>,
  semantic: Option<SemanticResult>,
  diagnostics: Vec<Diagnostic>,
}

impl FrontendResult {
  pub fn from(
    ast: Option<Ast>,
    semantic: Option<SemanticResult>,
    diagnostics: Vec<Diagnostic>,
  ) -> Self {
    Self {
      ast,
      semantic,
      diagnostics,
    }
  }

  pub fn into_diagnostics(self) -> Vec<Diagnostic> {
    self.diagnostics
  }

  pub fn ast(&self) -> Option<&Ast> {
    self.ast.as_ref()
  }

  pub fn semantic(&self) -> Option<&SemanticResult> {
    self.semantic.as_ref()
  }
}
