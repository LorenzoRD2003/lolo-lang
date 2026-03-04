// Representa el resultado final del frontend.

use crate::{ast::Ast, diagnostics::Diagnostic, semantic::SemanticResult};

#[derive(Debug, Clone, PartialEq)]
pub struct FrontendResult {
  ast: Option<Ast>,
  semantic: Option<SemanticResult>,
  diagnostics: Vec<Diagnostic>,
}

impl FrontendResult {
  pub(crate) fn from(
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

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn has_diagnostics(&self) -> bool {
    !self.diagnostics().is_empty()
  }

  #[cfg(test)]
  pub(crate) fn ast(&self) -> Option<&Ast> {
    self.ast.as_ref()
  }

  #[cfg(test)]
  pub(crate) fn semantic(&self) -> Option<&SemanticResult> {
    self.semantic.as_ref()
  }

  #[cfg(test)]
  pub(crate) fn into_diagnostics(self) -> Vec<Diagnostic> {
    self.diagnostics
  }
}
