mod dce;
mod manager;
mod pass_api;
mod plan;
mod uce;

pub(crate) use manager::PassManager;
pub(crate) use pass_api::{IrPass, PassContext, PassStats};
pub(crate) use plan::PassPlan;
pub(crate) use {dce::DcePass, uce::UcePass};
