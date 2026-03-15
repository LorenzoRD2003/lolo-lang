use crate::{
  analysis::dominators::idom::compute_idom,
  ir::{
    BlockId,
    test_helpers::{
      add_bool_const, base_test_module_with_blocks, build_test_cfg, set_branch_terminator,
      set_jump_terminator, set_return_terminator,
    },
  },
};

#[test]
fn idom_linear_cfg() {
  // 0 -> 1 -> 2
  let mut module = base_test_module_with_blocks(3);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  set_jump_terminator(&mut module, BlockId(1), BlockId(2));
  set_return_terminator(&mut module, BlockId(2));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");
  let idom = compute_idom(&cfg);

  assert_eq!(idom[0], Some(BlockId(0)));
  assert_eq!(idom[1], Some(BlockId(0)));
  assert_eq!(idom[2], Some(BlockId(1)));
}

#[test]
fn idom_diamond_cfg() {
  // 0 -> 1,2 ; 1 -> 3 ; 2 -> 3
  let mut module = base_test_module_with_blocks(4);
  let cond = add_bool_const(&mut module, BlockId(0), true);
  set_branch_terminator(&mut module, BlockId(0), cond, BlockId(1), BlockId(2));
  set_jump_terminator(&mut module, BlockId(1), BlockId(3));
  set_jump_terminator(&mut module, BlockId(2), BlockId(3));
  set_return_terminator(&mut module, BlockId(3));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");
  let idom = compute_idom(&cfg);

  assert_eq!(idom[0], Some(BlockId(0)));
  assert_eq!(idom[1], Some(BlockId(0)));
  assert_eq!(idom[2], Some(BlockId(0)));
  assert_eq!(idom[3], Some(BlockId(0)));
}

#[test]
fn idom_ignores_unreachable_predecessor() {
  // 0 -> 1 ; 2 -> 1, pero 2 es inalcanzable desde entry=0
  let mut module = base_test_module_with_blocks(3);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  set_return_terminator(&mut module, BlockId(1));
  set_jump_terminator(&mut module, BlockId(2), BlockId(1));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");
  let idom = compute_idom(&cfg);

  assert_eq!(idom[0], Some(BlockId(0)));
  assert_eq!(idom[1], Some(BlockId(0)));
  assert_eq!(idom[2], None);
}

#[test]
fn idom_unreachable_block_is_none() {
  // 0 -> 1 ; 2 unreachable
  let mut module = base_test_module_with_blocks(3);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  set_return_terminator(&mut module, BlockId(1));
  set_return_terminator(&mut module, BlockId(2));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");
  let idom = compute_idom(&cfg);

  assert_eq!(idom[0], Some(BlockId(0)));
  assert_eq!(idom[1], Some(BlockId(0)));
  assert_eq!(idom[2], None);
}

#[test]
fn idom_loop_header_and_body() {
  // 0 -> 1 -> 2 -> 1 (loop), 1 -> 3
  let mut module = base_test_module_with_blocks(4);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));

  let cond = add_bool_const(&mut module, BlockId(1), true);
  set_branch_terminator(&mut module, BlockId(1), cond, BlockId(2), BlockId(3));

  set_jump_terminator(&mut module, BlockId(2), BlockId(1));
  set_return_terminator(&mut module, BlockId(3));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");
  let idom = compute_idom(&cfg);

  assert_eq!(idom[0], Some(BlockId(0)));
  assert_eq!(idom[1], Some(BlockId(0)));
  assert_eq!(idom[2], Some(BlockId(1)));
  assert_eq!(idom[3], Some(BlockId(1)));
}
