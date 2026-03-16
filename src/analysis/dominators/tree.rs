use crate::{analysis::dominators::idom::Idom, ir::BlockId};

/// Arbol de dominadores construido desde `idom`.
#[derive(Debug, Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct DominatorTree {
  entry: BlockId,
  idom: Idom,
  children: Vec<Vec<BlockId>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl DominatorTree {
  pub(crate) fn from_idom(entry: BlockId, idom: Idom) -> Self {
    let mut children = vec![Vec::new(); idom.len()];

    idom
      .iter()
      .copied()
      .enumerate()
      .filter_map(|(child_idx, parent_opt)| parent_opt.map(|parent| (BlockId(child_idx), parent)))
      // entry no cuenta como hijo de si mismo en el arbol
      .filter(|(child, parent)| *child != entry && *child != *parent)
      .for_each(|(child, parent)| children[parent.0].push(child));

    Self {
      entry,
      idom,
      children,
    }
  }

  pub(crate) fn entry(&self) -> BlockId {
    self.entry
  }

  /// Dominador inmediato de un bloque.
  pub(crate) fn idom(&self, block: BlockId) -> Option<BlockId> {
    self.idom[block.0]
  }

  /// Hijos directos en el arbol de dominadores.
  pub(crate) fn children(&self, block: BlockId) -> &[BlockId] {
    &self.children[block.0]
  }

  /// `true` si `a` domina a `b`.
  pub(crate) fn dominates(&self, a: BlockId, b: BlockId) -> bool {
    if a == b {
      return true;
    }
    let mut current = b;
    while let Some(parent) = self.idom(current) {
      if parent == a {
        return true;
      }
      // Evita loop infinito ante datos corruptos, i.e. entry -> entry.
      if parent == current {
        break;
      }
      current = parent;
    }
    false
  }

  /// `true` si `a` domina a `b` y ademas son diferentes.
  pub(crate) fn strictly_dominates(&self, a: BlockId, b: BlockId) -> bool {
    a != b && self.dominates(a, b)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_idom_builds_children_and_skips_entry_self_edge() {
    // 0(entry), 1<-0, 2<-1, 3<-1, 4 unreachable
    let idom: Idom = vec![
      Some(BlockId(0)),
      Some(BlockId(0)),
      Some(BlockId(1)),
      Some(BlockId(1)),
      None,
    ];
    let tree = DominatorTree::from_idom(BlockId(0), idom);

    assert_eq!(tree.entry(), BlockId(0));
    assert_eq!(tree.children(BlockId(0)), &[BlockId(1)]);
    assert_eq!(tree.children(BlockId(1)), &[BlockId(2), BlockId(3)]);
    assert!(tree.children(BlockId(2)).is_empty());
    assert!(tree.children(BlockId(3)).is_empty());
    assert!(tree.children(BlockId(4)).is_empty());
  }

  #[test]
  fn idom_accessor_returns_expected_values() {
    let idom: Idom = vec![Some(BlockId(0)), Some(BlockId(0)), Some(BlockId(1)), None];
    let tree = DominatorTree::from_idom(BlockId(0), idom);

    assert_eq!(tree.idom(BlockId(0)), Some(BlockId(0)));
    assert_eq!(tree.idom(BlockId(1)), Some(BlockId(0)));
    assert_eq!(tree.idom(BlockId(2)), Some(BlockId(1)));
    assert_eq!(tree.idom(BlockId(3)), None);
  }

  #[test]
  fn dominates_true_for_self_and_ancestors() {
    let idom: Idom = vec![
      Some(BlockId(0)),
      Some(BlockId(0)),
      Some(BlockId(1)),
      Some(BlockId(1)),
    ];
    let tree = DominatorTree::from_idom(BlockId(0), idom);

    assert!(tree.dominates(BlockId(2), BlockId(2))); // self-dominance
    assert!(tree.dominates(BlockId(1), BlockId(2)));
    assert!(tree.dominates(BlockId(0), BlockId(3)));
  }

  #[test]
  fn dominates_false_for_siblings_and_unreachable() {
    let idom: Idom = vec![
      Some(BlockId(0)),
      Some(BlockId(0)),
      Some(BlockId(1)),
      Some(BlockId(1)),
      None,
    ];
    let tree = DominatorTree::from_idom(BlockId(0), idom);

    assert!(!tree.dominates(BlockId(2), BlockId(3))); // siblings
    assert!(!tree.dominates(BlockId(1), BlockId(4))); // unreachable target (None chain)
    assert!(!tree.dominates(BlockId(4), BlockId(1))); // unreachable source cannot dominate reachable
  }

  #[test]
  fn strictly_dominates_requires_distinct_nodes() {
    let idom: Idom = vec![Some(BlockId(0)), Some(BlockId(0)), Some(BlockId(1))];
    let tree = DominatorTree::from_idom(BlockId(0), idom);

    assert!(!tree.strictly_dominates(BlockId(1), BlockId(1)));
    assert!(tree.strictly_dominates(BlockId(1), BlockId(2)));
    assert!(!tree.strictly_dominates(BlockId(2), BlockId(1)));
  }
}
