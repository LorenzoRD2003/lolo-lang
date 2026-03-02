// El frontend debe:
// Recibir codigo fuente (&str), y ejecutar el Lexer, Parser, SemanticAnalyzer
// Devolver AST, SemanticResult, Diagnostics acumulados
// Definir politica de errores
// Debe ser la unica puerta de entrada publica del compilador.
// Orquesta lexer, parser, semantic analyzer

use crate::{
  frontend::{FrontendConfig, FrontendResult},
  lexer::lexer::Lexer,
  parser::{parser::Parser, token_stream::TokenStream},
  semantic::semantic_analyzer::SemanticAnalyzer,
};

#[derive(Debug, Clone)]
pub struct Frontend {
  config: FrontendConfig,
}

impl Frontend {
  pub fn new(config: FrontendConfig) -> Self {
    Self { config }
  }

  /// Compila el codigo fuente
  pub fn compile(&self, source: &str) -> FrontendResult {
    let mut diagnostics = Vec::new();

    // Fase sintactica: Lex + parse
    let lexer = Lexer::new(source);
    let mut ts = TokenStream::new(lexer);
    let mut parser = Parser::new(&mut ts, &mut diagnostics);
    let (ast, program) = if let Some(program) = parser.parse_program() {
      (parser.into_ast(), program)
    } else {
      return FrontendResult::from_diagnostics(diagnostics);
    };

    // Fase semantica
    let mut semantic_analyzer = SemanticAnalyzer::new(&ast, &mut diagnostics);
    let result = semantic_analyzer.analyze(&program);

    FrontendResult {
      ast: Some(ast),
      semantic: Some(result),
      diagnostics,
    }
  }
}
