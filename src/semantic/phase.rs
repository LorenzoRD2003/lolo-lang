// Trait base para cada fase semantica. esto es importante para despues hacer el paralelismo

use crate::{
  ast::{ast::Ast, program::Program},
  diagnostics::diagnostic::Diagnostic,
  semantic::{
    category_checker::category_checker::{CategoryChecker, CategoryInfo},
    compile_time_constant::compile_time_constant_checker::{
      CompileTimeConstantChecker, CompileTimeConstantInfo,
    },
    context::SemanticContext,
    mutability_checker::mutability_checker::{MutabilityChecker, MutabilityInfo},
    resolver::{name_resolver::NameResolver, resolution_info::ResolutionInfo},
    type_checker::{type_checker::TypeChecker, type_info::TypeInfo},
  },
};

#[derive(Debug, Clone)]
pub enum PhaseOutputInfo {
  Resolution(ResolutionInfo),
  Types(TypeInfo),
  Mutability(MutabilityInfo),
  Constants(CompileTimeConstantInfo),
  Categories(CategoryInfo),
}

impl From<ResolutionInfo> for PhaseOutputInfo {
  fn from(value: ResolutionInfo) -> Self {
    PhaseOutputInfo::Resolution(value)
  }
}

impl From<TypeInfo> for PhaseOutputInfo {
  fn from(value: TypeInfo) -> Self {
    PhaseOutputInfo::Types(value)
  }
}

impl From<MutabilityInfo> for PhaseOutputInfo {
  fn from(value: MutabilityInfo) -> Self {
    PhaseOutputInfo::Mutability(value)
  }
}

impl From<CompileTimeConstantInfo> for PhaseOutputInfo {
  fn from(value: CompileTimeConstantInfo) -> Self {
    PhaseOutputInfo::Constants(value)
  }
}

impl From<CategoryInfo> for PhaseOutputInfo {
  fn from(value: CategoryInfo) -> Self {
    PhaseOutputInfo::Categories(value)
  }
}

pub struct PhaseOutput {
  info: PhaseOutputInfo,
  diagnostics: Vec<Diagnostic>,
}

impl PhaseOutput {
  pub fn from(info: PhaseOutputInfo, diagnostics: Vec<Diagnostic>) -> Self {
    Self { info, diagnostics }
  }

  pub fn consume(self) -> (PhaseOutputInfo, Vec<Diagnostic>) {
    (self.info, self.diagnostics)
  }
}

pub trait SemanticPhase<'a>: Send + Sync {
  fn name(&self) -> &'static str;

  fn dependencies(&self) -> &'static [&'static str];

  fn run(&self, ast: &'a Ast, program: &Program, ctx: &SemanticContext) -> PhaseOutput;
}

pub struct NameResolverPhase;
pub struct TypeCheckerPhase;
pub struct MutabilityCheckerPhase;
pub struct CompileTimeConstantCheckerPhase;
pub struct CategoryCheckerPhase;

impl<'a> SemanticPhase<'a> for NameResolverPhase {
  fn name(&self) -> &'static str {
    "NameResolver"
  }

  fn dependencies(&self) -> &'static [&'static str] {
    &[]
  }

  fn run(&self, ast: &'a Ast, program: &Program, _ctx: &SemanticContext) -> PhaseOutput {
    let mut resolver = NameResolver::new(ast);
    resolver.resolve_program(program);
    let diagnostics = resolver.diagnostics().to_vec();
    let info = resolver.into_resolution_info();
    PhaseOutput::from(info.into(), diagnostics)
  }
}

impl<'a> SemanticPhase<'a> for TypeCheckerPhase {
  fn name(&self) -> &'static str {
    "TypeChecker"
  }

  fn dependencies(&self) -> &'static [&'static str] {
    &["NameResolver"]
  }

  fn run(&self, ast: &'a Ast, program: &Program, ctx: &SemanticContext) -> PhaseOutput {
    let resolution_info = ctx.resolution_info.as_ref().unwrap();
    let mut checker = TypeChecker::new(ast, resolution_info);
    checker.check_program(program);
    let diagnostics = checker.diagnostics().to_vec();
    let info = checker.into_type_info();
    PhaseOutput::from(info.into(), diagnostics)
  }
}

impl<'a> SemanticPhase<'a> for MutabilityCheckerPhase {
  fn name(&self) -> &'static str {
    "MutabilityChecker"
  }

  fn dependencies(&self) -> &'static [&'static str] {
    &["NameResolver"]
  }

  fn run(&self, ast: &'a Ast, program: &Program, ctx: &SemanticContext) -> PhaseOutput {
    let resolution_info = ctx.resolution_info.as_ref().unwrap();
    let mut checker = MutabilityChecker::new(ast, resolution_info);
    checker.check_program(program);
    let diagnostics = checker.diagnostics().to_vec();
    let info = checker.into_mutability_info();
    PhaseOutput::from(info.into(), diagnostics)
  }
}

impl<'a> SemanticPhase<'a> for CompileTimeConstantCheckerPhase {
  fn name(&self) -> &'static str {
    "CompileTimeConstantChecker"
  }

  fn dependencies(&self) -> &'static [&'static str] {
    &[]
  }

  fn run(&self, ast: &'a Ast, program: &Program, _ctx: &SemanticContext) -> PhaseOutput {
    let mut checker = CompileTimeConstantChecker::new(ast);
    checker.check_program(program);
    let diagnostics = checker.diagnostics().to_vec();
    let info = checker.into_compile_time_constant_info();
    PhaseOutput::from(info.into(), diagnostics)
  }
}

impl<'a> SemanticPhase<'a> for CategoryCheckerPhase {
  fn name(&self) -> &'static str {
    "CompileTimeConstantChecker"
  }

  fn dependencies(&self) -> &'static [&'static str] {
    &[]
  }

  fn run(&self, ast: &'a Ast, program: &Program, ctx: &SemanticContext) -> PhaseOutput {
    let compile_time_constant_info = ctx.compile_time_constant_info.as_ref().unwrap();
    let mut checker = CategoryChecker::new(ast, compile_time_constant_info);
    checker.check_program(program);
    let diagnostics = checker.diagnostics().to_vec();
    let info = checker.into_category_info();
    PhaseOutput::from(info.into(), diagnostics)
  }
}
