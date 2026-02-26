// Responsabilidad: Definir que es un simbolo.
// Un simbolo representa una entidad declarada en el programa.
// Por ejemplo, puede representar una variable, una funcion, un parametro, una constante, un tipo, etc.
// No todas esas las tenemos aun en lolo-lang.

use crate::{
  ast::expr::VarId,
  common::span::Span,
  semantic::{scope::ScopeId, types::Type},
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SymbolId(pub(crate) usize);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Symbol {
  name: VarId,
  r#type: Type,
  /// Scope de declaracion. El ScopeId referente al scope que contiene a la entidad
  scope: ScopeId,
  /// Informacion sobre mutabilidad de la variable. Para implementar let/const en un futuro.
  /// Por ahora van a ser todas mutables.
  mutability: Mutability,
  /// Lugar que ocupa en el programa. Lo heredamos directamente de la parte sintactica
  span: Span,
  /// Id interno unico del simbolo, para referenciarlo desde el `SemanticInfo`
  id: SymbolId,
}

impl Symbol {
  pub(crate) fn new(
    id: SymbolId,
    name: &VarId,
    r#type: Type,
    scope: ScopeId,
    mutability: Mutability,
    span: Span,
  ) -> Self {
    Self {
      id,
      name: name.clone(),
      r#type,
      scope,
      mutability,
      span,
    }
  }

  /// Devuelve el `SymbolId` unico del simbolo.
  pub(crate) fn id(&self) -> SymbolId {
    self.id
  }

  /// Devuelve el tipo de la variable (`Int32`/`Bool`)
  pub(crate) fn r#type(&self) -> Type {
    self.r#type
  }

  /// Devuelve el nombre del simbolo.
  pub(crate) fn name(&self) -> VarId {
    self.name.clone()
  }

  /// Devuelve el `ScopeId` asociado al bloque donde fue declarado el simbolo.
  pub(crate) fn scope(&self) -> ScopeId {
    self.scope
  }

  /// Indica si el simbolo es mutable.
  pub(crate) fn is_mutable(&self) -> bool {
    self.mutability.is_mutable()
  }

  /// Indica el span del simbolo
  pub(crate) fn span(&self) -> Span {
    self.span.clone()
  }
}
