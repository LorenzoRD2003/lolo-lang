use crate::{
  frontend::{
    FrontendConfig,
    pipeline_context::PipelineContext,
    stage::{Stage, StageResult},
  },
  semantic::semantic_analyzer::SemanticAnalyzer,
};

#[derive(Debug, Clone)]
pub struct SemanticStage;

impl Stage for SemanticStage {
  fn name(&self) -> &'static str {
    "semantic"
  }

  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult {
    let before_errors = ctx.diagnostics.len();
    let mut semantic_analyzer =
      SemanticAnalyzer::new(&ctx.ast.as_ref().unwrap(), &mut ctx.diagnostics);
    let result = semantic_analyzer.analyze(&ctx.program.as_ref().unwrap());

    ctx.semantic = Some(result);
    if ctx.diagnostics.len() > before_errors && config.stop_after_semantic_errors {
      StageResult::Stop
    } else {
      StageResult::Continue
    }
  }
}
