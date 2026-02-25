// Responsable de mantener el estado global del analisis.
// Contiene:
// - Una referencia al AST
// - La symbol table
// - Los diagnostics
// - Scope stack
// - Metadata futura
// Es esencial para evitar pasar demasiados parametros a las funciones.
// La logica va en el Analyzer. El estado va en el Context.
