use crate::{analysis::cfg::Cfg, ir::BlockId};

pub(crate) type Idom = Vec<Option<BlockId>>;

/// Calcula el immediate dominator (`idom`) de cada bloque.
///
/// Un bloque `d` domina a `n` si todo camino desde `entry` hasta `n`
/// pasa por `d`.
///
/// El immediate dominator de `n` es su dominador mas cercano.
/// Convencion: `idom[entry] = Some(entry)`
/// Bloques inalcanzables => `None`
pub(crate) fn compute_idom(cfg: &Cfg) -> Idom {
  // Orden de recorrido: usamos reverse-postorder (RPO) de bloques alcanzables.
  let rpo = reverse_postorder_reachable(cfg);

  // Mapa bloque -> posicion en RPO. Lo usamos en `intersect` para "subir"
  // por la cadena de `idom` comparando alturas relativas.
  let mut rpo_index = vec![usize::MAX; cfg.block_count()];
  for (idx, block) in rpo.iter().copied().enumerate() {
    rpo_index[block.0] = idx;
  }

  // inicializacion: entry se domina a si mismo, y el resto arranca sin idom conocido
  let mut idom = vec![None; cfg.block_count()];
  idom[cfg.entry().0] = Some(cfg.entry());
  let mut changed = true;

  // Iteramos hasta un punto fijo.
  // en cada pasada refinamos `idom[b]` para cada bloque alcanzable `b != entry`.
  while changed {
    changed = false;

    // Entry no se procesa: su idom ya es entry.
    for &bid in rpo.iter().skip(1) {
      // Solo sirven predecesores alcanzables cuyo idom ya fue establecido.
      let mut valid_preds = cfg
        .predecessors(bid)
        .iter()
        .copied()
        .filter(|pred| cfg.is_reachable(*pred) && idom[pred.0].is_some());

      // Invariante: todo bloque alcanzable distinto de `entry` tiene al menos
      // un predecesor alcanzable, y ese predecesor ya posee `idom` en este punto.
      let mut new_idom = valid_preds
        .next()
        .expect("bloque alcanzable sin predecesor util para idom");

      // Unimos el candidato con el resto de predecesores via "intersect":
      // esto da el ancestro comun mas cercano en el arbol parcial de idom.
      for pred in valid_preds {
        new_idom = intersect(pred, new_idom, &idom, &rpo_index);
      }

      // Si mejoramos `idom[b]`, seguimos iterando otra pasada.
      if idom[bid.0] != Some(new_idom) {
        idom[bid.0] = Some(new_idom);
        changed = true;
      }
    }
  }

  idom
}

fn intersect(mut left: BlockId, mut right: BlockId, idom: &Idom, rpo_index: &[usize]) -> BlockId {
  // Recorremos ambos nodos "hacia arriba" por la cadena de immediate dominators
  // hasta que converjan en el mismo bloque.
  while left != right {
    // mientras left esta mas abajo que right: subimos left un paso por su idom.
    while rpo_index[left.0] > rpo_index[right.0] {
      left = idom[left.0].expect("left debe tener idom en intersect");
    }
    // mientras right esta mas abajo que left: subimos right un paso por su idom.
    while rpo_index[right.0] > rpo_index[left.0] {
      right = idom[right.0].expect("right debe tener idom en intersect");
    }
  }

  // Punto de encuentro: ancestro comun usado como nuevo idom candidato.
  left
}

/// Computa el reverse postorder de bloques alcanzables para un cfg.
fn reverse_postorder_reachable(cfg: &Cfg) -> Vec<BlockId> {
  let mut seen = vec![false; cfg.block_count()];
  let mut postorder = Vec::new();
  let entry = cfg.entry();
  cfg_dfs_postorder(cfg, entry, &mut seen, &mut postorder);
  postorder.reverse();
  postorder
}

fn cfg_dfs_postorder(
  cfg: &Cfg,
  start_block: BlockId,
  seen: &mut [bool],
  postorder: &mut Vec<BlockId>,
) {
  if seen[start_block.0] {
    return;
  }
  seen[start_block.0] = true;
  for &succ in cfg.successors(start_block) {
    cfg_dfs_postorder(cfg, succ, seen, postorder);
  }
  postorder.push(start_block);
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;

  use super::reverse_postorder_reachable;
  use crate::ir::{
    BlockId,
    test_helpers::{
      add_bool_const, base_test_module_with_blocks, build_test_cfg, set_branch_terminator,
      set_jump_terminator, set_return_terminator,
    },
  };

  #[test]
  fn reverse_postorder_linear_cfg() {
    // 0 -> 1 -> 2
    let mut module = base_test_module_with_blocks(3);
    set_jump_terminator(&mut module, BlockId(0), BlockId(1));
    set_jump_terminator(&mut module, BlockId(1), BlockId(2));
    set_return_terminator(&mut module, BlockId(2));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let rpo = reverse_postorder_reachable(&cfg);
    assert_eq!(rpo, vec![BlockId(0), BlockId(1), BlockId(2)]);
  }

  #[test]
  fn reverse_postorder_skips_unreachable_blocks() {
    // 0 -> 1, 2 ; 1 -> 3 ; 2 -> 3 ; 4 unreachable
    let mut module = base_test_module_with_blocks(5);
    let cond = add_bool_const(&mut module, BlockId(0), true);
    set_branch_terminator(&mut module, BlockId(0), cond, BlockId(1), BlockId(2));
    set_jump_terminator(&mut module, BlockId(1), BlockId(3));
    set_jump_terminator(&mut module, BlockId(2), BlockId(3));
    set_return_terminator(&mut module, BlockId(3));
    set_return_terminator(&mut module, BlockId(4));

    let (cfg, errors) = build_test_cfg(&module, BlockId(0));
    assert!(errors.is_empty());

    let rpo = reverse_postorder_reachable(&cfg);
    assert_eq!(rpo.first().copied(), Some(BlockId(0)));

    let rpo_set: BTreeSet<usize> = rpo.iter().map(|b| b.0).collect();
    let reachable_set: BTreeSet<usize> = cfg.reachable_blocks().map(|b| b.0).collect();
    assert_eq!(rpo_set, reachable_set);
    assert!(!rpo_set.contains(&4));
  }
}
