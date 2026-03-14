use crate::{
  ast::Stmt,
  ir::{
    LoweringCtx, ids::ValueId, inst::InstKind, test_helpers::{lower_source, parse_and_analyze},
    types::IrType, value::IrConstant,
  },
  semantic::SymbolId,
};

#[test]
fn lower_empty_main_emits_unit_and_single_return() {
  let (ir, diagnostics) = lower_source("main {}");
  assert!(diagnostics.is_empty());

  let unit_const_amount =
    ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Const(IrConstant::Unit)));
  assert_eq!(unit_const_amount, 1);
  let return_inst_amount = ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Return { .. }));
  assert_eq!(return_inst_amount, 1);

  let returns = ir.return_values();
  assert_eq!(returns.len(), 1);
  let returned_value = returns[0].expect("return principal debe tener valor");
  assert_eq!(ir.value(returned_value).ty(), &IrType::Unit);
}

#[test]
fn lower_let_assign_and_print_use_latest_ssa_value() {
  let source = r#"
    main {
      let x = 5;
      x = 10;
      print x;
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  let int32_const =
    ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Const(IrConstant::Int32(_))));
  assert_eq!(int32_const, 2);
  let print_amount = ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Print(_)));
  assert_eq!(print_amount, 1);

  let const_10 = ir.const_results(IrConstant::Int32(10));
  assert_eq!(const_10.len(), 1);
  let prints = ir.print_operands();
  assert_eq!(prints, const_10);
}

#[test]
fn lower_unary_and_binary_emit_expected_instruction_kinds() {
  let source = r#"
    main {
      let x = -5;
      let y = 1 + 2;
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  let unary_insts = ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Unary { .. }));
  assert_eq!(unary_insts, 1);
  let binary_insts = ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Binary { .. }));
  assert_eq!(binary_insts, 1);

  for inst_id in ir.reachable_inst_ids() {
    match ir.inst(inst_id).kind {
      InstKind::Unary { .. } | InstKind::Binary { .. } => {
        let result = ir
          .inst(inst_id)
          .result
          .expect("unary/binary deben producir valor");
        assert_eq!(ir.value(result).ty(), &IrType::Int32);
      }
      _ => {}
    }
  }
}

#[test]
fn lower_if_else_expression_emits_result_phi() {
  let source = r#"
    main {
      let y = if true {
        return 1;
      } else {
        return 2;
      };
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  let phis = ir.phi_results_with_types();
  assert_eq!(phis.len(), 1);
  assert_eq!(phis[0].1, IrType::Int32);
  let branch_insts = ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Branch { .. }));
  assert_eq!(branch_insts, 1);
  let jump_insts = ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Jump { .. }));
  assert_eq!(jump_insts, 2);
}

#[test]
fn lower_if_without_else_does_not_emit_result_phi() {
  let source = r#"
    main {
      let y = if true {
        return 1;
      };
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  let phis = ir.phi_results_with_types();
  assert_eq!(phis.len(), 0);
  let unit_consts =
    ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Const(IrConstant::Unit)));
  assert_eq!(unit_consts, 2);
}

#[test]
fn lower_if_statement_merges_symbol_and_print_uses_phi_value() {
  let source = r#"
    main {
      let x = 0;
      if true {
        x = 1;
      } else {
        x = 2;
      };
      print x;
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  let phis = ir.phi_results_with_types();
  assert_eq!(phis.len(), 2);

  let int32_phi_results: Vec<ValueId> = phis
    .iter()
    .filter_map(|(value_id, ty)| {
      if *ty == IrType::Int32 {
        Some(*value_id)
      } else {
        None
      }
    })
    .collect();
  let unit_phi_results: Vec<ValueId> = phis
    .iter()
    .filter_map(|(value_id, ty)| {
      if *ty == IrType::Unit {
        Some(*value_id)
      } else {
        None
      }
    })
    .collect();

  assert_eq!(int32_phi_results.len(), 1);
  assert_eq!(unit_phi_results.len(), 1);

  let prints = ir.print_operands();
  assert_eq!(prints.len(), 1);
  assert_eq!(prints[0], int32_phi_results[0]);
}

#[test]
fn lower_nested_block_return_does_not_emit_extra_function_return() {
  let source = r#"
    main {
      let x = { return 5; };
      print x;
    }
  "#;
  let (ir, diagnostics) = lower_source(source);
  assert!(diagnostics.is_empty());

  assert_eq!(
    ir.count_insts_by_kind(|kind| matches!(kind, InstKind::Return { .. })),
    1
  );
}

#[test]
fn lower_emits_missing_ssa_value_for_symbol_diagnostic() {
  let source = r#"
    main {
      let x = 1;
      x;
    }
  "#;
  let (ast, program, mut semantic, mut diagnostics) = parse_and_analyze(source);
  assert!(diagnostics.is_empty());

  let block = ast.block(program.main_block(&ast));
  let usage_expr_id = match ast.stmt(block.stmts()[1]) {
    Stmt::Expr(expr_id) => *expr_id,
    _ => panic!("se esperaba una expresion de uso de variable"),
  };
  semantic
    .resolution_info
    .insert_expr_symbol(usage_expr_id, SymbolId(999_999));

  let _ir = LoweringCtx::lower_to_ir(&program, &ast, &semantic, &mut diagnostics);
  assert!(
    diagnostics
      .iter()
      .any(|d| d.msg().contains("no se pudo bajar a IR: el simbolo")),
  );
}

#[test]
fn lower_emits_cannot_lower_error_typed_expr_diagnostic_for_if_expression() {
  let source = r#"
    main {
      let x = if true {
        return 1;
      } else {
        return false;
      };
    }
  "#;
  let (_ir, diagnostics) = lower_source(source);
  assert!(
    diagnostics
      .iter()
      .any(|d| d.msg().contains("no se pudo bajar a IR: la expresion")),
  );
}

#[test]
fn lowering_should_not_panic_on_unresolved_variable_expression() {
  let source = r#"
    main {
      x;
    }
  "#;
  let (ast, program, semantic, mut diagnostics) = parse_and_analyze(source);
  assert!(
    diagnostics
      .iter()
      .any(|d| d.msg().contains("variable 'x' indefinida")),
  );

  let lowering_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    let _ = LoweringCtx::lower_to_ir(&program, &ast, &semantic, &mut diagnostics);
  }));
  assert!(
    lowering_result.is_ok(),
    "BUG: lowering paniquea en Expr::Var sin simbolo resuelto; deberia emitir MissingResolvedSymbol"
  );
}

#[test]
fn lowering_should_not_panic_on_unresolved_assignment_lvalue() {
  let source = r#"
    main {
      x = 1;
    }
  "#;
  let (ast, program, semantic, mut diagnostics) = parse_and_analyze(source);
  assert!(
    diagnostics
      .iter()
      .any(|d| d.msg().contains("variable 'x' indefinida")),
  );

  let lowering_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    let _ = LoweringCtx::lower_to_ir(&program, &ast, &semantic, &mut diagnostics);
  }));

  assert!(
    lowering_result.is_ok(),
    "BUG: lowering paniquea en lvalue sin simbolo resuelto; deberia emitir MissingResolvedSymbol"
  );
}
