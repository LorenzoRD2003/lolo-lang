mod error;
mod name_resolver;
mod resolution_info;

pub(crate) use name_resolver::{NameResolver, resolve};
pub(crate) use resolution_info::ResolutionInfo;
