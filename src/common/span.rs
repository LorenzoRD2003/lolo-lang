use std::ops::Range;

// Cualquier cosa que tenga ubicacion en el archivo fuente, debe tener un Span
// Vamos a hacer un trait que funcione como una abstraccion semantica para "tiene ubicacion en el archivo fuente"
// Por ahora todos los .span() van a estar en todo!() porque no hay un lexer implementado todavia.

pub(crate) type Span = Range<usize>; // start <= x < end

// `span.rs` no deberia vivir ni “dentro” del AST ni “dentro” del lexer, sino en un modulo comun que ambos puedan usar.
// El lexer es quien crea los Span, porque es quien conoce las posiciones exactas en el código fuente
// El Span no es un concepto sintactico, sino de infraestructura del compilador.
