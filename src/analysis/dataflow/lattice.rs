//! Define el contrato algebraico del dominio abstracto, para que pueda ser usado por el solver.
//! Conceptualmente: "que significa combinar informacion"

/// Contrato algebraico minimo que debe ofrecer el dominio abstracto de un
/// analisis de dataflow.
pub(crate) trait Lattice {
  /// Un hecho abstracto de dataflow.
  /// El solver solo necesita poder clonarlo y detectar cambios luego de una
  /// iteracion. Cada analisis concreto decide su representacion interna.
  type Fact: Clone + PartialEq;

  /// Elemento neutro/base usado para inicializacion por defecto.
  fn bottom(&self) -> Self::Fact;

  /// Elemento tope del lattice.
  fn top(&self) -> Self::Fact;

  /// Combina dos hechos del dominio.
  fn meet(&self, lhs: &Self::Fact, rhs: &Self::Fact) -> Self::Fact;

  /// Combina una secuencia de hechos del dominio. Si es vacia, devuelve `bottom`.
  fn meet_all<'a>(&self, facts: impl IntoIterator<Item = &'a Self::Fact>) -> Self::Fact
  where
    Self::Fact: 'a,
  {
    let mut iter = facts.into_iter();
    let Some(first) = iter.next() else {
      return self.bottom();
    };

    let mut acc = first.clone();
    for fact in iter {
      acc = self.meet(&acc, fact);
    }
    acc
  }
}
