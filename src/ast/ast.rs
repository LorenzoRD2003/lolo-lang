// Responsabilidades del AST: crear nodos, asociarles un Span, y devolver el ExprId
// Usando empty() y add_expr() puedo crear todos los nodos.
// no es exactamente un arbol ahora, sino una arena de nodos (pool)
// es importante remarcar que una Expr NO TIENE Span, sino que el compilador guarda un Span asociado a cada nodo del "arbol"

use crate::{
  ast::{
    expr::Expr,
    stmt::{Block, Stmt},
  },
  common::span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// Esto es arena-based allocation
/// por como escalaria en un futuro, es mejor que un Vec<(Expr, Span)>.
#[derive(Debug, Clone)]
pub struct Ast {
  /// Se debe cumplir el invariante de que los expr_arena, expr_spans esten asociados por el indice ExprId.
  expr_arena: Vec<Expr>,
  expr_spans: Vec<Span>,
  /// Se debe cumplir el invariante de que los stmt_arena, stmt_spans esten asociados por el indice StmtId.
  stmt_arena: Vec<Stmt>,
  stmt_spans: Vec<Span>,
  /// Se debe cumplir el invariante de que los block_arena, block_spans esten asociados por el indice BlockId.
  block_arena: Vec<Block>,
  block_spans: Vec<Span>,
}

impl Ast {
  pub fn empty() -> Self {
    Self {
      expr_arena: vec![],
      expr_spans: vec![],
      stmt_arena: vec![],
      stmt_spans: vec![],
      block_arena: vec![],
      block_spans: vec![],
    }
  }

  pub fn expr(&self, id: ExprId) -> Expr {
    self.expr_arena[id.0].clone()
  }

  pub fn expr_span(&self, id: ExprId) -> Span {
    self.expr_spans[id.0].clone()
  }

  pub fn add_expr(&mut self, expr: Expr, span: Span) -> ExprId {
    self.expr_arena.push(expr);
    self.expr_spans.push(span);
    ExprId(self.expr_arena.len() - 1)
  }

  pub fn update_expr_span(&mut self, id: ExprId, span: Span) -> ExprId {
    self.expr_spans[id.0] = span;
    id
  }

  pub fn stmt(&self, id: StmtId) -> Stmt {
    self.stmt_arena[id.0].clone()
  }

  pub fn stmt_span(&self, id: StmtId) -> Span {
    self.stmt_spans[id.0].clone()
  }

  pub fn add_stmt(&mut self, stmt: Stmt, span: Span) -> StmtId {
    self.stmt_arena.push(stmt);
    self.stmt_spans.push(span);
    StmtId(self.stmt_arena.len() - 1)
  }

  pub fn update_stmt_span(&mut self, id: StmtId, span: Span) -> StmtId {
    self.stmt_spans[id.0] = span;
    id
  }

  pub fn block(&self, id: BlockId) -> Block {
    self.block_arena[id.0].clone()
  }

  pub fn block_span(&self, id: BlockId) -> Span {
    self.block_spans[id.0].clone()
  }

  pub fn add_block(&mut self, block: Block, span: Span) -> BlockId {
    self.block_arena.push(block);
    self.block_spans.push(span);
    BlockId(self.block_arena.len() - 1)
  }

  pub fn update_block_span(&mut self, id: BlockId, span: Span) -> BlockId {
    self.block_spans[id.0] = span;
    id
  }
}

#[cfg(test)]
mod tests;
