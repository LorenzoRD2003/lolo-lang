use rustc_hash::FxHashMap;

use crate::{
  ast::{Ast, AstVisitor, BlockId, ExprId, Stmt, StmtId, walk_block, walk_expr, walk_stmt},
  diagnostics::{Diagnosable, Diagnostic},
  semantic::{
    mutability_checker::error::MutabilityError, resolver::ResolutionInfo, symbol::SymbolId,
    symbol_table::SymbolTable,
  },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Mutability {
  Mutable,
  Immutable,
}

impl Mutability {
  pub(crate) fn is_mutable(&self) -> bool {
    match self {
      Mutability::Mutable => true,
      Mutability::Immutable => false,
    }
  }
}

pub(crate) type MutabilityInfo = FxHashMap<SymbolId, Mutability>;

#[derive(Debug)]
pub(crate) struct MutabilityChecker<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// Informacion de resolucion de nombres, recibida al consumir el NameResolver.
  resolution_info: &'a ResolutionInfo,
  /// Tabla de simbolos, recibida al consumir el NameResolver.
  symbol_table: &'a SymbolTable,
  /// Donde se van acumulando los errores encontrados durante el analisis de mutabilidad.
  diagnostics: Vec<Diagnostic>,
  /// Informacion sobre mutabilidad que se va acumulando.
  mutability_info: MutabilityInfo,
}

impl<'a> MutabilityChecker<'a> {
  pub(crate) fn new(
    ast: &'a Ast,
    resolution_info: &'a ResolutionInfo,
    symbol_table: &'a SymbolTable,
  ) -> Self {
    Self {
      ast,
      resolution_info,
      symbol_table,
      diagnostics: Vec::new(),
      mutability_info: FxHashMap::default(),
    }
  }

  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  pub(crate) fn into_mutability_info(self) -> MutabilityInfo {
    self.mutability_info
  }

  fn emit_error(&mut self, err: &MutabilityError) {
    self.diagnostics.push(err.to_diagnostic())
  }
}

impl AstVisitor for MutabilityChecker<'_> {
  /// Resuelve el analisis de mutabilidad para el bloque indicado.
  fn visit_block(&mut self, block_id: BlockId) {
    walk_block(self, self.ast, block_id);
  }

  /// Resuelve el analisis de mutabilidad para el statement indicado.
  fn visit_stmt(&mut self, stmt_id: StmtId) {
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
      Stmt::ConstBinding {
        var,
        initializer: _,
      } => {
        if let Some(symbol) = self.resolution_info.symbol_of(var) {
          self.mutability_info.insert(symbol, Mutability::Immutable);
        }
      }
      Stmt::Assign { var, value_expr: _ } => {
        if let Some(symbol) = self.resolution_info.symbol_of(var)
          && self
            .mutability_info
            .get(&symbol)
            .is_some_and(|m| !m.is_mutable())
        {
          self.emit_error(&MutabilityError::ImmutableVariable {
            name: self.symbol_table.symbol(symbol).name().to_string(),
            span: self.ast.expr_span(var),
          });
        }
      }
      _ => {}
    }
    walk_stmt(self, self.ast, stmt_id);
  }

  /// Resuelve el analisis de mutabilidad para la expresion indicada.
  fn visit_expr(&mut self, expr_id: ExprId) {
    walk_expr(self, self.ast, expr_id);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::semantic::resolver::resolve;

  fn mutability_check(source: &str) -> (MutabilityInfo, Vec<Diagnostic>) {
    let (resolution_info, symbol_table, _, ast, program) = resolve(source);
    let mut checker = MutabilityChecker::new(&ast, &resolution_info, &symbol_table);
    checker.visit_program(&program);
    let diagnostics = checker.diagnostics().to_vec();
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
  fn const_binding_is_immutable() {
    let (_, diagnostics) = mutability_check("main { const x = 5; x = 10; }");
    assert!(!diagnostics.is_empty());
    assert!(
      diagnostics[0]
        .msg()
        .contains(&format!("se intento modificar la variable inmutable 'x'"))
    );
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

  #[test]
  fn assign_to_outer_block_inside_inner_block_is_error() {
    let source = r#"
      main {
        const x = 5;
        {
          x = 2;
        };
      }
    "#;
    let (_, diagnostics) = mutability_check(source);
    assert!(!diagnostics.is_empty());
  }

  #[test]
  fn shadowing_const_with_mutable_inside_block_is_ok() {
    let source = r#"
      main {
        const x = 5;
        {
          let x = 2;
          x = 3;
        };
      }
    "#;
    let (_, diagnostics) = mutability_check(source);
    assert!(diagnostics.is_empty());
  }
}
