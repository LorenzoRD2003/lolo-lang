use crate::{
  ast::AstVisitor,
  diagnostics::Diagnostic,
  semantic::{
    mutability_checker::{MutabilityChecker, MutabilityInfo},
    name_resolver::resolve,
  },
};

fn mutability_check(source: &str) -> (MutabilityInfo, Vec<Diagnostic>) {
  let (resolution_info, symbol_table, _, ast, program) = resolve(source);
  let mut checker = MutabilityChecker::new(&ast, &resolution_info, &symbol_table);
  checker.visit_program(&program);
  let diagnostics = checker.diagnostics().to_vec();
  let type_info = checker.into_mutability_info();
  (type_info, diagnostics)
}

#[test]
fn let_binding_is_marked_mutable() {
  let (mutability_info, diagnostics) = mutability_check("main { let x = 5;}");
  assert!(diagnostics.is_empty());
  assert_eq!(mutability_info.len(), 1);
  let (_, mutability) = mutability_info.iter().next().unwrap();
  assert!(mutability.is_mutable());
}

#[test]
fn assign_to_mutable_variable_is_ok() {
  let (_, diagnostics) = mutability_check("main { let x = 5; x = 10; }");
  assert!(diagnostics.is_empty());
}

#[test]
fn const_binding_is_immutable() {
  let (_, diagnostics) = mutability_check("main { const x = 5; x = 10; }");
  assert!(!diagnostics.is_empty());
  assert!(
    diagnostics[0]
      .msg()
      .contains(&"se intento modificar la variable inmutable 'x'".to_string())
  );
}

#[test]
fn multiple_let_bindings_are_all_mutable() {
  let source = r#"
      main {
        let x = 1;
        let y = 2;
        let z = 3;
      }
    "#;
  let (mutability_info, diagnostics) = mutability_check(source);
  assert!(diagnostics.is_empty());
  assert_eq!(mutability_info.len(), 3);
  for mutability in mutability_info.values() {
    assert!(mutability.is_mutable());
  }
}

#[test]
fn assignment_inside_if_is_ok() {
  let source = r#"
      main {
        let x = 5;
        if x {
          x = 10;
        }
      }
    "#;
  let (_, diagnostics) = mutability_check(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn assign_without_let_does_not_crash_checker() {
  // El resolver debería haber producido error antes.
  // El mutability checker no debería crashear.
  // No afirmamos nada fuerte, solo que no panic.
  mutability_check("main { x = 10; }");
}

#[test]
fn assign_to_outer_block_inside_inner_block_is_error() {
  let source = r#"
      main {
        const x = 5;
        {
          x = 2;
        };
      }
    "#;
  let (_, diagnostics) = mutability_check(source);
  assert!(!diagnostics.is_empty());
}

#[test]
fn shadowing_const_with_mutable_inside_block_is_ok() {
  let source = r#"
      main {
        const x = 5;
        {
          let x = 2;
          x = 3;
        };
      }
    "#;
  let (_, diagnostics) = mutability_check(source);
  assert!(diagnostics.is_empty());
}
