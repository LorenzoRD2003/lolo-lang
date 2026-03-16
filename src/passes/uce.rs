/*!
Unreachable Code Elimination (UCE)
*/

mod plan;

use crate::{
  ir::IrModule,
  passes::{IrPass, PassStats, uce::plan::UcePlan},
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
  fn run(module: &mut IrModule) -> UceStats {
    let plan = Self::build_plan(module);
    Self::rebuild_blocks(module, &plan)
  }

  fn build_plan(module: &IrModule) -> UcePlan {
    todo!()
  }

  fn rebuild_blocks(module: &mut IrModule, plan: &UcePlan) -> UceStats {
    todo!()
  }
}

impl IrPass for UcePass {
  fn name(&self) -> &'static str {
    "dce"
  }

  fn run(&self, module: &mut IrModule) -> PassStats {
    let stats = Self::run(module);
    PassStats::Uce(stats)
  }
}

#[cfg(test)]
mod tests;
