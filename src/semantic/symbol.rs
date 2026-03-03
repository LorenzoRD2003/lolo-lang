// Responsabilidad: Definir que es un simbolo.
// Un simbolo representa una entidad declarada en el programa.
// Por ejemplo, puede representar una variable, una funcion, un parametro, una constante, un tipo, etc.
// No todas esas las tenemos aun en lolo-lang.

use crate::{ast::expr::VarId, common::span::Span, semantic::scope::ScopeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
  name: VarId,
  /// Scope de declaracion. El ScopeId referente al scope que contiene a la entidad
  scope: ScopeId,
  /// Lugar que ocupa en el programa. Lo heredamos directamente de la parte sintactica
  span: Span,
  /// Id interno unico del simbolo, para referenciarlo desde el `SemanticInfo`
  id: SymbolId,
}

impl Symbol {
  pub fn new(id: SymbolId, name: &VarId, scope: ScopeId, span: Span) -> Self {
    Self {
      id,
      name: name.clone(),
      scope,
      span,
    }
  }

  /// Devuelve el `SymbolId` unico del simbolo.
  pub fn id(&self) -> SymbolId {
    self.id
  }

  /// Devuelve el nombre del simbolo.
  pub fn name(&self) -> VarId {
    self.name.clone()
  }

  /// Devuelve el `ScopeId` asociado al bloque donde fue declarado el simbolo.
  pub fn scope(&self) -> ScopeId {
    self.scope
  }
}
