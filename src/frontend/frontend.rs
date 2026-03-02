// El frontend debe:
// Recibir codigo fuente (&str), y ejecutar el Lexer, Parser, SemanticAnalyzer
// Devolver AST, SemanticResult, Diagnostics acumulados
// Definir politica de errores
// Debe ser la unica puerta de entrada publica del compilador.
// Orquesta lexer, parser, semantic analyzer

use crate::frontend::{FrontendConfig, FrontendResult, pipeline::FrontendPipeline};

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
    let pipeline = FrontendPipeline::default();
    pipeline.run(source, &self.config)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pipeline_successful_program() {
    let config = FrontendConfig::cli_mode();
    let frontend = Frontend::new(config);
    let source = r#"
      main {
        let x = 5;
        x = 10;
      }
    "#;
    let result = frontend.compile(source);
    assert!(result.ast().is_some());
    assert!(result.semantic().is_some());
    assert!(result.into_diagnostics().is_empty());
  }

  #[test]
  fn stops_after_parse_errors_in_strict_mode() {
    let mut config = FrontendConfig::strict_mode();
    config.stop_after_parse_errors = true;
    let frontend = Frontend::new(config);
    let source = r#"
      main {
        let x = ;
      }
    "#;
    let result = frontend.compile(source);
    assert!(result.ast().is_some());
    assert!(result.semantic().is_none());
    assert!(!result.into_diagnostics().is_empty());
  }

  #[test]
  fn continues_after_parse_errors_in_tolerant_mode() {
    let config = FrontendConfig::cli_mode();
    let frontend = Frontend::new(config);
    let source = r#"
      main {
        let x = ;
      }
    "#;
    let result = frontend.compile(source);
    assert!(result.ast().is_some());
    assert!(result.semantic().is_none());
    assert!(!result.into_diagnostics().is_empty());
  }
}

#[test]
fn stops_after_semantic_errors_in_strict_mode() {
  let config = FrontendConfig::strict_mode();
  let frontend = Frontend::new(config);
  let source = r#"
    main {
      let x = true;
      x = 5;
    }
  "#;
  let result = frontend.compile(source);
  assert!(result.ast().is_some());
  assert!(result.semantic().is_some());
  assert!(!result.into_diagnostics().is_empty());
}

#[test]
fn continues_after_semantic_errors_in_tolerant_mode() {
  let config = FrontendConfig::cli_mode();
  let frontend = Frontend::new(config);
  let source = r#"
    main {
      let x = true;
      x = 5;
    }
  "#;

  let result = frontend.compile(source);
  assert!(result.ast().is_some());
  assert!(result.semantic().is_some());
  assert!(!result.into_diagnostics().is_empty());
}

#[test]
fn pipeline_is_stateless_between_runs() {
  let config = FrontendConfig::cli_mode();
  let frontend = Frontend::new(config);
  let source = "main { let x = 5; if x == 5 { let y = true } else { let z = 3 + x } }";
  let r1 = frontend.compile(source);
  let r2 = frontend.compile(source);
  assert_eq!(r1, r2);
}

#[test]
fn test_ide_mode() {
  let config = FrontendConfig::ide_mode();
  let frontend = Frontend::new(config);
  let source = r#"
    main {
      let x = 5;
      x = 10;
    }
  "#;
  let result = frontend.compile(source);
  assert!(result.ast().is_some());
  assert!(result.semantic().is_some());
  assert!(result.into_diagnostics().is_empty());
}
