use std::fmt;
use std::fs;

use lolo_lang::CliOptions;
use lolo_lang::{Frontend, FrontendConfig, FrontendResult, Renderer};

fn compile(source_code: &str, show_stage_timings: bool) -> FrontendResult {
  let config = FrontendConfig::cli_mode().with_stage_timings(show_stage_timings);
  let frontend = Frontend::new(config);
  frontend.compile(source_code)
}

fn render_diagnostics(
  source_code: &str,
  filename: &str,
  result: &FrontendResult,
) -> Result<String, fmt::Error> {
  let mut out = String::new();
  let mut renderer = Renderer::new(source_code, filename, &mut out);
  if result.has_diagnostics() {
    for diag in result.diagnostics() {
      renderer.render(diag)?;
    }
  }
  Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let CliOptions {
    filename,
    show_stage_timings,
  } = match CliOptions::parse() {
    Ok(v) => v,
    Err(e) => {
      eprintln!("{e}");
      std::process::exit(1);
    }
  };

  let path = format!("files-lang/{}", filename);
  let source_code = fs::read_to_string(path)?;

  let result = compile(&source_code, show_stage_timings);
  let out = render_diagnostics(&source_code, &filename, &result)?;
  if out.is_empty() {
    println!("No se encontraron diagnosticos sobre el programa. O sea, todo piola!");
  } else {
    println!("{out}");
  }
  Ok(())
}
