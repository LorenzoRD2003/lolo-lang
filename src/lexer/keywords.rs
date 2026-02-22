use crate::lexer::token::TokenKind;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Queremos un lookup O(1) para keywords en vez de iterar por un slice.
// Como es estatico y conocido en tiempo de compilacion, podemos usar `once_cell::sync::Lazy`.

// Tabla de keywords estatica
pub static KEYWORDS: Lazy<HashMap<&'static str, TokenKind>> = Lazy::new(|| {
  let mut m = HashMap::new();

  // Boolean literals
  m.insert("true", TokenKind::BooleanLiteral);
  m.insert("false", TokenKind::BooleanLiteral);

  // Statements
  m.insert("let", TokenKind::Let);
  m.insert("if", TokenKind::If);
  m.insert("else", TokenKind::Else);
  m.insert("print", TokenKind::Print);

  // Unary operators
  m.insert("neg", TokenKind::Neg);
  m.insert("not", TokenKind::Not);

  // Arithmetic / comparison / logical operators
  m.insert("add", TokenKind::Add);
  m.insert("sub", TokenKind::Sub);
  m.insert("mul", TokenKind::Mul);
  m.insert("div", TokenKind::Div);
  m.insert("eq", TokenKind::Eq);
  m.insert("neq", TokenKind::Neq);
  m.insert("lt", TokenKind::Lt);
  m.insert("gt", TokenKind::Gt);
  m.insert("lte", TokenKind::Lte);
  m.insert("gte", TokenKind::Gte);
  m.insert("and", TokenKind::And);
  m.insert("or", TokenKind::Or);
  m.insert("xor", TokenKind::Xor);

  m
});

// Funcion auxiliar para obtener los keywords de la tabla.
pub(crate) fn lookup_keyword(lexeme: &str) -> TokenKind {
  KEYWORDS
    .get(lexeme)
    .copied()
    .unwrap_or(TokenKind::Identifier)
}
