#[cfg(any(test, feature = "ir-verify"))]
mod cfg;
#[cfg(any(test, feature = "ir-verify"))]
mod dominators;

#[cfg(any(test, feature = "ir-verify"))]
pub(crate) use cfg::Cfg;
#[cfg(any(test, feature = "ir-verify"))]
pub(crate) use dominators::Dominators;
