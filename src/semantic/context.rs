// Contiene el estado compartido mutable:

use crate::{
  diagnostics::diagnostic::Diagnostic,
  semantic::{
    category_checker::category_checker::CategoryInfo,
    compile_time_constant::compile_time_constant_checker::CompileTimeConstantInfo,
    mutability_checker::mutability_checker::MutabilityInfo,
    phase::{PhaseOutput, PhaseOutputInfo},
    resolver::resolution_info::ResolutionInfo,
    symbol_table::SymbolTable,
    type_checker::type_info::TypeInfo,
  },
};

#[derive(Debug)]
pub struct SemanticContext<'a> {
  pub resolution_info: Option<ResolutionInfo>,
  pub symbol_table: Option<SymbolTable>,
  pub type_info: Option<TypeInfo>,
  pub mutability_info: Option<MutabilityInfo>,
  pub compile_time_constant_info: Option<CompileTimeConstantInfo>,
  pub category_info: Option<CategoryInfo>,
  pub diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> SemanticContext<'a> {
  pub fn new(diagnostics: &'a mut Vec<Diagnostic>) -> Self {
    Self {
      resolution_info: None,
      symbol_table: None,
      type_info: None,
      mutability_info: None,
      compile_time_constant_info: None,
      category_info: None,
      diagnostics,
    }
  }

  pub fn apply_phase_output(&mut self, output: PhaseOutput) {
    let (info, diagnostics) = output.consume();
    match info {
      PhaseOutputInfo::Resolution {
        resolution_info,
        symbol_table,
      } => {
        self.resolution_info = Some(resolution_info);
        self.symbol_table = Some(symbol_table);
      }
      PhaseOutputInfo::Types(type_info) => self.type_info = Some(type_info),
      PhaseOutputInfo::Mutability(mutability_info) => self.mutability_info = Some(mutability_info),
      PhaseOutputInfo::Constants(compile_time_constant_info) => {
        self.compile_time_constant_info = Some(compile_time_constant_info)
      }
      PhaseOutputInfo::Categories(category_info) => self.category_info = Some(category_info),
    }
    self.diagnostics.extend(diagnostics);
  }
}
