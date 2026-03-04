use lolo_lang::{Frontend, FrontendConfig};

fn main() {
  let config = FrontendConfig::cli_mode();
  let frontend = Frontend::new(config);
  let source = r#"
      main {
        let x = 5;
        x = 10;
      }
    "#;
  let result = frontend.compile(source);
  if result.has_diagnostics() {
    for diag in result.diagnostics() {
      println!("{}", diag.msg());
    }
  }
}
