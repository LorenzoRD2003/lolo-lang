// Vamos a hacer que cada fase del pipeline implemente el trait Stage.

use crate::frontend::{FrontendConfig, pipeline_context::PipelineContext};

pub trait Stage {
  fn name(&self) -> &'static str;
  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult;
}

pub enum StageResult {
  Continue,
  Stop,
}
