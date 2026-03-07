use std::time::Instant;

use crate::frontend::{
  config::FrontendConfig,
  frontend_result::FrontendResult,
  parsing_stage::ParsingStage,
  pipeline_context::PipelineContext,
  semantic_stage::SemanticStage,
  stage::{Stage, StageResult},
};

pub(crate) struct FrontendPipeline {
  /// Un vector de punteros en heap a objetos que implementan el trait Stage, con dynamic dispatch
  stages: Vec<Box<dyn Stage>>,
}

impl FrontendPipeline {
  pub(crate) fn default() -> Self {
    Self {
      stages: vec![Box::new(ParsingStage), Box::new(SemanticStage)],
    }
  }

  pub(crate) fn run(&self, source: &str, config: &FrontendConfig) -> FrontendResult {
    let mut ctx = PipelineContext::start(source.into());
    for stage in &self.stages {
      let start = Instant::now();
      let result = stage.run(&mut ctx, config);
      let elapsed = start.elapsed();

      if config.show_stage_timings {
        eprintln!("[timing] {}: {:?}", stage.name(), elapsed);
      }

      match result {
        StageResult::Continue => {}
        StageResult::Stop => break,
      }
    }

    FrontendResult::from(ctx.ast, ctx.semantic, ctx.diagnostics)
  }
}
