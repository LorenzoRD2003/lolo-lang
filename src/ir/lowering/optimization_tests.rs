use crate::ir::test_helpers::lower_source;

#[test]
fn lower_if_with_many_variables_merges_only_modified() {
  let mut source = String::from("main {\n");
  for i in 0..100 {
    source.push_str(&format!("  let v{} = {};\n", i, i));
  }
  source.push_str("  let c = true;\n");
  source.push_str("  if c {\n");
  source.push_str("    v0 = 999;\n");
  source.push_str("  } else {\n");
  source.push_str("    v0 = 888;\n");
  source.push_str("  };\n");
  source.push_str("  print v0;\n");
  source.push_str("  print v99;\n");
  source.push_str("}\n");

  let (ir, diagnostics) = lower_source(&source);
  assert!(diagnostics.is_empty());

  // v0 should have a phi because it's modified in both branches (or just one, but here both)
  // Other 99 variables should NOT have phis because they weren't modified.
  // The if expression itself also produces a unit phi if it's used as a statement but it's an expression.
  // In our case it's a statement `if ... { ... } else { ... };` which is an expression statement.

  // Actually, `lower_if` always emits a phi for the branch result if it has an else branch.
  // and it merges all symbols from `env_before` that were modified.

  // Let's count how many phis we have.
  // 1 for the if expression result (Unit)
  // 1 for v0
  // Total: 2
  let phis = ir.phi_results_with_types();
  assert_eq!(
    phis.len(),
    2,
    "Should only have 2 phis: one for the if result and one for v0. Found: {:?}",
    phis
  );
}
