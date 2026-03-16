use crate::{
  analysis::Cfg,
  ir::{InstKind, IrConstant, IrModule, test_helpers::lower_source},
  passes::dce::DcePass,
};

fn run_dce(source: &str) -> (IrModule, crate::passes::dce::DceStats) {
  let (mut ir, diagnostics) = lower_source(source);
  assert!(
    diagnostics.is_empty(),
    "el programa de test debe bajar a IR sin diagnosticos: {diagnostics:?}"
  );
  let mut cfg_errors = Vec::new();
  let cfg = Cfg::build(&ir, ir.entry_block(), &mut cfg_errors);
  assert!(cfg_errors.is_empty(), "el CFG de test debe ser valido");
  let stats = DcePass::run(&mut ir, &cfg);
  (ir, stats)
}

#[test]
fn dce_removes_dead_integer_constant() {
  let source = r#"
    main {
      let x = 1;
      let y = 2;
      print x;
    }
  "#;

  let (ir, stats) = run_dce(source);

  assert_eq!(ir.const_results(IrConstant::Int32(2)).len(), 0);
  assert!(
    stats.removed_insts >= 1,
    "DCE deberia remover al menos una instruccion muerta"
  );
}

#[test]
fn dce_keeps_print_and_its_operand_definition() {
  let source = r#"
    main {
      let x = 42;
      print x;
    }
  "#;

  let (ir, _stats) = run_dce(source);

  assert_eq!(
    ir.count_insts_by_kind(|k| matches!(k, InstKind::Print(_))),
    1
  );
  assert_eq!(ir.const_results(IrConstant::Int32(42)).len(), 1);
}

#[test]
fn dce_removes_dead_phis_when_merge_value_is_not_used() {
  let source = r#"
    main {
      let c = true;
      let x = 0;
      if c {
        x = 1;
      } else {
        x = 2;
      };
    }
  "#;

  let (ir_before, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());
  assert!(
    ir_before.count_insts_by_kind(|k| matches!(k, InstKind::Phi { .. })) >= 1,
    "el lowering deberia producir phi(s) en este caso"
  );

  let mut ir_after = ir_before.clone();
  let mut cfg_errors = Vec::new();
  let cfg = Cfg::build(&ir_after, ir_after.entry_block(), &mut cfg_errors);
  assert!(cfg_errors.is_empty(), "el CFG de test debe ser valido");
  let stats = DcePass::run(&mut ir_after, &cfg);

  assert_eq!(
    ir_after.count_insts_by_kind(|k| matches!(k, InstKind::Phi { .. })),
    0
  );
  assert!(
    stats.removed_phis >= 1,
    "DCE deberia remover al menos un phi muerto"
  );
}

#[test]
fn dce_keeps_live_phi_when_value_flows_to_print() {
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

  let (ir, _stats) = run_dce(source);

  assert_eq!(
    ir.count_insts_by_kind(|k| matches!(k, InstKind::Print(_))),
    1
  );
  assert!(
    ir.count_insts_by_kind(|k| matches!(k, InstKind::Phi { .. })) >= 1,
    "debe sobrevivir al menos el phi que alimenta al print"
  );
}

#[test]
fn dce_handles_repeated_mark_live_for_same_definition() {
  // `c` alimenta tanto un `branch` (terminador raiz) como un `print` (side-effect raiz).
  // Eso hace que la misma definicion se intente marcar viva mas de una vez.
  let source = r#"
    main {
      let c = true;
      if c {
        let x = 1;
      } else {
        let y = 2;
      };
      print c;
    }
  "#;

  let (ir, _stats) = run_dce(source);

  assert_eq!(
    ir.count_insts_by_kind(|k| matches!(k, InstKind::Branch { .. })),
    1
  );
  assert_eq!(
    ir.count_insts_by_kind(|k| matches!(k, InstKind::Print(_))),
    1
  );
  assert_eq!(ir.const_results(IrConstant::Bool(true)).len(), 1);
}
