use std::ops::Range;

// Cualquier cosa que tenga ubicacion en el archivo fuente, debe tener un Span
// Vamos a hacer un trait que funcione como una abstraccion semantica para "tiene ubicacion en el archivo fuente"
// Por ahora todos los .span() van a estar en todo!() porque no hay un lexer implementado todavía.

pub type Span = Range<usize>; // start <= x < end

pub trait Spanned {
  fn span(&self) -> Span;
}
