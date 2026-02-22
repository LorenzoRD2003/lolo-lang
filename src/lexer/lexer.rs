// El trabajo de un lexer es convertir codigo fuente en una secuencia de tokens
// Nota mental importante: El lexer no entiende el lenguaje, solamente entiende caracteres
// La semantica del programa viene despues
// Solamente emite tokens. Si estoy modelando semantica en el lexer, estoy haciendo algo mal
// Mientras mas tonto sea el lexer, mejor. si no, vienen los bugs.

use crate::lexer::{
  error::LexerError,
  token::{Token, TokenKind},
};

// &'a u8 es un slice del source que vive fuera del Lexer.
// necesitamos un lifetime para garantizar que el lexer no viva más que el source, ya que no lo vamos a clonar
// pensemos al lexer como mover un puntero sobre memoria
// no necesitamos backtracking si hacemos bien las cosas
#[derive(Debug)]
pub struct Lexer<'a> {
  source: &'a str,
  // el estado primario del lexer es la posicion del puntero y el offset, nada mas
  position: usize,
  // fila/columna son para simplificar errores, son metadata derivada de position
  line: usize,
  column: usize,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Self {
      source,
      position: 0,
      line: 1,
      column: 1,
    }
  }

  // Funcion auxiliar para obtener el caracter actual
  // No se debe asumir que hay un caracter actual, asi EOF no es un caso especial
  fn current_char(&self) -> Option<char> {
    self
      .source
      .as_bytes()
      .get(self.position)
      .map(|b| *b as char) // EOF si y solo si es None
  }

  // Funcion auxiliar para ver el siguiente caracter
  fn peek(&self) -> Option<char> {
    self.current_char()
  }

  // Funcion auxiliar para consumir exactamente un caracter, y avanzar
  // la idea es que solamente se actualice `self.position` aca, reduciendo bugs
  fn advance(&mut self) -> Option<char> {
    let ch = self.current_char();
    if let Some(c) = ch {
      // Observemos que EOF no rompe nada
      self.position += 1;
      if c == '\n' {
        self.line += 1;
      } else {
        self.column += 1;
      }
    }
    ch
  }

  // Funcion auxiliar para hacer tokens. El lexer es quien los hace.
  fn make_token(&self, kind: TokenKind, start: usize) -> Token {
    Token {
      kind,
      lexeme: self.source[start..self.position].to_string(),
      span: start..self.position,
    }
  }

  // Devuelve el siguiente token sin avanzar (lookahead)
  fn peek_token(&self) -> Token {
    todo!()
  }

  // Indica si llegamos al final del input, pregunta "ya consumi todo?"
  // No deberia ser un metodo de Token, porque EOF es solo un tipo de token,
  // pero is_eof() depende de posicionar al lexer para saber si se termino
  fn is_eof(&self) -> bool {
    todo!()
  }
}

// Esto nos permite hacer `for token in lexer` despues
impl<'a> Iterator for Lexer<'a> {
  type Item = Result<Token, LexerError>;

  /*
    Devuelve el siguiente token y avanza
    El lexer decide que token producir basandose en el char actual y aplicando reglas
    1. Mirar char actual
    2. Decidir que tipo de "cosa" empieza aca -> revisar la jerarquia de reconocimiento en `manual.md`
    3. Consumir caracteres segun las reglas
    4. Emitir token
  */
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match self.peek() {
        // EOF
        None => return Some(Ok(self.make_token(TokenKind::EOF, self.position))),

        // whitespace: no genera ningun token
        Some(c) if c.is_whitespace() => {
          self.advance();
        }

        // delimitadores: el mapeo es uno a uno. es consumir un solo token
        // Fundamental: No tengo que hardcodear lexemas. Solamente consumir caracteres y fabricar tokens
        Some('(') => {
          let start = self.position;
          self.advance();
          return Some(Ok(self.make_token(TokenKind::LParen, start)));
        }

        Some(')') => {
          let start = self.position;
          self.advance();
          return Some(Ok(self.make_token(TokenKind::RParen, start)));
        }

        Some('{') => {
          let start = self.position;
          self.advance();
          return Some(Ok(self.make_token(TokenKind::LCurlyBrace, start)));
        }

        Some('}') => {
          let start = self.position;
          self.advance();
          return Some(Ok(self.make_token(TokenKind::RCurlyBrace, start)));
        }

        Some(';') => {
          let start = self.position;
          self.advance();
          return Some(Ok(self.make_token(TokenKind::Semicolon, start)));
        }

        // digitos: seguro el token deberia ser un NumberLiteral. hay que consumir todo el numero
        // no mezclar numeros con identificadores: "123abc" deberia dar error
        Some(c) if c.is_ascii_digit() => {
          // return Some(self.lex_number());
          todo!()
        }

        // Identifiers / Keywords
        // en particular, al terminar de parsear se verifica si el lexema obtenido es una keyword
        Some(c) if is_identifier_start(c) => {
          // return Some(self.lex_identifier_or_keyword());
          todo!()
        }

        // operadores: todavia no tenemos (serian +, -, *, ==, etc). pero irian aca
        // Regla fundamental: siempre intentar primero reconocer los operadores mas largos

        // si llegamos hasta aca, es un caracter invalido
        Some(c) => {
          self.advance();
          return Some(Err(LexerError::InvalidCharacter {
            c,
            line: self.line,
            column: self.column,
          }));
        }
      }
    }
  }
}

// Funcion para determinar si el caracter puede ser el inicio de un identificador (variable/keyword)
fn is_identifier_start(c: char) -> bool {
  c.is_ascii_alphabetic() || c == '_'
}

// Funcion para determinar si el caracter puede ser la continuacion de un identificador (variable/keyword)
fn is_identifier_continue(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_'
}
