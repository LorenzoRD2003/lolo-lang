use super::Dominators;
use crate::ir::{
  BlockId,
  test_helpers::{
    add_bool_const, base_test_module_with_blocks, build_test_cfg, set_branch_terminator,
    set_jump_terminator, set_return_terminator,
  },
};

#[test]
fn dominators_compute_linear_cfg() {
  // 0 -> 1 -> 2
  let mut module = base_test_module_with_blocks(3);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  set_jump_terminator(&mut module, BlockId(1), BlockId(2));
  set_return_terminator(&mut module, BlockId(2));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");

  let dom = Dominators::compute(&cfg);

  assert_eq!(dom.tree().entry(), BlockId(0));
  assert_eq!(dom.tree().idom(BlockId(0)), Some(BlockId(0)));
  assert_eq!(dom.tree().idom(BlockId(1)), Some(BlockId(0)));
  assert_eq!(dom.tree().idom(BlockId(2)), Some(BlockId(1)));

  assert!(dom.tree().dominates(BlockId(0), BlockId(2)));
  assert!(dom.tree().strictly_dominates(BlockId(1), BlockId(2)));
  assert!(!dom.tree().strictly_dominates(BlockId(2), BlockId(2)));

  assert!(dom.frontier().for_block(BlockId(0)).is_empty());
  assert!(dom.frontier().for_block(BlockId(1)).is_empty());
  assert!(dom.frontier().for_block(BlockId(2)).is_empty());
}

#[test]
fn dominators_compute_diamond_cfg() {
  // 0 -> 1,2 ; 1 -> 3 ; 2 -> 3
  let mut module = base_test_module_with_blocks(4);
  let cond = add_bool_const(&mut module, BlockId(0), true);
  set_branch_terminator(&mut module, BlockId(0), cond, BlockId(1), BlockId(2));
  set_jump_terminator(&mut module, BlockId(1), BlockId(3));
  set_jump_terminator(&mut module, BlockId(2), BlockId(3));
  set_return_terminator(&mut module, BlockId(3));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");

  let dom = Dominators::compute(&cfg);

  assert_eq!(dom.tree().idom(BlockId(0)), Some(BlockId(0)));
  assert_eq!(dom.tree().idom(BlockId(1)), Some(BlockId(0)));
  assert_eq!(dom.tree().idom(BlockId(2)), Some(BlockId(0)));
  assert_eq!(dom.tree().idom(BlockId(3)), Some(BlockId(0)));

  assert!(!dom.tree().dominates(BlockId(1), BlockId(2)));
  assert!(!dom.tree().dominates(BlockId(2), BlockId(1)));

  assert!(dom.frontier().for_block(BlockId(0)).is_empty());
  assert!(dom.frontier().for_block(BlockId(1)).contains(&BlockId(3)));
  assert!(dom.frontier().for_block(BlockId(2)).contains(&BlockId(3)));
}

#[test]
fn dominators_compute_loop_cfg() {
  // 0 -> 1 ; 1 -> 2,3 ; 2 -> 1
  let mut module = base_test_module_with_blocks(4);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  let cond = add_bool_const(&mut module, BlockId(1), true);
  set_branch_terminator(&mut module, BlockId(1), cond, BlockId(2), BlockId(3));
  set_jump_terminator(&mut module, BlockId(2), BlockId(1));
  set_return_terminator(&mut module, BlockId(3));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");

  let dom = Dominators::compute(&cfg);

  assert_eq!(dom.tree().idom(BlockId(1)), Some(BlockId(0)));
  assert_eq!(dom.tree().idom(BlockId(2)), Some(BlockId(1)));
  assert_eq!(dom.tree().idom(BlockId(3)), Some(BlockId(1)));

  assert!(dom.tree().dominates(BlockId(1), BlockId(2)));
  assert!(dom.tree().dominates(BlockId(1), BlockId(3)));

  assert!(dom.frontier().for_block(BlockId(1)).contains(&BlockId(1)));
  assert!(dom.frontier().for_block(BlockId(2)).contains(&BlockId(1)));
}

#[test]
fn dominators_compute_ignores_unreachable_predecessor() {
  // 0 -> 1 ; 2 -> 1 (2 inalcanzable desde entry=0)
  let mut module = base_test_module_with_blocks(3);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  set_return_terminator(&mut module, BlockId(1));
  set_jump_terminator(&mut module, BlockId(2), BlockId(1));

  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty(), "el CFG de test no deberia tener errores");

  let dom = Dominators::compute(&cfg);

  assert_eq!(dom.tree().idom(BlockId(2)), None);
  assert!(!dom.tree().dominates(BlockId(1), BlockId(2)));
  assert!(dom.frontier().for_block(BlockId(1)).is_empty());
}
