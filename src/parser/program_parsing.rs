use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    program::Program,
  },
  lexer::lexer::Lexer,
  parser::{parser::Parser, token_stream::TokenStream},
};

pub(crate) fn parse_expr(input: &str) -> (Ast, Option<ExprId>) {
  let lexer = Lexer::new(input);
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts);
  let expr = parser.parse_expression();
  (parser.into_ast(), expr)
}

pub(crate) fn parse_stmt(input: &str) -> (Ast, Option<StmtId>) {
  let lexer = Lexer::new(input);
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts);
  let stmt = parser.parse_statement();
  (parser.into_ast(), stmt)
}

pub(crate) fn parse_block(input: &str) -> (Ast, Option<BlockId>) {
  let lexer = Lexer::new(input);
  let mut ts = TokenStream::new(lexer);
  let mut parser = Parser::new(&mut ts);
  let block = parser.parse_block();
  (parser.into_ast(), block)
}

pub(crate) fn parse_program(source: &str) -> (Ast, Program) {
  let mut ts = TokenStream::new(Lexer::new(source));
  let mut parser = Parser::new(&mut ts);
  let program = parser
    .parse_program()
    .expect("el codigo fuente no pudo ser parseado correctamente");
  let ast = parser.into_ast();
  (ast, program)
}
