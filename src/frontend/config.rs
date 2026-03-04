// Responsabilidad: controlar la politica del frontend

#[derive(Debug, Clone)]
pub(crate) struct FrontendConfig {
  /// Seria el --dump-ast.
  pub(crate) show_ast: bool,
  /// Seria el --dump-semantic.
  pub(crate) show_semantic_result: bool,
  /// Deja de compilar si encuentra errores en la fase de Parsing.
  pub(crate) stop_after_parse_errors: bool,
  /// Deja de compilar si encuentra errores en la fase de analisis semantico.
  pub(crate) stop_after_semantic_errors: bool,
}

impl FrontendConfig {
  /// Modo CLI -> reporta todos los errores
  pub(crate) fn cli_mode() -> Self {
    Self {
      show_ast: false,
      show_semantic_result: false,
      stop_after_parse_errors: false,
      stop_after_semantic_errors: false,
    }
  }

  /// Modo strict/test -> fallar inmediatamente si algo esta mal
  pub(crate) fn strict_mode() -> Self {
    Self {
      show_ast: false,
      show_semantic_result: false,
      stop_after_parse_errors: true,
      stop_after_semantic_errors: true,
    }
  }

  /// Modo IDE -> usar AST aunque haya errores, mejorar experiencia del usuario
  pub(crate) fn ide_mode() -> Self {
    Self {
      show_ast: true,
      show_semantic_result: true,
      stop_after_parse_errors: false,
      stop_after_semantic_errors: false,
    }
  }
}
