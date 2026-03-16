use crate::{
  analysis::{Cfg, CfgError},
  ir::IrModule,
  passes::{dce::DceStats, uce::UceStats},
};

#[derive(Debug, Clone)]
pub(crate) struct PassContext {
  cfg: Cfg,
}

impl PassContext {
  pub(crate) fn from_module(module: &IrModule) -> Result<Self, Vec<CfgError>> {
    let mut errors = Vec::new();
    let cfg = Cfg::build(module, module.entry_block(), &mut errors);
    if errors.is_empty() {
      Ok(Self { cfg })
    } else {
      Err(errors)
    }
  }

  pub(crate) fn cfg(&self) -> &Cfg {
    &self.cfg
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PassStats {
  Dce(DceStats),
  Uce(UceStats),
}

pub(crate) trait IrPass {
  #[allow(dead_code)]
  fn name(&self) -> &'static str;
  fn run(&self, module: &mut IrModule, ctx: &PassContext) -> PassStats;
}
