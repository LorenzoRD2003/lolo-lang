use crate::{
  frontend::{
    config::FrontendConfig,
    pipeline_context::PipelineContext,
    stage::{Stage, StageResult},
  },
  semantic::{PhaseGraph, SemanticAnalyzer},
};

#[derive(Debug, Clone)]
pub(crate) struct SemanticStage;

impl Stage for SemanticStage {
  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult {
    let before_errors = ctx.diagnostics.len();
    let result = {
      let mut semantic_analyzer = SemanticAnalyzer::new(
        ctx.ast.as_ref().unwrap(),
        PhaseGraph::default_semantic_graph(),
        &mut ctx.diagnostics,
      );
      semantic_analyzer.analyze(ctx.program.as_ref().unwrap())
    };
    ctx.semantic = Some(result);
    if ctx.diagnostics.len() > before_errors && config.stop_after_semantic_errors {
      StageResult::Stop
    } else {
      StageResult::Continue
    }
  }
}
