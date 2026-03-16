use crate::ir::{BlockId, InstKind, PhiInput};

pub(crate) struct UcePlan {
  /// Mapa de bloques old -> new
  block_remap: Vec<Option<BlockId>>,
  /// Orden final de reconstruccion de bloques
  kept_old_blocks: Vec<BlockId>,
}

impl UcePlan {
  pub(crate) fn new(block_remap: Vec<Option<BlockId>>, kept_old_blocks: Vec<BlockId>) -> Self {
    Self {
      block_remap,
      kept_old_blocks,
    }
  }

  pub(crate) fn kept_old_blocks(&self) -> &[BlockId] {
    &self.kept_old_blocks
  }

  pub(crate) fn remap_block(&self, old: BlockId) -> Option<BlockId> {
    self.block_remap[old.0]
  }

  pub(crate) fn rewrite_terminator_targets(&self, inst_kind: &mut InstKind) {
    match inst_kind {
      InstKind::Jump { target } => {
        *target = self
          .remap_block(*target)
          .expect("todo target alcanzable debe tener remap en UCE");
      }
      InstKind::Branch {
        if_block,
        else_block,
        ..
      } => {
        *if_block = self
          .remap_block(*if_block)
          .expect("if_block alcanzable debe tener remap en UCE");
        *else_block = self
          .remap_block(*else_block)
          .expect("else_block alcanzable debe tener remap en UCE");
      }
      _ => {}
    }
  }

  pub(crate) fn rewrite_phi_inputs(&self, inputs: &mut Vec<PhiInput>) {
    let mut rewritten = Vec::with_capacity(inputs.len());

    for input in inputs.iter() {
      if let Some(new_pred) = self.remap_block(input.pred_block()) {
        rewritten.push(PhiInput::new(new_pred, input.value()));
      }
    }

    *inputs = rewritten;
  }
}
