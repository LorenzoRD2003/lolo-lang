// Contiene el estado compartido mutable:

use crate::{
  diagnostics::Diagnostic,
  semantic::{
    category_checker::CategoryInfo,
    compile_time_constant_checker::CompileTimeConstantInfo,
    mutability_checker::MutabilityInfo,
    name_resolver::ResolutionInfo,
    phase::{PhaseOutput, PhaseOutputInfo},
    symbol_table::SymbolTable,
    type_checker::TypeInfo,
  },
};

#[derive(Debug)]
pub(crate) struct SemanticContext<'a> {
  pub(crate) resolution_info: Option<ResolutionInfo>,
  pub(crate) symbol_table: Option<SymbolTable>,
  pub(crate) type_info: Option<TypeInfo>,
  pub(crate) mutability_info: Option<MutabilityInfo>,
  pub(crate) compile_time_constant_info: Option<CompileTimeConstantInfo>,
  pub(crate) category_info: Option<CategoryInfo>,
  pub(crate) diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> SemanticContext<'a> {
  pub(crate) fn new(diagnostics: &'a mut Vec<Diagnostic>) -> Self {
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

  pub(crate) fn apply_phase_output(&mut self, output: PhaseOutput) {
    let (info, diagnostics) = output.consume();
    match info {
      PhaseOutputInfo::Resolution {
        resolution_info,
        symbol_table,
      } => {
        self.resolution_info = Some(*resolution_info);
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
