use crate::{
  ast::{Ast, Program},
  diagnostics::Diagnostic,
  ir::{LoweringCtx, module::IrModule},
  parser::parse_program,
  semantic::{PhaseGraph, SemanticAnalyzer, SemanticResult},
};

pub(crate) fn parse_and_analyze(source: &str) -> (Ast, Program, SemanticResult, Vec<Diagnostic>) {
  let (ast, program) = parse_program(source);
  let mut diagnostics = Vec::new();
  let semantic = {
    let mut analyzer =
      SemanticAnalyzer::new(&ast, PhaseGraph::default_semantic_graph(), &mut diagnostics);
    analyzer.analyze(&program)
  };
  (ast, program, semantic, diagnostics)
}

pub(crate) fn lower_source(source: &str) -> (IrModule, Vec<Diagnostic>) {
  let (ast, program, semantic, mut diagnostics) = parse_and_analyze(source);
  let ir = LoweringCtx::lower_to_ir(&program, &ast, &semantic, &mut diagnostics);
  (ir, diagnostics)
}
