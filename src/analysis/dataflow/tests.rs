use crate::{
  analysis::{
    Cfg,
    dataflow::{
      direction::Direction, lattice::Lattice, problem::DataflowProblem, result::DataflowResult,
      solver::DataflowSolver, worklist::Worklist,
    },
  },
  ir::{
    BlockId, IrModule,
    test_helpers::{
      add_bool_const, base_test_module_with_blocks, build_test_cfg, set_branch_terminator,
      set_jump_terminator, set_return_terminator,
    },
  },
};

#[derive(Debug, Clone, Copy)]
struct TestProblem {
  direction: Direction,
  boundary: i32,
  add_one_on_transfer: bool,
}

impl TestProblem {
  fn add_one_forward() -> Self {
    Self {
      direction: Direction::Forward,
      boundary: 1,
      add_one_on_transfer: true,
    }
  }

  fn add_one_backward() -> Self {
    Self {
      direction: Direction::Backward,
      boundary: 1,
      add_one_on_transfer: true,
    }
  }

  fn stable_forward() -> Self {
    Self {
      direction: Direction::Forward,
      boundary: 0,
      add_one_on_transfer: false,
    }
  }
}

impl Lattice for TestProblem {
  type Fact = i32;

  fn bottom(&self) -> Self::Fact {
    0
  }

  fn top(&self) -> Self::Fact {
    i32::MAX
  }

  fn meet(&self, lhs: &Self::Fact, rhs: &Self::Fact) -> Self::Fact {
    lhs + rhs
  }
}

impl DataflowProblem for TestProblem {
  fn direction(&self) -> Direction {
    self.direction
  }

  fn transfer_block(
    &self,
    _block: BlockId,
    fact: &Self::Fact,
    _module: &IrModule,
    _cfg: &Cfg,
  ) -> Self::Fact {
    if self.add_one_on_transfer {
      *fact + 1
    } else {
      *fact
    }
  }

  fn boundary_fact(&self) -> Self::Fact {
    self.boundary
  }
}

fn linear_module() -> IrModule {
  let mut module = base_test_module_with_blocks(3);
  set_jump_terminator(&mut module, BlockId(0), BlockId(1));
  set_jump_terminator(&mut module, BlockId(1), BlockId(2));
  set_return_terminator(&mut module, BlockId(2));
  module
}

fn diamond_module() -> IrModule {
  let mut module = base_test_module_with_blocks(4);
  let cond = add_bool_const(&mut module, BlockId(0), true);
  set_branch_terminator(&mut module, BlockId(0), cond, BlockId(1), BlockId(2));
  set_jump_terminator(&mut module, BlockId(1), BlockId(3));
  set_jump_terminator(&mut module, BlockId(2), BlockId(3));
  set_return_terminator(&mut module, BlockId(3));
  module
}

#[test]
fn direction_helpers_classify_both_directions() {
  assert!(Direction::Forward.is_forward());
  assert!(!Direction::Forward.is_backward());
  assert!(Direction::Backward.is_backward());
  assert!(!Direction::Backward.is_forward());
}

#[test]
fn lattice_helpers_cover_empty_and_non_empty_meet_all() {
  let problem = TestProblem::add_one_forward();

  assert_eq!(problem.top(), i32::MAX);
  assert_eq!(problem.meet_all(std::iter::empty::<&i32>()), 0);

  let facts = [2, 3, 4];
  assert_eq!(problem.meet_all(facts.iter()), 9);
}

#[test]
fn dataflow_result_helper_methods_work() {
  let mut result = DataflowResult::new(2, 0);
  result.set_in_fact(BlockId(0), 10);
  result.set_out_fact(BlockId(0), 11);
  result.set_in_fact(BlockId(1), 20);
  result.set_out_fact(BlockId(1), 21);

  assert_eq!(result.block_count(), 2);
  assert_eq!(result.facts_for(BlockId(1)), (&20, &21));

  let collected: Vec<(usize, i32, i32)> = result
    .iter()
    .map(|(block, in_fact, out_fact)| (block.0, *in_fact, *out_fact))
    .collect();
  assert_eq!(collected.len(), 2);
  assert!(collected.contains(&(0, 10, 11)));
  assert!(collected.contains(&(1, 20, 21)));

  let (in_vec, out_vec) = result.into_parts();
  assert_eq!(in_vec[0], 10);
  assert_eq!(out_vec[1], 21);
}

#[test]
fn worklist_is_fifo_and_deduplicates_pending_blocks() {
  let mut worklist = Worklist::new(4);
  worklist.push(BlockId(1));
  worklist.push(BlockId(2));
  worklist.push(BlockId(1));

  assert_eq!(worklist.pop(), Some(BlockId(1)));
  assert_eq!(worklist.pop(), Some(BlockId(2)));
  assert_eq!(worklist.pop(), None);
  assert!(worklist.is_empty());
}

#[test]
fn solver_forward_propagates_across_linear_cfg() {
  let module = linear_module();
  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty());

  let result = DataflowSolver::solve(&TestProblem::add_one_forward(), &module, &cfg);

  assert_eq!(*result.in_fact(BlockId(0)), 1);
  assert_eq!(*result.out_fact(BlockId(0)), 2);
  assert_eq!(*result.in_fact(BlockId(1)), 2);
  assert_eq!(*result.out_fact(BlockId(1)), 3);
  assert_eq!(*result.in_fact(BlockId(2)), 3);
  assert_eq!(*result.out_fact(BlockId(2)), 4);
}

#[test]
fn solver_backward_propagates_across_linear_cfg() {
  let module = linear_module();
  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty());

  let result = DataflowSolver::solve(&TestProblem::add_one_backward(), &module, &cfg);

  assert_eq!(*result.out_fact(BlockId(2)), 1);
  assert_eq!(*result.in_fact(BlockId(2)), 2);
  assert_eq!(*result.out_fact(BlockId(1)), 2);
  assert_eq!(*result.in_fact(BlockId(1)), 3);
  assert_eq!(*result.out_fact(BlockId(0)), 3);
  assert_eq!(*result.in_fact(BlockId(0)), 4);
}

#[test]
fn solver_forward_merges_predecessor_information() {
  let module = diamond_module();
  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty());

  let result = DataflowSolver::solve(&TestProblem::add_one_forward(), &module, &cfg);

  assert_eq!(*result.out_fact(BlockId(0)), 2);
  assert_eq!(*result.in_fact(BlockId(1)), 2);
  assert_eq!(*result.out_fact(BlockId(1)), 3);
  assert_eq!(*result.in_fact(BlockId(2)), 2);
  assert_eq!(*result.out_fact(BlockId(2)), 3);
  assert_eq!(*result.in_fact(BlockId(3)), 6);
  assert_eq!(*result.out_fact(BlockId(3)), 7);
}

#[test]
fn solver_backward_merges_successor_information() {
  let module = diamond_module();
  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty());

  let result = DataflowSolver::solve(&TestProblem::add_one_backward(), &module, &cfg);

  assert_eq!(*result.out_fact(BlockId(3)), 1);
  assert_eq!(*result.in_fact(BlockId(3)), 2);
  assert_eq!(*result.out_fact(BlockId(1)), 2);
  assert_eq!(*result.in_fact(BlockId(1)), 3);
  assert_eq!(*result.out_fact(BlockId(2)), 2);
  assert_eq!(*result.in_fact(BlockId(2)), 3);
  assert_eq!(*result.out_fact(BlockId(0)), 6);
  assert_eq!(*result.in_fact(BlockId(0)), 7);
}

#[test]
fn solver_stops_when_problem_is_already_stable() {
  let module = linear_module();
  let (cfg, errors) = build_test_cfg(&module, BlockId(0));
  assert!(errors.is_empty());

  let result = DataflowSolver::solve(&TestProblem::stable_forward(), &module, &cfg);

  assert_eq!(*result.in_fact(BlockId(0)), 0);
  assert_eq!(*result.out_fact(BlockId(0)), 0);
  assert_eq!(*result.in_fact(BlockId(1)), 0);
  assert_eq!(*result.out_fact(BlockId(1)), 0);
  assert_eq!(*result.in_fact(BlockId(2)), 0);
  assert_eq!(*result.out_fact(BlockId(2)), 0);
}
