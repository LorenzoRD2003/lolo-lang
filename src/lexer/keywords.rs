use crate::lexer::token::TokenKind;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Queremos un lookup O(1) para keywords en vez de iterar por un slice.
// Como es estatico y conocido en tiempo de compilacion, podemos usar `once_cell::sync::Lazy`.

/// Tabla de keywords estatica
pub static KEYWORDS: Lazy<HashMap<&'static str, TokenKind>> = Lazy::new(|| {
  let mut m = HashMap::new();

  // Boolean literals
  m.insert("true", TokenKind::BooleanLiteral);
  m.insert("false", TokenKind::BooleanLiteral);

  // Statements
  m.insert("main", TokenKind::Main);
  m.insert("const", TokenKind::Const);
  m.insert("let", TokenKind::Let);
  m.insert("if", TokenKind::If);
  m.insert("else", TokenKind::Else);
  m.insert("print", TokenKind::Print);
  m.insert("return", TokenKind::Return);

  // Unary operators
  m.insert("neg", TokenKind::Minus);
  m.insert("not", TokenKind::Bang);

  // Arithmetic / comparison / logical operators
  m.insert("add", TokenKind::Plus);
  m.insert("sub", TokenKind::Minus);
  m.insert("mul", TokenKind::Star);
  m.insert("div", TokenKind::Slash);
  m.insert("eq", TokenKind::EqualEqual);
  m.insert("neq", TokenKind::BangEqual);
  m.insert("lt", TokenKind::Less);
  m.insert("gt", TokenKind::Greater);
  m.insert("lte", TokenKind::LessEqual);
  m.insert("gte", TokenKind::GreaterEqual);
  m.insert("and", TokenKind::AndAnd);
  m.insert("or", TokenKind::OrOr);
  m.insert("xor", TokenKind::CaretCaret);

  m
});

// Funcion auxiliar para obtener los keywords de la tabla.
pub(crate) fn lookup_keyword(lexeme: &str) -> TokenKind {
  KEYWORDS
    .get(lexeme)
    .copied()
    .unwrap_or(TokenKind::Identifier)
}
