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
  pub(crate) fn new(ast: &'a Ast, graph: PhaseGraph<'a>, diagnostics: &'a mut Vec<Diagnostic>) -> Self {
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
mod tests {
  use super::*;
  use crate::{ast::ConstValue, parser::parse_program};

  #[test]
  fn semantic_analyzer_collects_all_metadata() {
    let source = r#"
      main {
        let x = 5;
        x = 10;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    {
      let mut analyzer =
        SemanticAnalyzer::new(&ast, PhaseGraph::default_semantic_graph(), &mut diagnostics);
      let _result = analyzer.analyze(&program);
    }
    assert!(diagnostics.is_empty());
  }

  #[test]
  fn semantic_analyzer_accumulates_diagnostics() {
    let source = r#"
      main {
        x = 10;
        let y = true;
        y = 5;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    {
      let mut analyzer =
        SemanticAnalyzer::new(&ast, PhaseGraph::default_semantic_graph(), &mut diagnostics);
      let _result = analyzer.analyze(&program);
    }
    // Lo importante: mas de un error
    assert!(diagnostics.len() >= 2);
  }

  #[test]
  fn semantic_analyzer_returns_result_even_with_errors() {
    let source = r#"
      main {
        x = 10;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    {
      let mut analyzer =
        SemanticAnalyzer::new(&ast, PhaseGraph::default_semantic_graph(), &mut diagnostics);
      let result = analyzer.analyze(&program);
      let _ = result.type_info;
    };
    assert!(!diagnostics.is_empty());
    // aun asi tenemos resultado -> no debe panickear
  }

  #[test]
  fn constant_info_flows_into_category_checker() {
    let source = r#"
      main {
        let x = 5;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    {
      let mut analyzer =
        SemanticAnalyzer::new(&ast, PhaseGraph::default_semantic_graph(), &mut diagnostics);
      let result = analyzer.analyze(&program);
      // Literal 5 debería ser constante, y su categoría debería incluir CONSTANT
      let compile_time_constant_info = &result.compile_time_constant_info;
      assert!(
        compile_time_constant_info
          .iter()
          .any(|(_, value)| value == &ConstValue::Int32(5))
      );
      let category_info = &result.category_info;
      assert!(category_info.iter().any(|(_, cat)| cat.is_constant()));
    }
    assert!(diagnostics.is_empty());
  }
}
