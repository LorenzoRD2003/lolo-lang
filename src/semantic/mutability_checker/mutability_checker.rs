use rustc_hash::FxHashMap;

use crate::{
  ast::{
    ast::{Ast, BlockId, StmtId},
    expr::Expr,
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    mutability_checker::{error::MutabilityError, mutability::Mutability},
    resolver::resolution_info::ResolutionInfo,
    symbol::SymbolId,
  },
};

pub type MutabilityInfo = FxHashMap<SymbolId, Mutability>;

#[derive(Debug)]
pub struct MutabilityChecker<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// Informacion de resolucion de nombres, recibida al consumir el NameResolver.
  resolution_info: &'a ResolutionInfo,
  /// Donde se van acumulando los errores encontrados durante el analisis de mutabilidad.
  diagnostics: &'a mut Vec<Diagnostic>,
  /// Informacion sobre mutabilidad que se va acumulando.
  mutability_info: MutabilityInfo,
}

impl<'a> MutabilityChecker<'a> {
  pub fn new(
    ast: &'a Ast,
    resolution_info: &'a ResolutionInfo,
    diagnostics: &'a mut Vec<Diagnostic>,
  ) -> Self {
    Self {
      ast,
      resolution_info,
      diagnostics,
      mutability_info: FxHashMap::default(),
    }
  }

  pub fn check_program(&mut self, program: &Program) {
    self.check_block(program.main_block());
  }

  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub fn into_mutability_info(self) -> MutabilityInfo {
    self.mutability_info
  }

  // ===================
  // Metodos internos
  // ===================

  /// Resuelve el analisis de mutabilidad para el bloque indicado.
  fn check_block(&mut self, block_id: BlockId) {
    let block = self.ast.block(block_id);
    for stmt_id in block.stmts() {
      self.check_stmt(*stmt_id);
    }
  }

  /// Resuelve el analisis de mutabilidad para el statement indicado.
  fn check_stmt(&mut self, stmt_id: StmtId) {
    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding {
        var,
        initializer: _,
      } => {
        // siempre deberiamos entrar a esta guarda
        if let Some(symbol) = self.resolution_info.symbol_of(var) {
          self.mutability_info.insert(symbol, Mutability::Mutable);
        }
      }
      Stmt::Assign { var, value_expr: _ } => {
        if let Expr::Var(name) = self.ast.expr(var)
          && let Some(symbol) = self.resolution_info.symbol_of(var)
          && let Some(symbol_mutability) = self.mutability_info.get(&symbol)
        {
          if !symbol_mutability.is_mutable() {
            self.emit_error(&MutabilityError::ImmutableVariable {
              name,
              span: self.ast.expr_span(var),
            });
          }
        }
      }
      _ => {}
    }
  }

  fn emit_error(&mut self, err: &MutabilityError) {
    self.diagnostics.push(err.to_diagnostic())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::semantic::resolver::name_resolver::tests::resolve;

  fn mutability_check(source: &str) -> (MutabilityInfo, Vec<Diagnostic>) {
    let (resolution_info, mut diagnostics, ast, program) = resolve(source);
    let mut checker = MutabilityChecker::new(&ast, &resolution_info, &mut diagnostics);
    checker.check_program(&program);
    let type_info = checker.into_mutability_info();
    (type_info, diagnostics)
  }

  #[test]
  fn let_binding_is_marked_mutable() {
    let (mutability_info, diagnostics) = mutability_check("main { let x = 5;}");
    assert!(diagnostics.is_empty());
    assert_eq!(mutability_info.len(), 1);
    let (_, mutability) = mutability_info.iter().next().unwrap();
    assert!(mutability.is_mutable());
  }

  #[test]
  fn assign_to_mutable_variable_is_ok() {
    let (_, diagnostics) = mutability_check("main { let x = 5; x = 10; }");
    assert!(diagnostics.is_empty());
  }

  #[test]
  fn multiple_let_bindings_are_all_mutable() {
    let source = r#"
      main {
        let x = 1;
        let y = 2;
        let z = 3;
      }
    "#;
    let (mutability_info, diagnostics) = mutability_check(source);
    assert!(diagnostics.is_empty());
    assert_eq!(mutability_info.len(), 3);
    for (_, mutability) in &mutability_info {
      assert!(mutability.is_mutable());
    }
  }

  #[test]
  fn assignment_inside_if_is_ok() {
    let source = r#"
      main {
        let x = 5;
        if x {
          x = 10;
        }
      }
    "#;
    let (_, diagnostics) = mutability_check(source);
    assert!(diagnostics.is_empty());
  }

  #[test]
  fn assign_without_let_does_not_crash_checker() {
    // El resolver debería haber producido error antes.
    // El mutability checker no debería crashear.
    // No afirmamos nada fuerte, solo que no panic.
    mutability_check("main { x = 10; }");
  }
}
