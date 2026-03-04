mod error;
mod name_resolver;
mod resolution_info;

pub(crate) use name_resolver::NameResolver;
pub(crate) use resolution_info::ResolutionInfo;

#[cfg(test)]
pub(crate) use name_resolver::resolve;
