use crate::{analysis::cfg::Cfg, ir::BlockId};

use super::DominatorTree;

/// Frontera de dominancia por bloque.
/// Para un bloque `A`, es el conjunto de bloques donde la dominancia
/// de `A` deja de ser estricta. Es decir, son bloques `B` tales que
/// `A` domina al menos un predecesor de `B`, pero `A` no domina a `B`.
/// Esto es util para los puntos de merge donde se usa phi.
#[derive(Debug, Clone)]
pub(crate) struct DominanceFrontier {
  frontier: Vec<Vec<BlockId>>,
}

impl DominanceFrontier {
  /// Calcula la frontera de dominancia a partir de CFG + arbol de dominadores.
  pub(crate) fn compute(cfg: &Cfg, tree: &DominatorTree) -> Self {
    let mut frontier = vec![Vec::new(); cfg.block_count()];

    for block in cfg.blocks() {
      if !cfg.is_reachable(block) {
        continue;
      }

      let preds = cfg.predecessors(block);
      if preds.len() < 2 {
        continue;
      }

      let idom_block = tree.idom(block);
      for &pred in preds {
        if !cfg.is_reachable(pred) {
          continue;
        }

        let mut runner = pred;
        while Some(runner) != idom_block {
          if !frontier[runner.0].contains(&block) {
            frontier[runner.0].push(block);
          }

          let next = tree
            .idom(runner)
            .expect("runner alcanzable debe tener idom en `DominanceFrontier::compute`");
          debug_assert_ne!(
            next, runner,
            "`idom(runner)` no debe formar un loop en `DominanceFrontier::compute`"
          );
          runner = next;
        }
      }
    }

    Self { frontier }
  }

  pub(crate) fn for_block(&self, block: BlockId) -> &[BlockId] {
    &self.frontier[block.0]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    analysis::dominators::{idom::compute_idom, tree::DominatorTree},
    ir::{
      BlockId,
      test_helpers::{
        add_bool_const, base_test_module_with_blocks, build_test_cfg, set_branch_terminator,
        set_jump_terminator, set_return_terminator,
      },
    },
  };

  #[test]
  fn frontier_linear_cfg_is_empty() {
    // 0 -> 1 -> 2
    let mut module = base_test_module_with_blocks(3);
    set_jump_terminator(&mut module, BlockId(0), BlockId(1));
    set_jump_terminator(&mut module, BlockId(1), BlockId(2));
    set_return_terminator(&mut module, BlockId(2));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let idom = compute_idom(&cfg);
    let tree = DominatorTree::from_idom(cfg.entry(), idom);
    let df = DominanceFrontier::compute(&cfg, &tree);

    assert!(df.for_block(BlockId(0)).is_empty());
    assert!(df.for_block(BlockId(1)).is_empty());
    assert!(df.for_block(BlockId(2)).is_empty());
  }

  #[test]
  fn frontier_diamond_cfg_puts_merge_in_both_branches() {
    // 0 -> 1,2 ; 1 -> 3 ; 2 -> 3
    let mut module = base_test_module_with_blocks(4);
    let cond = add_bool_const(&mut module, BlockId(0), true);
    set_branch_terminator(&mut module, BlockId(0), cond, BlockId(1), BlockId(2));
    set_jump_terminator(&mut module, BlockId(1), BlockId(3));
    set_jump_terminator(&mut module, BlockId(2), BlockId(3));
    set_return_terminator(&mut module, BlockId(3));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let idom = compute_idom(&cfg);
    let tree = DominatorTree::from_idom(cfg.entry(), idom);
    let df = DominanceFrontier::compute(&cfg, &tree);

    assert!(df.for_block(BlockId(0)).is_empty());
    assert!(df.for_block(BlockId(1)).contains(&BlockId(3)));
    assert!(df.for_block(BlockId(2)).contains(&BlockId(3)));
    assert!(df.for_block(BlockId(3)).is_empty());
  }

  #[test]
  fn frontier_loop_header_contains_itself() {
    // 0 -> 1 -> 2 -> 1 ; 1 -> 3
    let mut module = base_test_module_with_blocks(4);
    set_jump_terminator(&mut module, BlockId(0), BlockId(1));
    let cond = add_bool_const(&mut module, BlockId(1), true);
    set_branch_terminator(&mut module, BlockId(1), cond, BlockId(2), BlockId(3));
    set_jump_terminator(&mut module, BlockId(2), BlockId(1));
    set_return_terminator(&mut module, BlockId(3));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let idom = compute_idom(&cfg);
    let tree = DominatorTree::from_idom(cfg.entry(), idom);
    let df = DominanceFrontier::compute(&cfg, &tree);

    assert!(df.for_block(BlockId(1)).contains(&BlockId(1)));
    assert!(df.for_block(BlockId(2)).contains(&BlockId(1)));
  }

  #[test]
  fn frontier_ignores_unreachable_blocks_and_predecessors() {
    // 0 -> 1 ; 2 -> 1 (2 inalcanzable desde entry=0)
    let mut module = base_test_module_with_blocks(3);
    set_jump_terminator(&mut module, BlockId(0), BlockId(1));
    set_return_terminator(&mut module, BlockId(1));
    set_jump_terminator(&mut module, BlockId(2), BlockId(1));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let idom = compute_idom(&cfg);
    let tree = DominatorTree::from_idom(cfg.entry(), idom);
    let df = DominanceFrontier::compute(&cfg, &tree);

    assert!(df.for_block(BlockId(0)).is_empty());
    assert!(df.for_block(BlockId(1)).is_empty());
    assert!(df.for_block(BlockId(2)).is_empty());
  }

  #[test]
  fn frontier_deduplicates_via_two_distinct_backedges() {
    // 0 -> 1 ; 1 -> 2,3 ; 2 -> 1 ; 3 -> 1
    // Dos backedges distintos intentan insertar el mismo bloque en DF(1).
    // La frontera debe mantener una sola ocurrencia.
    let mut module = base_test_module_with_blocks(4);
    set_jump_terminator(&mut module, BlockId(0), BlockId(1));

    let cond = add_bool_const(&mut module, BlockId(1), true);
    set_branch_terminator(&mut module, BlockId(1), cond, BlockId(2), BlockId(3));

    set_jump_terminator(&mut module, BlockId(2), BlockId(1));
    set_jump_terminator(&mut module, BlockId(3), BlockId(1));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let idom = compute_idom(&cfg);
    let tree = DominatorTree::from_idom(cfg.entry(), idom);
    let df = DominanceFrontier::compute(&cfg, &tree);

    assert_eq!(df.for_block(BlockId(1)), &[BlockId(1)]);
    assert_eq!(df.for_block(BlockId(2)), &[BlockId(1)]);
    assert_eq!(df.for_block(BlockId(3)), &[BlockId(1)]);
  }
}
