// Responsabilidad: Chequear invariantes internas de la IR.
// - todos los BlockId, InstId, ValueId, LocalId usados existen;
// - cada bloque termina en terminador;
// - no hay instrucciones después del terminador;
// - el tipo del Branch.cond es Bool;
// - el tipo del Return coincide con el tipo de retorno de la función;
// - Load y Store usan el tipo correcto del local;
// - los operandos de un BinaryOp tienen tipos compatibles;
// - toda función tiene entry.
