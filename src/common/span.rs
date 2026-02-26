// Cualquier cosa que tenga ubicacion en el archivo fuente, debe tener un Span
// `span.rs` no deberia vivir ni “dentro” del AST ni “dentro” del lexer, sino en un modulo comun que ambos puedan usar.
// El lexer es quien crea los Span, porque es quien conoce las posiciones exactas en el código fuente
// El Span no es un concepto sintactico, sino de infraestructura del compilador.
use std::ops::Range;

pub(crate) type Span = Range<usize>; // start <= x < end
