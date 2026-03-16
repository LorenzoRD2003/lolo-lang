mod dce;
mod uce;

use crate::{
  ir::IrModule,
  passes::{dce::DceStats, uce::UceStats},
};
pub(crate) use dce::DcePass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PassStats {
  Dce(DceStats),
  Uce(UceStats),
}

pub(crate) trait IrPass {
  #[allow(dead_code)]
  fn name(&self) -> &'static str;
  fn run(&self, module: &mut IrModule) -> PassStats;
}
