// Responsabilidad: controlar la politica del frontend

#[derive(Debug, Clone)]
pub struct FrontendConfig {
  /// Seria el --dump-ast.
  #[allow(dead_code)]
  pub(crate) show_ast: bool,
  /// Seria el --dump-semantic.
  #[allow(dead_code)]
  pub(crate) show_semantic_result: bool,
  /// Deja de compilar si encuentra errores en la fase de Parsing.
  pub(crate) stop_after_parse_errors: bool,
  /// Deja de compilar si encuentra errores en la fase de analisis semantico.
  pub(crate) stop_after_semantic_errors: bool,
  /// Indica cuanto tiempo tardo cada stage.
  pub(crate) show_stage_timings: bool,
}

impl FrontendConfig {
  /// Modo CLI -> reporta todos los errores
  pub fn cli_mode() -> Self {
    Self {
      show_ast: false,
      show_semantic_result: false,
      stop_after_parse_errors: false,
      stop_after_semantic_errors: false,
      show_stage_timings: false,
    }
  }

  /// Modo strict/test -> fallar inmediatamente si algo esta mal
  pub fn strict_mode() -> Self {
    Self {
      show_ast: false,
      show_semantic_result: false,
      stop_after_parse_errors: true,
      stop_after_semantic_errors: true,
      show_stage_timings: false,
    }
  }

  /// Modo IDE -> usar AST aunque haya errores, mejorar experiencia del usuario
  pub fn ide_mode() -> Self {
    Self {
      show_ast: true,
      show_semantic_result: true,
      stop_after_parse_errors: false,
      stop_after_semantic_errors: false,
      show_stage_timings: false,
    }
  }

  pub fn with_stage_timings(mut self, enabled: bool) -> Self {
    self.show_stage_timings = enabled;
    self
  }
}
