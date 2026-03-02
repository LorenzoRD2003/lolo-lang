use crate::{
  ast::{ast::Ast, program::Program},
  diagnostics::diagnostic::Diagnostic,
  semantic::{
    category_checker::category_checker::{CategoryChecker, CategoryInfo},
    compile_time_constant::compile_time_constant_checker::{
      CompileTimeConstantChecker, CompileTimeConstantInfo,
    },
    mutability_checker::mutability_checker::{MutabilityChecker, MutabilityInfo},
    resolver::{name_resolver::NameResolver, resolution_info::ResolutionInfo},
    type_checker::{type_checker::TypeChecker, type_info::TypeInfo},
  },
};

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticResult {
  resolution_info: ResolutionInfo,
  type_info: TypeInfo,
  mutability_info: MutabilityInfo,
  compile_time_constant_info: CompileTimeConstantInfo,
  category_info: CategoryInfo,
}

#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
  ast: &'a Ast,
  diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> SemanticAnalyzer<'a> {
  pub fn new(ast: &'a Ast, diagnostics: &'a mut Vec<Diagnostic>) -> Self {
    Self { ast, diagnostics }
  }

  pub fn analyze(&mut self, program: &Program) -> SemanticResult {
    let mut name_resolver = NameResolver::new(self.ast, self.diagnostics);
    name_resolver.resolve_program(program);
    let resolution_info = name_resolver.into_resolution_info();

    let mut type_checker = TypeChecker::new(self.ast, &resolution_info, self.diagnostics);
    type_checker.check_program(program);
    let type_info = type_checker.into_type_info();

    let mut mutability_checker =
      MutabilityChecker::new(self.ast, &resolution_info, self.diagnostics);
    mutability_checker.check_program(program);
    let mutability_info = mutability_checker.into_mutability_info();

    let mut compile_time_constant_checker =
      CompileTimeConstantChecker::new(self.ast, self.diagnostics);
    compile_time_constant_checker.check_program(program);
    let compile_time_constant_info =
      compile_time_constant_checker.into_compile_time_constant_info();

    let mut category_checker =
      CategoryChecker::new(self.ast, &compile_time_constant_info, self.diagnostics);
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{ast::expr::ConstValue, parser::program_parsing::parse_program};

  #[test]
  fn semantic_analyzer_collects_all_metadata() {
    let source = r#"
      main {
        let x = 5;
        x = 10;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    let mut analyzer = SemanticAnalyzer::new(&ast, &mut diagnostics);
    let _result = analyzer.analyze(&program);
    assert!(diagnostics.is_empty());
  }

  #[test]
  fn semantic_analyzer_accumulates_diagnostics() {
    let source = r#"
      main {
        x = 10;
        let y = true;
        y = 5;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    let mut analyzer = SemanticAnalyzer::new(&ast, &mut diagnostics);
    let _result = analyzer.analyze(&program);
    // Lo importante: mas de un error
    assert!(diagnostics.len() >= 2);
  }

  #[test]
  fn semantic_analyzer_returns_result_even_with_errors() {
    let source = r#"
      main {
        x = 10;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    let mut analyzer = SemanticAnalyzer::new(&ast, &mut diagnostics);
    let result = analyzer.analyze(&program);
    assert!(!diagnostics.is_empty());
    // aun asi tenemos resultado -> no debe panickear
    let _ = &result.type_info;
  }

  #[test]
  fn constant_info_flows_into_category_checker() {
    let source = r#"
      main {
        let x = 5;
      }
    "#;
    let (ast, program) = parse_program(source);
    let mut diagnostics = Vec::new();
    let mut analyzer = SemanticAnalyzer::new(&ast, &mut diagnostics);
    let result = analyzer.analyze(&program);

    assert!(diagnostics.is_empty());
    // Literal 5 debería ser constante, y su categoría debería incluir CONSTANT
    let compile_time_constant_info = &result.compile_time_constant_info;
    assert!(
      compile_time_constant_info
        .iter()
        .any(|(_, value)| value == &ConstValue::Int32(5))
    );
    let category_info = &result.category_info;
    assert!(category_info.iter().any(|(_, cat)| cat.is_constant()));
  }
}
