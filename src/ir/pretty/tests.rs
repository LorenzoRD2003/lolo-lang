use crate::ir::test_helpers::lower_source;
use crate::ir::{block::BlockData, module::IrModule, types::IrType};

#[test]
fn pretty_renders_module_header_and_entry_block() {
  let (ir, diagnostics) = lower_source("main {}");
  assert!(diagnostics.is_empty());

  let pretty = ir.pretty();

  assert!(pretty.starts_with("module main -> () entry bb0\n"));
  assert!(pretty.contains("bb0:\n"));
  assert!(pretty.contains("const"));
  assert!(pretty.contains("return %v"));
}

#[test]
fn pretty_renders_control_flow_and_phi_nodes() {
  let source = r#"
    main {
      let c = true;
      let x = 0;
      if c {
        x = 1;
      } else {
        x = 2;
      };
      print x;
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  let pretty = ir.pretty();

  assert!(pretty.contains("bb0:\n"));
  assert!(pretty.contains("branch %v"));
  assert!(pretty.contains("jump bb"));
  assert!(pretty.contains("phi["));
  assert!(pretty.contains("print %v"));
}

#[test]
fn pretty_marks_block_without_terminator() {
  let mut module = IrModule::new("m".into(), IrType::Unit);
  module.add_block(BlockData::new_block());
  module.set_entry_block(crate::ir::ids::BlockId(0));

  let pretty = module.pretty();

  assert!(pretty.contains("bb0:\n"));
  assert!(pretty.contains("  <missing terminator>\n"));
}
