/*!
Unreachable Code Elimination (UCE)
*/

mod plan;

use crate::{
  analysis::Cfg,
  ir::{BlockId, InstId, InstKind, IrModule},
  passes::{IrPass, PassContext, PassStats, uce::plan::UcePlan},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UceStats {
  pub(crate) removed_blocks: usize,
  pub(crate) rewritten_jumps: usize,
  pub(crate) rewritten_branches: usize,
  pub(crate) removed_phi_inputs: usize,
}

/// Responsabilidad: ejecutar el pass sobre `IrModule`
#[derive(Debug, Clone)]
pub(crate) struct UcePass;

impl UcePass {
  fn run(module: &mut IrModule, cfg: &Cfg) -> UceStats {
    let uce_plan = Self::build_plan(module, cfg);
    Self::rebuild_module(module, &uce_plan)
  }

  fn build_plan(module: &IrModule, cfg: &Cfg) -> UcePlan {
    // Bloques alcanzables segun los `BlockId` viejos
    let mut reachable = vec![false; module.block_count()];
    let mut block_remap = vec![None; module.block_count()];
    let mut kept_old_blocks = vec![];

    for block in cfg.reachable_blocks() {
      reachable[block.0] = true;
    }

    // Para cada BlockId alcanzable, asigno uno nuevo consecutivo.
    // Para inalcanzables, dejo None.
    for (old_index, is_reachable) in reachable.iter().copied().enumerate() {
      if !is_reachable {
        continue;
      }

      let old_block = BlockId(old_index);
      let new_block = BlockId(kept_old_blocks.len());
      block_remap[old_index] = Some(new_block);
      kept_old_blocks.push(old_block);
    }

    UcePlan::new(block_remap, kept_old_blocks)
  }

  fn rebuild_module(module: &mut IrModule, uce_plan: &UcePlan) -> UceStats {
    let old_count = module.block_count();

    Self::rebuild_block_storage(module, uce_plan);
    let (rewritten_jumps, rewritten_branches, removed_phi_inputs) =
      Self::rewrite_block_references(module, uce_plan);

    UceStats {
      removed_blocks: old_count - module.block_count(),
      rewritten_jumps,
      rewritten_branches,
      removed_phi_inputs,
    }
  }

  fn rebuild_block_storage(module: &mut IrModule, uce_plan: &UcePlan) {
    let old_entry = module.entry_block();
    let mut new_blocks = Vec::with_capacity(uce_plan.kept_old_blocks().len());

    for &old_block in uce_plan.kept_old_blocks() {
      let block_data = module.block(old_block).clone();
      new_blocks.push(block_data);
    }

    module.replace_blocks(new_blocks);

    let new_entry = uce_plan
      .remap_block(old_entry)
      .expect("el entry block debe ser alcanzable");
    module.set_entry_block(new_entry);
  }

  /// Reescribe todas las referencias a bloques usando `block_remap`.
  /// old BlockId -> new BlockId
  /// Tenemos referencias a bloques en los terminadores y en los phi
  fn rewrite_block_references(module: &mut IrModule, uce_plan: &UcePlan) -> (usize, usize, usize) {
    let mut stats = RewriteStats::default();

    for new_bid in (0..module.block_count()).map(BlockId) {
      let (phi_ids, term_id) = Self::snapshot_block_inst_ids(module, new_bid);
      stats.removed_phi_inputs += Self::rewrite_phi_refs_in_block(module, uce_plan, &phi_ids);

      if let Some(term_id) = term_id {
        let term_stats = Self::rewrite_terminator_ref(module, uce_plan, term_id);
        stats.rewritten_jumps += term_stats.rewritten_jumps;
        stats.rewritten_branches += term_stats.rewritten_branches;
      }
    }

    (
      stats.rewritten_jumps,
      stats.rewritten_branches,
      stats.removed_phi_inputs,
    )
  }

  fn snapshot_block_inst_ids(module: &IrModule, bid: BlockId) -> (Vec<InstId>, Option<InstId>) {
    // Snapshot para evitar conflictos de borrow entre block() e inst_mut().
    let block = module.block(bid);
    (
      block.phis().to_vec(),
      if block.has_terminator() {
        Some(block.terminator())
      } else {
        None
      },
    )
  }

  fn rewrite_phi_refs_in_block(
    module: &mut IrModule,
    uce_plan: &UcePlan,
    phi_ids: &[crate::ir::InstId],
  ) -> usize {
    let mut removed_phi_inputs = 0;
    for &phi_id in phi_ids {
      let inst = module.inst_mut(phi_id);
      if let InstKind::Phi { inputs } = &mut inst.kind {
        let before = inputs.len();
        uce_plan.rewrite_phi_inputs(inputs);
        removed_phi_inputs += before.saturating_sub(inputs.len());
      }
    }
    removed_phi_inputs
  }

  fn rewrite_terminator_ref(
    module: &mut IrModule,
    uce_plan: &UcePlan,
    term_id: crate::ir::InstId,
  ) -> RewriteStats {
    let term = module.inst_mut(term_id);
    let (before_jump_target, before_branch_targets) = match &term.kind {
      InstKind::Jump { target } => (Some(*target), None),
      InstKind::Branch {
        if_block,
        else_block,
        ..
      } => (None, Some((*if_block, *else_block))),
      _ => (None, None),
    };

    uce_plan.rewrite_terminator_targets(&mut term.kind);

    let mut stats = RewriteStats::default();
    match (&term.kind, before_jump_target, before_branch_targets) {
      (InstKind::Jump { target }, Some(before), _) if *target != before => {
        stats.rewritten_jumps += 1;
      }
      (
        InstKind::Branch {
          if_block,
          else_block,
          ..
        },
        _,
        Some((before_if, before_else)),
      ) if *if_block != before_if || *else_block != before_else => {
        stats.rewritten_branches += 1;
      }
      _ => {}
    }
    stats
  }
}

#[derive(Debug, Default, Clone, Copy)]
struct RewriteStats {
  rewritten_jumps: usize,
  rewritten_branches: usize,
  removed_phi_inputs: usize,
}

impl IrPass for UcePass {
  fn name(&self) -> &'static str {
    "uce"
  }

  fn run(&self, module: &mut IrModule, ctx: &PassContext) -> PassStats {
    let stats = Self::run(module, ctx.cfg());
    PassStats::Uce(stats)
  }
}

#[cfg(test)]
mod tests;
