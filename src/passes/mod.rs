mod dce;

use crate::{ir::IrModule, passes::dce::DceStats};

pub(crate) enum PassStats {
  Dce(DceStats)
}

pub(crate) trait IrPass {
  fn name(&self) -> &'static str;
  fn run(&self, module: &mut IrModule) -> PassStats;
}
