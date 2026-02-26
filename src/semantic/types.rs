// Los tipos los manejo con un enum simple porque por ahora todo va a tener tipos estaticos
// y los usuarios no van a poder definir tipos.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Type {
  Int32,
  Bool,
}
