// Definicion de las reglas del lenguaje.
// Responsabilidad:
// - Binding powers (clave en Pratt)
// - Asociatividad y Precedencia

pub(crate) const ASSIGN_BP: u8 = 10;
pub(crate) const XOR_BP: u8 = 20;
pub(crate) const OR_BP: u8 = 30;
pub(crate) const AND_BP: u8 = 40;
pub(crate) const CMP_BP: u8 = 50;
pub(crate) const ADD_BP: u8 = 60;
pub(crate) const MUL_BP: u8 = 70;
pub(crate) const UNARY_BP: u8 = 80;

// Ejemplos:
// 1 + 2 * 3 -> * gana sobre +
// a && b || c -> AND gana sobre OR
// a < b + c -> La suma se resuelve dentro del RHS
// a < b < c -> el parser detecta chaining de operadores no asociativos. error.
// -a * b -> el operador unario gana.
