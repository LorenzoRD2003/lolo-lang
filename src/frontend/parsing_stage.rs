use crate::{
  frontend::{
    config::FrontendConfig,
    pipeline_context::PipelineContext,
    stage::{Stage, StageResult},
  },
  lexer::Lexer,
  parser::{Parser, TokenStream},
};

#[derive(Debug, Clone)]
pub(crate) struct ParsingStage;

impl Stage for ParsingStage {
  fn name(&self) -> &'static str {
    "Parsing"
  }

  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult {
    let lexer = Lexer::new(&ctx.source);
    let mut ts = TokenStream::new(lexer);
    let mut parser = Parser::new(&mut ts, &mut ctx.diagnostics);
    let program_opt = parser.parse_program();
    ctx.ast = Some(parser.into_ast());

    match program_opt {
      Some(p) => {
        ctx.program = Some(p);
        if !ctx.diagnostics.is_empty() && config.stop_after_parse_errors {
          StageResult::Stop
        } else {
          StageResult::Continue
        }
      }
      None => StageResult::Stop,
    }
  }
}
