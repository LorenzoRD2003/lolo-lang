/// Las categorias sirven para diferenciar como se puede usar una expresion.
/// Una expresion puede tener mas de una categoria, por lo tanto, la implementacion es con flags/bitmask.
/// Este modelo es extensible en un futuro: VoidExpr, FunctionExpr, CallExpr, ReferenceExpr, etc.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExprCategory(u8);

impl ExprCategory {
  /// La expresion se puede evaluar y se puede leer su valor, pero no es conocida
  /// en tiempo de compilacion.
  const VALUE: u8 = 0b00000001;
  /// La expresion puede ser asignable. Es todo lo que puede aparecer en el lado izquierdo de una asignacion.
  const PLACE: u8 = 0b00000010;
  /// La expresion es literal o conocida en tiempo de compilacion.
  /// En un futuro cuando implemente `const`, tambien iria aca.
  const CONSTANT: u8 = 0b00000100;

  pub(crate) const fn value() -> Self {
    Self(Self::VALUE)
  }

  pub(crate) const fn place() -> Self {
    Self(Self::PLACE)
  }

  pub(crate) const fn constant() -> Self {
    Self(Self::CONSTANT)
  }

  /// Combina dos categorias en una sola. Por ejemplo: Place + Value
  pub(crate) fn with(self, other: ExprCategory) -> Self {
    Self(self.0 | other.0)
  }

  pub(crate) fn is_value(self) -> bool {
    self.0 & Self::VALUE != 0
  }

  pub(crate) fn is_place(self) -> bool {
    self.0 & Self::PLACE != 0
  }

  pub(crate) fn is_constant(self) -> bool {
    self.0 & Self::CONSTANT != 0
  }
}
