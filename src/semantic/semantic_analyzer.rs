use crate::{
  ast::{ast::Ast, program::Program},
  semantic::{
    category_checker::category_checker::{CategoryChecker, CategoryInfo},
    compile_time_constant::{
      self,
      compile_time_constant_checker::{CompileTimeConstantChecker, CompileTimeConstantInfo},
    },
    mutability_checker::mutability_checker::{MutabilityChecker, MutabilityInfo},
    resolver::{name_resolver::NameResolver, resolution_info::ResolutionInfo},
    type_checker::{self, type_checker::TypeChecker, type_info::TypeInfo},
  },
};

#[derive(Debug, Clone)]
pub struct SemanticResult {
  resolution_info: ResolutionInfo,
  type_info: TypeInfo,
  mutability_info: MutabilityInfo,
  compile_time_constant_info: CompileTimeConstantInfo,
  category_info: CategoryInfo,
}

#[derive(Debug, Clone)]
pub struct SemanticAnalyzer<'a> {
  ast: &'a Ast,
}

impl<'a> SemanticAnalyzer<'a> {
  pub fn analyze(&self, program: &Program) -> SemanticResult {
    let mut name_resolver = NameResolver::new(self.ast);
    name_resolver.resolve_program(program);
    let resolution_info = name_resolver.into_resolution_info();

    let mut type_checker = TypeChecker::new(self.ast, &resolution_info);
    type_checker.check_program(program);
    let type_info = type_checker.into_type_info();

    let mut mutability_checker = MutabilityChecker::new(self.ast, &resolution_info);
    mutability_checker.check_program(program);
    let mutability_info = mutability_checker.into_mutability_info();

    let mut compile_time_constant_checker = CompileTimeConstantChecker::new(self.ast);
    compile_time_constant_checker.check_program(program);
    let compile_time_constant_info =
      compile_time_constant_checker.into_compile_time_constant_info();

    let mut category_checker = CategoryChecker::new(self.ast, &compile_time_constant_info);
    category_checker.check_program(program);
    let category_info = category_checker.into_category_info();

    SemanticResult {
      resolution_info,
      type_info,
      mutability_info,
      compile_time_constant_info,
      category_info,
    }
  }
}
