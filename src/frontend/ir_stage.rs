use crate::{
  frontend::{
    config::FrontendConfig,
    pipeline_context::PipelineContext,
    stage::{Stage, StageResult},
  },
  ir::LoweringCtx,
};

#[derive(Debug, Clone)]
pub(crate) struct IrStage;

impl Stage for IrStage {
  fn name(&self) -> &'static str {
    "Semantic"
  }

  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult {
    let before_errors = ctx.diagnostics.len();
    let result = LoweringCtx::lower_to_ir(
      ctx.program.as_ref().unwrap(),
      ctx.ast.as_ref().unwrap(),
      &ctx.semantic.as_ref().unwrap(),
      &mut ctx.diagnostics,
    );
    ctx.ir = Some(result);
    if ctx.diagnostics.len() > before_errors && config.stop_after_semantic_errors {
      StageResult::Stop
    } else {
      StageResult::Continue
    }
  }
}
