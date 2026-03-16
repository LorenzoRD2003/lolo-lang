use std::fmt;
use std::fs;

use lolo_lang::CliOptions;
use lolo_lang::{Frontend, FrontendConfig, FrontendResult, Renderer};

fn compile(
  source_code: &str,
  show_stage_timings: bool,
  show_ir: bool,
  show_pass_stats: bool,
  passes_spec: Option<&str>,
) -> Result<FrontendResult, String> {
  let mut config = FrontendConfig::cli_mode()
    .with_stage_timings(show_stage_timings)
    .with_ir_dump(show_ir)
    .with_pass_stats(show_pass_stats);

  if let Some(passes_spec) = passes_spec {
    config = config.with_passes_spec(passes_spec)?;
  }

  let frontend = Frontend::new(config);
  Ok(frontend.compile(source_code))
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
    show_ir,
    show_pass_stats,
    passes_spec,
  } = match CliOptions::parse() {
    Ok(v) => v,
    Err(e) => {
      eprintln!("{e}");
      std::process::exit(1);
    }
  };

  let path = format!("files-lang/{}", filename);
  let source_code = fs::read_to_string(path)?;

  let result = match compile(
    &source_code,
    show_stage_timings,
    show_ir,
    show_pass_stats,
    passes_spec.as_deref(),
  ) {
    Ok(result) => result,
    Err(err) => {
      eprintln!("Error en --passes: {err}");
      std::process::exit(1);
    }
  };
  if show_ir && let Some(ir_pretty) = result.ir_pretty() {
    println!("--- IR (debug) ---");
    println!("{ir_pretty}");
  }
  if show_pass_stats && let Some(pass_stats) = result.pass_stats_pretty() {
    println!("--- Pass Stats ---");
    print!("{pass_stats}");
  }

  let out = render_diagnostics(&source_code, &filename, &result)?;
  if out.is_empty() {
    println!("No se encontraron diagnosticos sobre el programa. O sea, todo piola!");
  } else {
    println!("{out}");
  }
  Ok(())
}
