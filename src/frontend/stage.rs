// Vamos a hacer que cada fase del pipeline implemente el trait Stage.

use crate::frontend::{config::FrontendConfig, pipeline_context::PipelineContext};

pub(crate) trait Stage {
  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult;
}

pub(crate) enum StageResult {
  Continue,
  Stop,
}
