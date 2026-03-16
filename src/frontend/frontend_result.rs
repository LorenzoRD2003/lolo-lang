// Representa el resultado final del frontend.

use crate::{
  ast::Ast, diagnostics::Diagnostic, ir::IrModule, passes::PassStats, semantic::SemanticResult,
};

#[derive(Debug, Clone)]
pub struct FrontendResult {
  ast: Option<Ast>,
  semantic: Option<SemanticResult>,
  ir: Option<IrModule>,
  pass_stats: Vec<PassStats>,
  diagnostics: Vec<Diagnostic>,
}

impl FrontendResult {
  pub(crate) fn from(
    ast: Option<Ast>,
    semantic: Option<SemanticResult>,
    ir: Option<IrModule>,
    pass_stats: Vec<PassStats>,
    diagnostics: Vec<Diagnostic>,
  ) -> Self {
    Self {
      ast,
      semantic,
      ir,
      pass_stats,
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

  pub fn pass_stats_pretty(&self) -> Option<String> {
    if self.pass_stats.is_empty() {
      return None;
    }

    let mut out = String::new();
    for stat in &self.pass_stats {
      match stat {
        PassStats::Dce(dce) => out.push_str(&format!(
          "dce: removed_phis={}, removed_insts={}\n",
          dce.removed_phis, dce.removed_insts
        )),
        PassStats::Uce(uce) => out.push_str(&format!(
          "uce: removed_blocks={}, rewritten_jumps={}, rewritten_branches={}, removed_phi_inputs={}\n",
          uce.removed_blocks, uce.rewritten_jumps, uce.rewritten_branches, uce.removed_phi_inputs
        )),
      }
    }
    Some(out)
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
      && self.pass_stats == other.pass_stats
      && self.diagnostics == other.diagnostics
  }
}
