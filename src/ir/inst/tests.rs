use super::*;
use crate::ast::{BinaryOp, UnaryOp};

fn v(id: usize) -> ValueId {
  ValueId(id)
}

fn b(id: usize) -> BlockId {
  BlockId(id)
}

#[test]
fn with_result_sets_result_and_kind() {
  let inst = InstData::with_result(v(7), InstKind::Const(IrConstant::Bool(true)));
  assert_eq!(inst.result, Some(v(7)));
  assert!(matches!(inst.kind, InstKind::Const(IrConstant::Bool(true))));
}

#[test]
#[should_panic(expected = "la instruccion IR no debe tener resultado")]
fn with_result_panics_for_non_value_instruction() {
  let _ = InstData::with_result(v(0), InstKind::Print(v(1)));
}

#[test]
fn without_result_sets_none_result_and_kind() {
  let inst = InstData::without_result(InstKind::Jump { target: b(2) });
  assert_eq!(inst.result, None);
  assert!(matches!(inst.kind, InstKind::Jump { target } if target == b(2)));
}

#[test]
#[should_panic(expected = "la instruccion IR debe tener resultado")]
fn without_result_panics_for_value_instruction() {
  let _ = InstData::without_result(InstKind::Const(IrConstant::Int32(1)));
}

#[test]
fn inst_data_display_with_result_infers_type_from_kind() {
  let inst = InstData::with_result(
    v(3),
    InstKind::Unary {
      op: UnaryOp::Not,
      operand: v(1),
    },
  );
  assert_eq!(format!("{inst}"), "%v3: Bool = ! %v1");
}

#[test]
fn inst_data_display_without_result_prints_only_kind() {
  let inst = InstData::without_result(InstKind::Return { value: None });
  assert_eq!(format!("{inst}"), "return");
}

#[test]
fn produces_value_classifies_each_kind() {
  assert!(InstKind::Const(IrConstant::Unit).produces_value());
  assert!(InstKind::Copy(v(0)).produces_value());
  assert!(
    InstKind::Unary {
      op: UnaryOp::Neg,
      operand: v(1),
    }
    .produces_value()
  );
  assert!(
    InstKind::Binary {
      op: BinaryOp::Add,
      lhs: v(1),
      rhs: v(2),
    }
    .produces_value()
  );
  assert!(InstKind::Phi { inputs: vec![] }.produces_value());

  assert!(!InstKind::Jump { target: b(0) }.produces_value());
  assert!(
    !InstKind::Branch {
      condition: v(0),
      if_block: b(1),
      else_block: b(2),
    }
    .produces_value()
  );
  assert!(!InstKind::Return { value: Some(v(0)) }.produces_value());
  assert!(!InstKind::Print(v(0)).produces_value());
}

#[test]
fn is_terminator_classifies_each_kind() {
  assert!(InstKind::Jump { target: b(0) }.is_terminator());
  assert!(
    InstKind::Branch {
      condition: v(0),
      if_block: b(1),
      else_block: b(2),
    }
    .is_terminator()
  );
  assert!(InstKind::Return { value: None }.is_terminator());

  assert!(!InstKind::Const(IrConstant::Unit).is_terminator());
  assert!(!InstKind::Copy(v(0)).is_terminator());
  assert!(
    !InstKind::Unary {
      op: UnaryOp::Not,
      operand: v(1),
    }
    .is_terminator()
  );
  assert!(
    !InstKind::Binary {
      op: BinaryOp::Eq,
      lhs: v(1),
      rhs: v(2),
    }
    .is_terminator()
  );
  assert!(!InstKind::Phi { inputs: vec![] }.is_terminator());
  assert!(!InstKind::Print(v(0)).is_terminator());
}

#[test]
fn produced_value_type_matches_expected() {
  assert_eq!(
    InstKind::Const(IrConstant::Unit).produced_value_type(),
    Some(IrType::Unit)
  );
  assert_eq!(
    InstKind::Const(IrConstant::Int32(10)).produced_value_type(),
    Some(IrType::Int32)
  );
  assert_eq!(
    InstKind::Const(IrConstant::Bool(true)).produced_value_type(),
    Some(IrType::Bool)
  );
  assert_eq!(
    InstKind::Unary {
      op: UnaryOp::Neg,
      operand: v(0),
    }
    .produced_value_type(),
    Some(IrType::Int32)
  );
  assert_eq!(
    InstKind::Binary {
      op: BinaryOp::Eq,
      lhs: v(0),
      rhs: v(1),
    }
    .produced_value_type(),
    Some(IrType::Bool)
  );

  assert_eq!(InstKind::Copy(v(0)).produced_value_type(), None);
  assert_eq!(InstKind::Phi { inputs: vec![] }.produced_value_type(), None);
  assert_eq!(InstKind::Jump { target: b(0) }.produced_value_type(), None);
  assert_eq!(
    InstKind::Branch {
      condition: v(0),
      if_block: b(1),
      else_block: b(2),
    }
    .produced_value_type(),
    None
  );
  assert_eq!(
    InstKind::Return { value: Some(v(0)) }.produced_value_type(),
    None
  );
  assert_eq!(InstKind::Print(v(0)).produced_value_type(), None);
}

#[test]
fn operands_returns_expected_values_for_each_kind() {
  assert_eq!(InstKind::Const(IrConstant::Unit).operands(), vec![]);
  assert_eq!(InstKind::Copy(v(9)).operands(), vec![v(9)]);
  assert_eq!(
    InstKind::Unary {
      op: UnaryOp::Not,
      operand: v(4),
    }
    .operands(),
    vec![v(4)]
  );
  assert_eq!(
    InstKind::Binary {
      op: BinaryOp::Sub,
      lhs: v(1),
      rhs: v(2),
    }
    .operands(),
    vec![v(1), v(2)]
  );
  assert_eq!(InstKind::Jump { target: b(3) }.operands(), vec![]);
  assert_eq!(
    InstKind::Branch {
      condition: v(6),
      if_block: b(1),
      else_block: b(2),
    }
    .operands(),
    vec![v(6)]
  );
  assert_eq!(
    InstKind::Phi {
      inputs: vec![PhiInput::new(b(1), v(7)), PhiInput::new(b(2), v(8))],
    }
    .operands(),
    vec![v(7), v(8)]
  );
  assert_eq!(
    InstKind::Return { value: Some(v(5)) }.operands(),
    vec![v(5)]
  );
  assert_eq!(InstKind::Return { value: None }.operands(), vec![]);
  assert_eq!(InstKind::Print(v(11)).operands(), vec![v(11)]);
}

#[test]
fn for_each_operand_visits_operands_in_order() {
  let kind = InstKind::Binary {
    op: BinaryOp::Mul,
    lhs: v(3),
    rhs: v(4),
  };
  let mut visited = Vec::new();
  kind.for_each_operand(|operand| visited.push(operand));
  assert_eq!(visited, vec![v(3), v(4)]);
}

#[test]
fn inst_kind_display_formats_every_variant() {
  assert_eq!(
    format!("{}", InstKind::Const(IrConstant::Int32(5))),
    "const 5"
  );
  assert_eq!(format!("{}", InstKind::Copy(v(1))), "copy %v1");
  assert_eq!(
    format!(
      "{}",
      InstKind::Unary {
        op: UnaryOp::Not,
        operand: v(2),
      }
    ),
    "! %v2"
  );
  assert_eq!(
    format!(
      "{}",
      InstKind::Binary {
        op: BinaryOp::Add,
        lhs: v(1),
        rhs: v(2),
      }
    ),
    "%v1 + %v2"
  );
  assert_eq!(
    format!(
      "{}",
      InstKind::Phi {
        inputs: vec![PhiInput::new(b(1), v(3)), PhiInput::new(b(2), v(4))],
      }
    ),
    "phi[bb1 -> %v3][bb2 -> %v4]"
  );
  assert_eq!(format!("{}", InstKind::Jump { target: b(9) }), "jump bb9");
  assert_eq!(
    format!(
      "{}",
      InstKind::Branch {
        condition: v(0),
        if_block: b(1),
        else_block: b(2),
      }
    ),
    "branch %v0, bb1, bb2"
  );
  assert_eq!(format!("{}", InstKind::Print(v(8))), "print %v8");
  assert_eq!(
    format!("{}", InstKind::Return { value: Some(v(7)) }),
    "return %v7"
  );
  assert_eq!(format!("{}", InstKind::Return { value: None }), "return");
}

#[test]
fn phi_input_new_and_accessors_and_display() {
  let phi = PhiInput::new(b(3), v(12));
  assert_eq!(phi.pred_block(), b(3));
  assert_eq!(phi.value(), v(12));
  assert_eq!(format!("{phi}"), "[bb3 -> %v12]");
}
