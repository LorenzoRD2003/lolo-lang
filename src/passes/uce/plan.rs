use crate::ir::{BlockId, InstKind, PhiInput};

pub(crate) struct UcePlan {
  /// Bloques alcanzables segun los `BlockId` viejos
  reachable: Vec<bool>,
  /// Mapa de bloques old -> new
  block_remap: Vec<Option<BlockId>>,
  /// Orden final de reconstruccion de bloques
  kept_old_blocks: Vec<BlockId>,
}

impl UcePlan {
  fn remap_block(&self, old: BlockId) -> Option<BlockId> {
    todo!()
  }

  fn rewrite_terminator_targets(&self, inst_kind: &mut InstKind) {
    todo!()
  }

  fn rewrite_phi_inputs(&self, inputs: &mut Vec<PhiInput>) {
    todo!()
  }
}
