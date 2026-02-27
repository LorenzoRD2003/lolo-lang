// Los tipos los manejo con un enum simple porque por ahora todo va a tener tipos estaticos
// y los usuarios no van a poder definir tipos.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
  Int32,
  Bool,
  /// `DefaultType` es para cuando hay un error a la hora de asignar un tipo a una expresion.
  DefaultErrorType,
}

impl Type {
  pub fn to_string(&self) -> &str {
    match &self {
      Self::Int32 => "Int32",
      Self::Bool => "Bool",
      Self::DefaultErrorType => "DefaultErrorType",
    }
  }
}
