// Los tipos los manejo con un enum simple porque por ahora todo va a tener tipos estaticos
// y los usuarios no van a poder definir tipos.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Type {
  Int32,
  Bool,
}

impl Type {
  pub(crate) fn to_string(&self) -> &str {
    match &self {
      Self::Int32 => "Int32",
      Self::Bool => "Bool",
    }
  }
}
