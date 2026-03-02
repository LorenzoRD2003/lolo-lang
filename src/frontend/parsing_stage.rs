use crate::{
  frontend::{
    FrontendConfig,
    pipeline_context::PipelineContext,
    stage::{Stage, StageResult},
  },
  lexer::lexer::Lexer,
  parser::{parser::Parser, token_stream::TokenStream},
};

#[derive(Debug, Clone)]
pub struct ParsingStage;

impl Stage for ParsingStage {
  fn name(&self) -> &'static str {
    "parsing"
  }

  fn run(&self, ctx: &mut PipelineContext, config: &FrontendConfig) -> StageResult {
    let lexer = Lexer::new(&ctx.source);
    let mut ts = TokenStream::new(lexer);
    let mut parser = Parser::new(&mut ts, &mut ctx.diagnostics);
    let program_opt = parser.parse_program();
    ctx.ast = Some(parser.into_ast());

    let program = match program_opt {
      Some(p) => p,
      None => return StageResult::Stop,
    };

    ctx.program = Some(program);
    if !ctx.diagnostics.is_empty() && config.stop_after_parse_errors {
      StageResult::Stop
    } else {
      StageResult::Continue
    }
  }
}
