use crate::{
  ast::{Ast, Program},
  diagnostics::Diagnostic,
  semantic::{
    context::SemanticContext, phase_executor::Executor, phase_graph::PhaseGraph,
    result::SemanticResult,
  },
};

pub(crate) struct SemanticAnalyzer<'a> {
  ast: &'a Ast,
  graph: PhaseGraph<'a>,
  diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> SemanticAnalyzer<'a> {
  pub(crate) fn new(
    ast: &'a Ast,
    graph: PhaseGraph<'a>,
    diagnostics: &'a mut Vec<Diagnostic>,
  ) -> Self {
    Self {
      ast,
      graph,
      diagnostics,
    }
  }

  pub(crate) fn analyze(&mut self, program: &Program) -> SemanticResult {
    let mut ctx = SemanticContext::new(self.diagnostics);
    Executor::execute(self.ast, program, &mut self.graph, &mut ctx);
    ctx.into()
  }
}

#[cfg(test)]
mod tests;
