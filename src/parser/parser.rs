// El Parser en si mismo.
// Responsabilidad:
// - Implementar algoritmo Pratt parsing
// - Construir AST (o sea los ExprId)
// - Fusionar spans
// - Emitir errores sintacticos (luego el que los muestra es el Renderer de diagnostics)

use crate::ast::expr::Expr;

// Parser guarda la referencia a la arena y al stream de tokens

