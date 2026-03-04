// Responsabilidad: Definir que es un simbolo.
// Un simbolo representa una entidad declarada en el programa.
// Por ejemplo, puede representar una variable, una funcion, un parametro, una constante, un tipo, etc.
// No todas esas las tenemos aun en lolo-lang.

use crate::{
  ast::ast::StmtId,
  common::span::Span,
  semantic::id_generator::{ScopeId, SymbolId},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
  /// Id interno unico del simbolo, para referenciarlo desde el `SemanticInfo`
  id: SymbolId,
  /// Nombre de la variable.
  name: String,
  /// Scope de declaracion. El ScopeId referente al scope que contiene a la entidad
  scope: ScopeId,
  /// Lugar que ocupa en el programa. Lo heredamos directamente de la parte sintactica
  span: Span,
}

impl Symbol {
  pub fn new(id: SymbolId, name: String, scope: ScopeId, span: Span) -> Self {
    Self {
      id,
      name,
      scope,
      span,
    }
  }

  // Devuelve el `SymbolId` unico del simbolo.
  // pub fn id(&self) -> SymbolId {
  //   self.id
  // }

  /// Devuelve el nombre del simbolo.
  pub fn name(&self) -> &str {
    &self.name
  }

  // Devuelve el `ScopeId` asociado al bloque donde fue declarado el simbolo.
  // pub fn scope(&self) -> ScopeId {
  //   self.scope
  // }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolData {
  // Stmt que declaro un simbolo (util para buscar redeclaraciones)
  pub declaration_stmt: StmtId,
}

