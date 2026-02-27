use crate::{
  ast::ast::Ast,
  semantic::{analyzer::SemanticAnalyzer, scope::ScopeArena, symbol_table::SymbolTable},
};

mod binary;
mod const_folding;
mod const_value;
mod control_flow;
mod scope;
mod unary;
mod var;

// Helper para crear SemanticAnalyzer minimal
pub(crate) fn semantic_analyzer<'a>(ast: &'a Ast) -> SemanticAnalyzer<'a> {
  let scope_arena = ScopeArena::new();
  let symbol_table = SymbolTable::new(scope_arena);
  SemanticAnalyzer::new(ast, symbol_table)
}
