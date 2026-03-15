// Representa el resultado final del frontend.

use crate::{ast::Ast, diagnostics::Diagnostic, ir::IrModule, semantic::SemanticResult};

#[derive(Debug, Clone)]
pub struct FrontendResult {
  ast: Option<Ast>,
  semantic: Option<SemanticResult>,
  ir: Option<IrModule>,
  diagnostics: Vec<Diagnostic>,
}

impl FrontendResult {
  pub(crate) fn from(
    ast: Option<Ast>,
    semantic: Option<SemanticResult>,
    ir: Option<IrModule>,
    diagnostics: Vec<Diagnostic>,
  ) -> Self {
    Self {
      ast,
      semantic,
      ir,
      diagnostics,
    }
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn has_diagnostics(&self) -> bool {
    !self.diagnostics().is_empty()
  }

  pub fn ir_pretty(&self) -> Option<String> {
    self.ir.as_ref().map(|ir| ir.pretty())
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

impl PartialEq for FrontendResult {
  fn eq(&self, other: &Self) -> bool {
    self.ast == other.ast
      && self.semantic == other.semantic
      && self.diagnostics == other.diagnostics
  }
}
