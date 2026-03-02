// El trabajo de un lexer es convertir codigo fuente en una secuencia de tokens
// Nota mental importante: El lexer no entiende el lenguaje, solamente entiende caracteres
// La semantica del programa viene despues
// Solamente emite tokens. Si estoy modelando semantica en el lexer, estoy haciendo algo mal
// Mientras mas tonto sea el lexer, mejor. si no, vienen los bugs.

use crate::{
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::{
    error::LexerError,
    keywords::lookup_keyword,
    operators::match_operator,
    token::{Token, TokenKind},
  },
};

// &'a u8 es un slice del source que vive fuera del Lexer.
// necesitamos un lifetime para garantizar que el lexer no viva más que el source, ya que no lo vamos a clonar
// pensemos al lexer como mover un puntero sobre memoria
// no necesitamos backtracking si hacemos bien las cosas

#[derive(Debug)]
pub struct Lexer<'a> {
  source: &'a str,
  /// El estado primario del lexer es la posicion del puntero y el offset, nada mas.
  /// Fila/columna son para simplificar errores, son metadata derivada.
  position: usize,
  emitted_eof: bool,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Self {
      source,
      position: 0,
      emitted_eof: false,
    }
  }

  /*
    Devuelve el siguiente token y avanza
    El lexer decide que token producir basandose en el char actual y aplicando reglas
    1. Mirar char actual
    2. Decidir que tipo de "cosa" empieza aca -> revisar la jerarquia de reconocimiento en `manual.md`
    3. Consumir caracteres segun las reglas
    4. Emitir token
  */
  pub fn next(&mut self, diagnostics: &mut Vec<Diagnostic>) -> Option<Token> {
    loop {
      match self.current_char() {
        // EOF
        None => {
          if self.emitted_eof {
            return None;
          }
          self.emitted_eof = true;
          return Some(self.make_token(TokenKind::EOF, self.position));
        }

        // whitespace: no genera ningun token
        Some(c) if c.is_whitespace() => {
          self.consume_while(char::is_whitespace);
          continue;
        }

        // delimitadores: el mapeo es uno a uno. es consumir un solo token
        // Fundamental: No tengo que hardcodear lexemas. Solamente consumir caracteres y fabricar tokens
        Some('(') => {
          let start = self.position;
          self.advance();
          return Some(self.make_token(TokenKind::LParen, start));
        }

        Some(')') => {
          let start = self.position;
          self.advance();
          return Some(self.make_token(TokenKind::RParen, start));
        }

        Some('{') => {
          let start = self.position;
          self.advance();
          return Some(self.make_token(TokenKind::LCurlyBrace, start));
        }

        Some('}') => {
          let start = self.position;
          self.advance();
          return Some(self.make_token(TokenKind::RCurlyBrace, start));
        }

        Some(';') => {
          let start = self.position;
          self.advance();
          return Some(self.make_token(TokenKind::Semicolon, start));
        }

        // digitos: seguro el token deberia ser un NumberLiteral. hay que consumir todo el numero
        // no mezclar numeros con identificadores: "123abc" deberia dar error
        Some(c) if c.is_ascii_digit() => return self.lex_number_literal(diagnostics),

        // Identifiers / Keywords
        // en particular, al terminar de parsear se verifica si el lexema obtenido es una keyword
        Some(c) if is_identifier_start(c) => return Some(self.lex_identifier_or_keyword()),

        // operadores: todavia no tenemos (serian +, -, *, ==, etc). pero irian aca
        // Regla fundamental: siempre intentar primero reconocer los operadores mas largos
        Some(c) => {
          if let Some(token) = self.lex_operator(c) {
            return Some(token);
          }
          // si llegamos hasta aca, es un caracter invalido
          self.emit_error(
            diagnostics,
            &LexerError::InvalidCharacter(c, self.position..(self.position + 1)),
          );
          self.advance();
          continue;
        }
      }
    }
  }

  // ============================
  // Helpers internos
  // ============================

  // Funcion auxiliar para obtener el caracter actual
  // No se debe asumir que hay un caracter actual, asi EOF no es un caso especial
  fn current_char(&self) -> Option<char> {
    self
      .source
      .as_bytes()
      .get(self.position)
      .map(|b| *b as char) // EOF si y solo si es None
  }

  // Devuelve el siguiente caracter, si lo hay
  fn peek_next_char(&self) -> Option<char> {
    self.source[self.position + 1..].chars().next()
  }

  // Funcion auxiliar para consumir exactamente un caracter, y avanzar
  // la idea es que solamente se actualice `self.position` aca, reduciendo bugs
  fn advance(&mut self) -> Option<char> {
    let ch = self.current_char();
    if let Some(_) = ch {
      // Observemos que EOF no rompe nada
      self.position += 1;
    }
    ch
  }

  // Funcion auxiliar para consumir caracteres mientras valga cierto predicado
  fn consume_while<F: Fn(char) -> bool>(&mut self, predicate: F) {
    while matches!(self.current_char(), Some(c) if predicate(c)) {
      self.advance();
    }
  }

  // Funcion auxiliar para hacer tokens. El lexer es quien los hace.
  fn make_token(&self, kind: TokenKind, start: usize) -> Token {
    Token::new(
      kind,
      self.source[start..self.position].to_string(),
      start..self.position,
    )
  }

  fn emit_error(&self, diagnostics: &mut Vec<Diagnostic>, err: &LexerError) {
    diagnostics.push(err.to_diagnostic());
  }

  // =========================
  // Lexers especificos
  // =========================

  // Funcion auxiliar para lexear un numero y devolver el token indicado
  fn lex_number_literal(&mut self, diagnostics: &mut Vec<Diagnostic>) -> Option<Token> {
    let start = self.position;
    self.consume_while(|c| c.is_ascii_digit());
    // self.check_unexpected_eof_error();

    if let Some(c) = self.current_char()
      && is_identifier_start(c)
    {
      // es un error de IllFormedLiteral. sigo consumiendo hasta llegar al final del literal
      self.advance();
      self.consume_while(is_identifier_continue);
      self.emit_error(
        diagnostics,
        &LexerError::IllFormedLiteral(
          self.source[start..self.position].to_string(),
          start..self.position,
        ),
      );
      return None;
    }
    Some(self.make_token(TokenKind::NumberLiteral, start))
  }

  fn lex_identifier_or_keyword(&mut self) -> Token {
    let start = self.position;
    self.advance();
    self.consume_while(is_identifier_continue);
    let lexeme = &self.source[start..self.position];

    // Aca voy a verificar las keywords con una HashMap, para hacer lookup en O(1) en vez de iterar la lista de keywords
    let kind = lookup_keyword(lexeme);
    self.make_token(kind, start)
  }

  fn lex_operator(&mut self, c: char) -> Option<Token> {
    let next_c = self.peek_next_char();
    if let Some((kind, width)) = match_operator(c, next_c) {
      let start = self.position;
      for _ in 0..width {
        self.advance();
      }
      return Some(self.make_token(kind, start));
    }
    None
  }
}

// =========================
// Helpers de clasificacion
// =========================

// Funcion para determinar si el caracter puede ser el inicio de un identificador (variable/keyword)
fn is_identifier_start(c: char) -> bool {
  c.is_ascii_alphabetic() || c == '_'
}

// Funcion para determinar si el caracter puede ser la continuacion de un identificador (variable/keyword)
fn is_identifier_continue(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests;
