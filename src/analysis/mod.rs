mod cfg;
#[cfg(any(test, feature = "ir-verify"))]
mod dominators;

pub(crate) use cfg::Cfg;
pub(crate) use cfg::CfgError;
#[cfg(any(test, feature = "ir-verify"))]
pub(crate) use dominators::Dominators;
