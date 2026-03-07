use std::env;

#[derive(Debug, Clone)]
pub struct CliOptions {
  pub filename: String,
  pub show_stage_timings: bool,
}

impl CliOptions {
  fn correct_use() -> &'static str {
    "Uso: cargo run -- <archivo.lolo> [--timings]"
  }

  pub fn parse() -> Result<Self, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
      return Err(Self::correct_use().into());
    }

    let mut filename: Option<String> = None;
    let mut show_stage_timings = false;

    for arg in args {
      match arg.as_str() {
        "--timings" => show_stage_timings = true,
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
    }

    let filename = filename.ok_or_else(|| "Falta el archivo de entrada.".to_string())?;

    Ok(Self {
      filename,
      show_stage_timings,
    })
  }
}
