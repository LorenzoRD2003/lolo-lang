// El trabajo de un lexer es convertir codigo fuente en una secuencia de tokens
// Nota mental importante: El lexer no entiende el lenguaje, solamente entiende caracteres
// La semantica del programa viene despues
// Solamente emite tokens. Si estoy modelando semantica en el lexer, estoy haciendo algo mal
// Mientras mas tonto sea el lexer, mejor. si no, vienen los bugs.

use crate::{
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  lexer::{
    error::{LexerError, LexerErrorKind},
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
  // el estado primario del lexer es la posicion del puntero y el offset, nada mas
  // fila/columna son para simplificar errores, son metadata derivada
  position: usize,
  next_token_cache: Option<Token>,
  diagnostics: Vec<Diagnostic>,
  emitted_eof: bool,
}

// al final del pipeline, se haria algo como
// for diag in diagnostics {
//   renderer.render(&diag)?;
// }

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str) -> Self {
    Self {
      source,
      position: 0,
      next_token_cache: None,
      diagnostics: Vec::new(),
      emitted_eof: false,
    }
  }

  // Devuelve el siguiente token sin avanzar (lookahead). Esto es util para parsers predictivos
  // La idea es tener un token cacheado, y rellenarlo si es None usando `next()`
  // Para que no cambie el estado, usar una snapshot. `peek()` jamas debe cambiar nada visible
  pub fn peek_token(&mut self) -> Option<Token> {
    if self.next_token_cache.is_none() {
      let snapshot_position = self.position;
      let snapshot_diagnostics_len = self.diagnostics.len();
      let snapshot_emitted_eof = self.emitted_eof;

      self.next_token_cache = self.next().and_then(Result::ok);
      self.position = snapshot_position;
      self.diagnostics.truncate(snapshot_diagnostics_len);
      self.emitted_eof = snapshot_emitted_eof;
    }
    self.next_token_cache.clone()
  }

  // Indica si llegamos al final del input, pregunta "ya consumi todo?"
  // No deberia ser un metodo de Token, porque EOF es solo un tipo de token,
  // pero is_eof() depende de posicionar al lexer para saber si se termino
  fn is_eof(&mut self) -> bool {
    let token = self.peek_token();
    token.is_none() || token.is_some_and(|t| t.kind == TokenKind::EOF)
  }

  // TODO: Hacer correctamente el error UnexpectedEOF
  // fn check_unexpected_eof_error(&mut self) {
  //   if self.is_eof() {
  //     let err = LexerError {
  //       kind: LexerErrorKind::UnexpectedEOF,
  //       span: self.position..(self.position + 1),
  //     };
  //     self.diagnostics.push(err.to_diagnostic());
  //   }
  // }

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
  fn peek_next(&self) -> Option<char> {
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
    Token {
      kind,
      lexeme: self.source[start..self.position].to_string(),
      span: start..self.position,
    }
  }

  // Funcion auxiliar para lexear un numero y devolver el token indicado
  fn lex_number_literal(&mut self) -> Result<Token, LexerError> {
    let start = self.position;
    self.consume_while(|c| c.is_ascii_digit());
    // self.check_unexpected_eof_error();

    if let Some(c) = self.current_char() {
      if is_identifier_start(c) {
        // es un error de IllFormedLiteral. sigo consumiendo hasta llegar al final del literal
        self.advance();
        self.consume_while(is_identifier_continue);
        // self.check_unexpected_eof_error();
        let err = LexerError {
          kind: LexerErrorKind::IllFormedLiteral(self.source[start..self.position].to_string()),
          span: start..self.position,
        };
        self.diagnostics.push(err.to_diagnostic());
        return Err(err);
      }
    }
    Ok(self.make_token(TokenKind::NumberLiteral, start))
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
    let next_c = self.peek_next();
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
    // borrar el cache actual
    self.next_token_cache = None;
    loop {
      match self.current_char() {
        // EOF
        None => {
          if self.emitted_eof {
            return None;
          }
          self.emitted_eof = true;
          return Some(Ok(self.make_token(TokenKind::EOF, self.position)));
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
        Some(c) if c.is_ascii_digit() => return Some(self.lex_number_literal()),

        // Identifiers / Keywords
        // en particular, al terminar de parsear se verifica si el lexema obtenido es una keyword
        Some(c) if is_identifier_start(c) => return Some(Ok(self.lex_identifier_or_keyword())),

        // operadores: todavia no tenemos (serian +, -, *, ==, etc). pero irian aca
        // Regla fundamental: siempre intentar primero reconocer los operadores mas largos
        Some(c) => {
          if let Some(token) = self.lex_operator(c) {
            return Some(Ok(token));
          }
          // si llegamos hasta aca, es un caracter invalido
          let err = LexerError {
            kind: LexerErrorKind::InvalidCharacter(c),
            span: self.position..(self.position + 1),
          };
          self.diagnostics.push(err.to_diagnostic());
          self.advance();
          return Some(Err(err));
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

#[cfg(test)]
mod tests {
  use super::*;
  use proptest::prelude::*;

  #[test]
  fn eof_is_emitted() {
    let mut lexer = Lexer::new("");
    let tok = lexer.next().unwrap().unwrap();
    assert_eq!(tok.kind, TokenKind::EOF);
  }

  #[test]
  fn lex_simple_delimiters_tokens() {
    let src = "( ) { } ;";
    let tokens: Vec<TokenKind> = Lexer::new(src)
      .map(Result::unwrap)
      .map(|t| t.kind)
      .collect();

    assert_eq!(
      tokens,
      vec![
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LCurlyBrace,
        TokenKind::RCurlyBrace,
        TokenKind::Semicolon,
        TokenKind::EOF,
      ]
    );
  }

  #[test]
  fn lex_keywords() {
    let tokens: Vec<TokenKind> = Lexer::new("let true false if else return")
      .map(Result::unwrap)
      .map(|t| t.kind)
      .collect();

    assert_eq!(
      tokens,
      vec![
        TokenKind::Let,
        TokenKind::BooleanLiteral,
        TokenKind::BooleanLiteral,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::Return,
        TokenKind::EOF
      ]
    );
  }

  #[test]
  fn lex_identifier() {
    let src = "hello world";
    let tokens: Vec<_> = Lexer::new(src).map(Result::unwrap).collect();
    // recordar que aun no hay strings en el lenguaje. esas deberian estar entre comillas en un futuro
    assert_eq!(tokens[0].lexeme, "hello");
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].lexeme, "world");
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
  }

  #[test]
  fn lex_number_literal() {
    let src = "12345";
    let token = Lexer::new(src).next().unwrap().unwrap();
    assert_eq!(token.kind, TokenKind::NumberLiteral);
    assert_eq!(token.lexeme, "12345");
  }

  #[test]
  fn lex_operators() {
    let src = "+ == ! != > >= !$ ^/ ^^";
    let lexer = Lexer::new(src);
    let tokens: Vec<TokenKind> = lexer
      .filter_map(|res| res.ok().map(|tok| tok.kind))
      .collect();
    assert_eq!(
      tokens,
      vec![
        TokenKind::Plus,
        TokenKind::EqualEqual,
        TokenKind::Bang,
        TokenKind::BangEqual,
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::Bang,
        TokenKind::Slash,
        TokenKind::CaretCaret,
        TokenKind::EOF,
      ]
    );
  }

  #[test]
  fn lex_ill_formed_literal() {
    let src = "123abc";
    let result = Lexer::new(src).next().unwrap();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind, LexerErrorKind::IllFormedLiteral(src.into()));
    assert_eq!(err.span, 0..6);
  }

  #[test]
  fn lex_invalid_character() {
    let src = "@";
    let result = Lexer::new(src).next().unwrap();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind, LexerErrorKind::InvalidCharacter('@'));
    assert_eq!(err.span, 0..1);
  }

  #[test]
  fn lex_operators_find_invalid_characters() {
    let src = "+ == ! != > >= !$ ^/ ^^";
    let lexer = Lexer::new(src);
    // chequeo que encuentre el InvalidCharacter('$')
    let errors: Vec<LexerError> = lexer.filter_map(|res| res.err()).collect();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].kind, LexerErrorKind::InvalidCharacter('$'));
    assert_eq!(errors[0].span, 16..17);
    assert_eq!(errors[1].kind, LexerErrorKind::InvalidCharacter('^'));
    assert_eq!(errors[1].span, 18..19);
  }

  // test unitario: mezcla operadores + números + identificadores
  #[test]
  fn lex_mixed_input() {
    let input = "a + b * 42 = false";
    let tokens: Vec<TokenKind> = Lexer::new(input)
      .filter_map(|res| res.ok().map(|tok| tok.kind))
      .collect();

    assert_eq!(
      tokens,
      vec![
        TokenKind::Identifier,
        TokenKind::Plus,
        TokenKind::Identifier,
        TokenKind::Star,
        TokenKind::NumberLiteral,
        TokenKind::Equal,
        TokenKind::BooleanLiteral,
        TokenKind::EOF,
      ]
    );
  }

  // necesito que las entradas sean ASCII para que no se rompan los proptest
  proptest! {
  #[test]
    fn lexer_never_panics(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
      let input = String::from_utf8(bytes).unwrap();
      let lexer = Lexer::new(&input);
      for tok in lexer { // iteracion completa
        let _ = tok;
      }
    }
  }

  proptest! {
    #[test]
    fn spans_are_always_valid(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
      let input = String::from_utf8(bytes).unwrap();
      let lexer = Lexer::new(&input);
      for tok in lexer {
        match tok {
          Ok(tok) => {
            prop_assert!(tok.span.start <= tok.span.end);
            prop_assert!(tok.span.end <= input.len());
          }
          Err(err) => {
            prop_assert!(err.span.start <= err.span.end);
            prop_assert!(err.span.end <= input.len());
          }
        }
      }
    }
  }

  // la concatenacion de los lexemas reconstruye el output (excepto por los whitespaces)
  proptest! {
    #[test]
    fn lexemes_reconstruct_input(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
      let input = String::from_utf8(bytes).unwrap();
      let mut lexer = Lexer::new(&input);
      let reconstructed: String = lexer
          .by_ref()
          .filter_map(Result::ok)
          .filter(|t| t.kind != TokenKind::EOF)
          .map(|t| t.lexeme)
          .collect();
      let no_ws: String = input.chars().filter(|c| !c.is_whitespace()).collect();
      // esto funciona en caso de que no haya habido diagnostics durante la ejecucion (por ejemplo un invalid character)
      if lexer.diagnostics.is_empty() {
        prop_assert_eq!(reconstructed, no_ws);
      }
    }
  }

  // peek_token() no cambia estado
  proptest! {
    #[test]
    fn peek_token_is_pure(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
      let input = String::from_utf8(bytes).unwrap();
      let mut lexer = Lexer::new(&input);
      let pos_before = lexer.position;
      let diagnostics_before = lexer.diagnostics.len();
      let emitted_eof = lexer.emitted_eof;
      // Peek arbitrarias veces
      for _ in 0..10 {
        let _ = lexer.peek_token();
      }
      prop_assert_eq!(lexer.position, pos_before);
      prop_assert_eq!(lexer.diagnostics.len(), diagnostics_before);
      prop_assert_eq!(lexer.emitted_eof, emitted_eof);
    }
  }

  // en particular, peek_token() no altera la siguiente ejecucion de next()
  proptest! {
    #[test]
    fn peek_does_not_change_next(bytes in proptest::collection::vec(0u8..=127u8, 0..100)) {
      let input = String::from_utf8(bytes).unwrap();
      let mut lexer1 = Lexer::new(&input);
      let mut lexer2 = Lexer::new(&input);

      for _ in 0..10 {
        lexer1.peek_token(); // lexer1 usa peek
      }
      loop {
        let tok1 = lexer1.next();
        let tok2 = lexer2.next();
        assert_eq!(tok1.is_some(), tok2.is_some());
        if tok1.is_none() { break; }
        assert_eq!(tok1.unwrap().ok().map(|tok| tok.kind), tok2.unwrap().ok().map(|tok| tok.kind));
      }
    }
  }
}
