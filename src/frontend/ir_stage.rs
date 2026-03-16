use crate::{
  frontend::{
    config::FrontendConfig,
    pipeline_context::PipelineContext,
    stage::{Stage, StageResult},
  },
  ir::LoweringCtx,
  passes::{DcePass, IrPass, PassContext, UcePass},
};

#[derive(Debug, Clone)]
pub(crate) struct IrStage;

impl Stage for IrStage {
  fn name(&self) -> &'static str {
    "Intermediate Representation (IR)"
  }

  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult {
    let before_errors = ctx.diagnostics.len();
    let mut result = LoweringCtx::lower_to_ir(
      ctx.program.as_ref().unwrap(),
      ctx.ast.as_ref().unwrap(),
      ctx.semantic.as_ref().unwrap(),
      &mut ctx.diagnostics,
    );

    if let Ok(pass_ctx) = PassContext::from_module(&result) {
      let uce_stats = UcePass.run(&mut result, &pass_ctx);
      let dce_stats = DcePass.run(&mut result, &pass_ctx);
      if config.show_pass_stats {
        ctx.pass_stats.push(uce_stats);
        ctx.pass_stats.push(dce_stats);
      }
    }

    // Verificacion estructural/tipada de IR habilitada por feature de compilacion.
    #[cfg(feature = "ir-verify")]
    result.verify(&mut ctx.diagnostics);

    ctx.ir = Some(result);
    if ctx.diagnostics.len() > before_errors && config.stop_after_semantic_errors {
      StageResult::Stop
    } else {
      StageResult::Continue
    }
  }
}
