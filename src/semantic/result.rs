use crate::semantic::{
  category_checker::CategoryInfo, compile_time_constant::CompileTimeConstantInfo,
  context::SemanticContext, mutability_checker::MutabilityInfo, resolver::ResolutionInfo,
  type_checker::TypeInfo,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticResult {
  pub(crate) resolution_info: ResolutionInfo,
  pub(crate) type_info: TypeInfo,
  pub(crate) mutability_info: MutabilityInfo,
  pub(crate) compile_time_constant_info: CompileTimeConstantInfo,
  pub(crate) category_info: CategoryInfo,
}

impl<'a> From<SemanticContext<'a>> for SemanticResult {
  fn from(sc: SemanticContext<'a>) -> Self {
    Self {
      resolution_info: sc.resolution_info.unwrap_or_else(ResolutionInfo::new),
      type_info: sc.type_info.unwrap_or_else(TypeInfo::new),
      mutability_info: sc.mutability_info.unwrap_or_default(),
      compile_time_constant_info: sc.compile_time_constant_info.unwrap_or_default(),
      category_info: sc.category_info.unwrap_or_default(),
    }
  }
}
