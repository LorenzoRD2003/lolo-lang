// Responsabilidad: Definir que es un simbolo.
// Un simbolo representa una entidad declarada en el programa.
// Por ejemplo, puede representar una variable, una funcion, un parametro, una constante, un tipo, etc.
// No todas esas las tenemos aun en lolo-lang.

use crate::{
  ast::StmtId,
  common::{IncrementalId, Span},
  semantic::scope::ScopeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SymbolId(pub(crate) usize);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Symbol {
  /// Id interno unico del simbolo, para referenciarlo desde el `SemanticInfo`
  id: SymbolId,
  /// Nombre de la variable.
  name: String,
  /// Scope de declaracion. El ScopeId referente al scope que contiene a la entidad
  scope: ScopeId,
  /// Lugar que ocupa en el programa. Lo heredamos directamente de la parte sintactica
  span: Span,
}

impl IncrementalId for SymbolId {
  fn from_usize(value: usize) -> Self {
    SymbolId(value)
  }
}

impl Symbol {
  pub(crate) fn new(id: SymbolId, name: String, scope: ScopeId, span: Span) -> Self {
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
  pub(crate) fn name(&self) -> &str {
    &self.name
  }

  // Devuelve el `ScopeId` asociado al bloque donde fue declarado el simbolo.
  // pub fn scope(&self) -> ScopeId {
  //   self.scope
  // }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SymbolData {
  // Stmt que declaro un simbolo (util para buscar redeclaraciones)
  pub(crate) declaration_stmt: StmtId,
}
