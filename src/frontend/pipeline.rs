// Responsabilidad: ejecuta fases y aplica

use crate::{
  ast::{ast::Ast, program::Program},
  diagnostics::diagnostic::Diagnostic,
  frontend::{config::FrontendConfig, frontend_result::FrontendResult},
  lexer::lexer::Lexer,
  parser::{parser::Parser, token_stream::TokenStream},
  semantic::semantic_analyzer::{SemanticAnalyzer, SemanticResult},
};

enum PhaseOutcome<T, U> {
  Continue(T),
  Stop(U),
}

#[derive(Debug, Clone)]
pub struct FrontendPipeline;

impl FrontendPipeline {
  pub fn run(source: &str, config: &FrontendConfig) -> FrontendResult {
    let mut diagnostics = Vec::new();

    // Fase sintactica: Lex + parse
    let (ast, program) = match Self::parsing_phase(source, config, &mut diagnostics) {
      PhaseOutcome::Continue(data) => data,
      PhaseOutcome::Stop(ast_opt) => return FrontendResult::from(ast_opt, None, diagnostics),
    };

    // Fase semantica
    match Self::semantic_phase(config, &ast, &program, &mut diagnostics) {
      PhaseOutcome::Continue(semantic) => {
        FrontendResult::from(Some(ast), Some(semantic), diagnostics)
      }
      PhaseOutcome::Stop(()) => FrontendResult::from(Some(ast), None, diagnostics),
    }
  }

  fn parsing_phase(
    source: &str,
    config: &FrontendConfig,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> PhaseOutcome<(Ast, Program), Option<Ast>> {
    let lexer = Lexer::new(source);
    let mut ts = TokenStream::new(lexer);
    let mut parser = Parser::new(&mut ts, diagnostics);
    let program_opt = parser.parse_program();
    let ast = parser.into_ast();
    match program_opt {
      Some(program) => PhaseOutcome::Continue((ast, program)),
      None => {
        if config.stop_after_parse_errors {
          PhaseOutcome::Stop(None)
        } else {
          PhaseOutcome::Stop(Some(ast))
        }
      }
    }
  }

  fn semantic_phase(
    config: &FrontendConfig,
    ast: &Ast,
    program: &Program,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> PhaseOutcome<SemanticResult, ()> {
    let before_errors = diagnostics.len();
    let mut semantic_analyzer = SemanticAnalyzer::new(&ast, diagnostics);
    let result = semantic_analyzer.analyze(program);
    if diagnostics.len() > before_errors && config.stop_after_semantic_errors {
      PhaseOutcome::Stop(())
    } else {
      PhaseOutcome::Continue(result)
    }
  }
}
