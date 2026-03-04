use std::env;
use std::fmt;
use std::fs;

use lolo_lang::{Frontend, FrontendConfig, FrontendResult, Renderer};

fn compile(source_code: &str) -> FrontendResult {
  let config = FrontendConfig::cli_mode();
  let frontend = Frontend::new(config);
  frontend.compile(source_code)
}

fn render_diagnostics(source_code: &str, result: &FrontendResult) -> Result<String, fmt::Error> {
  let mut out = String::new();
  let mut renderer = Renderer::new(source_code, &mut out);
  if result.has_diagnostics() {
    for diag in result.diagnostics() {
      renderer.render(diag)?;
    }
  }
  Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("Uso: cargo run -- <archivo.lolo>");
    std::process::exit(1);
  }

  let filename = &args[1];
  let path = format!("files-lang/{}", filename);
  let source_code = fs::read_to_string(path)?;

  let result = compile(&source_code);
  let out = render_diagnostics(&source_code, &result)?;
  if out.is_empty() {
    println!("No se encontraron diagnosticos sobre el programa. O sea, todo piola!");
  } else {
    println!("{out}");
  }
  Ok(())
}
