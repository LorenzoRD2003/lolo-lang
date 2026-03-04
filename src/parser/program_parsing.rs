use crate::{
  ast::{
    Program, {Ast, BlockId, ExprId, StmtId},
  },
  lexer::lexer::Lexer,
  parser::{parser::Parser, token_stream::TokenStream},
};

pub(crate) fn parse_expr(input: &str) -> (Ast, Option<ExprId>) {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new(input);
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts, &mut diagnostics);
  let expr = parser.parse_expression();
  (parser.into_ast(), expr)
}

pub(crate) fn parse_stmt(input: &str) -> (Ast, Option<StmtId>) {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new(input);
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts, &mut diagnostics);
  let stmt = parser.parse_statement();
  (parser.into_ast(), stmt)
}

pub(crate) fn parse_block(input: &str) -> (Ast, Option<BlockId>) {
  let mut diagnostics = Vec::new();
  let lexer = Lexer::new(input);
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts, &mut diagnostics);
  let block = parser.parse_block();
  (parser.into_ast(), block)
}

pub(crate) fn parse_program(source: &str) -> (Ast, Program) {
  let mut diagnostics = Vec::new();
  let mut ts = TokenStream::new(Lexer::new(source));
  let mut parser = Parser::new(&mut ts, &mut diagnostics);
  let program = parser
    .parse_program()
    .expect("el codigo fuente no pudo ser parseado correctamente");
  let ast = parser.into_ast();
  (ast, program)
}
