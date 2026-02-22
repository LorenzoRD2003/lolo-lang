// El trabajo de un lexer es convertir codigo fuente en una secuencia de tokens
// Nota mental importante: El lexer no entiende el lenguaje, solamente entiende caracteres
// La semantica del programa viene despues
// Solamente emite tokens. Si estoy modelando semantica en el lexer, estoy haciendo algo mal
// Mientras mas tonto sea el lexer, mejor. si no, vienen los bugs.