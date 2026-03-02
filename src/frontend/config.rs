// Responsabilidad: controlar la politica del frontend

#[derive(Debug, Clone)]
pub struct FrontendConfig {
  /// Seria el --dump-ast.
  pub show_ast: bool,
  /// Seria el --dump-semantic.
  pub show_semantic_result: bool,
  /// Deja de compilar si encuentra errores en la fase de Parsing.
  pub stop_after_parse_errors: bool,
  /// Deja de compilar si encuentra errores en la fase de analisis semantico.
  pub stop_after_semantic_errors: bool,
}

impl FrontendConfig {
  /// Modo CLI -> reporta todos los errores
  pub fn cli_mode() -> Self {
    Self {
      show_ast: false,
      show_semantic_result: false,
      stop_after_parse_errors: false,
      stop_after_semantic_errors: false,
    }
  }

  /// Modo strict/test -> fallar inmediatamente si algo esta mal
  pub fn strict_mode() -> Self {
    Self {
      show_ast: false,
      show_semantic_result: false,
      stop_after_parse_errors: true,
      stop_after_semantic_errors: true,
    }
  }

  /// Modo IDE -> usar AST aunque haya errores, mejorar experiencia del usuario
  pub fn ide_mode() -> Self {
    Self {
      show_ast: true,
      show_semantic_result: true,
      stop_after_parse_errors: false,
      stop_after_semantic_errors: false,
    }
  }
}
