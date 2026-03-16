use std::env;

#[derive(Debug, Clone)]
pub struct CliOptions {
  pub filename: String,
  pub show_stage_timings: bool,
  pub show_ir: bool,
  pub show_pass_stats: bool,
  pub passes_spec: Option<String>,
}

impl CliOptions {
  fn correct_use() -> &'static str {
    "Uso: cargo run -- <archivo.lolo> [--timings] [--dump-ir] [--pass-stats] [--passes <plan>]"
  }

  pub fn parse() -> Result<Self, String> {
    Self::parse_from(env::args().skip(1))
  }

  fn parse_from(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
    let args: Vec<String> = args.into_iter().collect();

    if args.is_empty() {
      return Err(Self::correct_use().into());
    }

    let mut filename: Option<String> = None;
    let mut show_stage_timings = false;
    let mut show_ir = false;
    let mut show_pass_stats = false;
    let mut passes_spec: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
      let arg = &args[i];
      match arg.as_str() {
        "--timings" => show_stage_timings = true,
        "--dump-ir" => show_ir = true,
        "--pass-stats" => show_pass_stats = true,
        "--passes" => {
          let value = args
            .get(i + 1)
            .ok_or_else(|| "Falta valor para --passes".to_string())?;
          passes_spec = Some(value.clone());
          i += 1;
        }
        s if s.starts_with("--passes=") => {
          let value = s.trim_start_matches("--passes=");
          if value.is_empty() {
            return Err("Falta valor para --passes".into());
          }
          passes_spec = Some(value.to_string());
        }
        s if !s.starts_with("--") && filename.is_none() => {
          filename = Some(s.to_string());
        }
        _ => {
          return Err(format!(
            "Argumento no reconocido: {arg}\n{}",
            Self::correct_use()
          ));
        }
      }

      i += 1;
    }

    let filename = filename.ok_or_else(|| "Falta el archivo de entrada.".to_string())?;

    Ok(Self {
      filename,
      show_stage_timings,
      show_ir,
      show_pass_stats,
      passes_spec,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::CliOptions;

  #[test]
  fn parse_accepts_passes_with_separate_value() {
    let opts = CliOptions::parse_from([
      "prog.lolo".to_string(),
      "--passes".to_string(),
      "uce,dce*2".to_string(),
    ])
    .expect("parse valido");

    assert_eq!(opts.filename, "prog.lolo");
    assert_eq!(opts.passes_spec.as_deref(), Some("uce,dce*2"));
  }

  #[test]
  fn parse_accepts_passes_with_equals_syntax() {
    let opts = CliOptions::parse_from([
      "prog.lolo".to_string(),
      "--passes=uce,dce".to_string(),
      "--timings".to_string(),
    ])
    .expect("parse valido");

    assert_eq!(opts.passes_spec.as_deref(), Some("uce,dce"));
    assert!(opts.show_stage_timings);
  }

  #[test]
  fn parse_rejects_missing_passes_value() {
    let err = CliOptions::parse_from(["prog.lolo".to_string(), "--passes".to_string()])
      .expect_err("debe fallar");
    assert!(err.contains("Falta valor para --passes"));
  }

  #[test]
  fn parse_rejects_unknown_argument() {
    let err = CliOptions::parse_from(["prog.lolo".to_string(), "--x".to_string()])
      .expect_err("debe fallar");
    assert!(err.contains("Argumento no reconocido"));
  }
}
