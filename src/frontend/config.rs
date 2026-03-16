use crate::passes::PassPlan;

// Responsabilidad: controlar la politica del frontend

#[derive(Debug, Clone)]
pub struct FrontendConfig {
  /// Seria el --dump-ast.
  #[allow(dead_code)]
  pub(crate) show_ast: bool,
  /// Seria el --dump-semantic.
  #[allow(dead_code)]
  pub(crate) show_semantic_result: bool,
  /// Seria el --dump-ir.
  pub(crate) show_ir: bool,
  /// Seria el --pass-stats.
  pub(crate) show_pass_stats: bool,
  /// Deja de compilar si encuentra errores en la fase de Parsing.
  pub(crate) stop_after_parse_errors: bool,
  /// Deja de compilar si encuentra errores en la fase de analisis semantico.
  pub(crate) stop_after_semantic_errors: bool,
  /// Indica cuanto tiempo tardo cada stage.
  pub(crate) show_stage_timings: bool,
  /// Plan de passes de optimizacion para IR (orden + repeticiones).
  pub(crate) pass_plan: PassPlan,
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
      show_ir: false,
      show_pass_stats: false,
      pass_plan: PassPlan::default(),
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
      show_ir: false,
      show_pass_stats: false,
      pass_plan: PassPlan::default(),
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
      show_ir: false,
      show_pass_stats: false,
      pass_plan: PassPlan::default(),
    }
  }

  pub fn with_stage_timings(mut self, enabled: bool) -> Self {
    self.show_stage_timings = enabled;
    self
  }

  pub fn with_ir_dump(mut self, enabled: bool) -> Self {
    self.show_ir = enabled;
    self
  }

  pub fn with_pass_stats(mut self, enabled: bool) -> Self {
    self.show_pass_stats = enabled;
    self
  }

  pub fn with_passes_spec(mut self, spec: &str) -> Result<Self, String> {
    self.pass_plan = PassPlan::parse(spec)?;
    Ok(self)
  }

  pub(crate) fn pass_plan(&self) -> &PassPlan {
    &self.pass_plan
  }
}

#[cfg(test)]
mod tests {
  use super::FrontendConfig;

  #[test]
  fn with_passes_spec_overrides_default_order() {
    let config = FrontendConfig::cli_mode()
      .with_passes_spec("dce*2,uce")
      .expect("spec valida");

    let expanded: Vec<_> = config
      .pass_plan()
      .expanded_passes()
      .map(|id| id.name())
      .collect();

    assert_eq!(expanded, vec!["dce", "dce", "uce"]);
  }

  #[test]
  fn with_passes_spec_rejects_invalid_spec() {
    let err = FrontendConfig::cli_mode()
      .with_passes_spec("foo")
      .expect_err("debe fallar");
    assert!(err.contains("pass desconocida"));
  }
}
