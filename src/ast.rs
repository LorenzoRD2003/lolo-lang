// Responsabilidades del AST: crear nodos, asociarles un Span, y devolver el ExprId
// Usando empty() y add_expr() puedo crear todos los nodos.
// no es exactamente un arbol ahora, sino una arena de nodos (pool)
// es importante remarcar que una Expr NO TIENE Span, sino que el compilador guarda un Span asociado a cada nodo del "arbol"

mod block;
mod expr;
mod program;
mod stmt;
mod visitor;

pub(crate) use block::Block;
pub(crate) use expr::{BinaryExpr, BinaryOp, ConstValue, Expr, IfExpr, UnaryExpr, UnaryOp};
pub(crate) use program::Program;
pub(crate) use stmt::Stmt;
pub(crate) use visitor::{AstVisitor, walk_block, walk_expr, walk_stmt};

use crate::common::{IdGenerator, IncrementalId, IncrementalIdGenerator, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ExprId(pub(crate) usize);

impl IncrementalId for ExprId {
  fn from_usize(value: usize) -> Self {
    ExprId(value)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StmtId(pub(crate) usize);

impl IncrementalId for StmtId {
  fn from_usize(value: usize) -> Self {
    StmtId(value)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BlockId(pub(crate) usize);

impl IncrementalId for BlockId {
  fn from_usize(value: usize) -> Self {
    BlockId(value)
  }
}

/// Esto es arena-based allocation
/// por como escalaria en un futuro, es mejor que un Vec<(Expr, Span)>.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ast {
  /// Se debe cumplir el invariante de que los expr_arena, expr_spans esten asociados por el indice ExprId.
  expr_arena: Vec<Expr>,
  expr_spans: Vec<Span>,
  expr_id_gen: IncrementalIdGenerator<ExprId>,
  /// Se debe cumplir el invariante de que los stmt_arena, stmt_spans esten asociados por el indice StmtId.
  stmt_arena: Vec<Stmt>,
  stmt_spans: Vec<Span>,
  stmt_id_gen: IncrementalIdGenerator<StmtId>,
  /// Se debe cumplir el invariante de que los block_arena, block_spans esten asociados por el indice BlockId.
  block_arena: Vec<Block>,
  block_spans: Vec<Span>,
  block_id_gen: IncrementalIdGenerator<BlockId>,
}

impl Ast {
  pub(crate) fn empty() -> Self {
    Self {
      expr_arena: Vec::new(),
      expr_spans: Vec::new(),
      expr_id_gen: IncrementalIdGenerator::new(),
      stmt_arena: Vec::new(),
      stmt_spans: Vec::new(),
      stmt_id_gen: IncrementalIdGenerator::new(),
      block_arena: Vec::new(),
      block_spans: Vec::new(),
      block_id_gen: IncrementalIdGenerator::new(),
    }
  }

  pub(crate) fn expr(&self, id: ExprId) -> Expr {
    self.expr_arena[id.0].clone()
  }

  pub(crate) fn expr_span(&self, id: ExprId) -> Span {
    self.expr_spans[id.0].clone()
  }

  pub(crate) fn add_expr(&mut self, expr: Expr, span: Span) -> ExprId {
    let expr_id = self.expr_id_gen.next_id();
    self.expr_arena.push(expr);
    self.expr_spans.push(span);
    expr_id
  }

  pub(crate) fn update_expr_span(&mut self, id: ExprId, span: Span) -> ExprId {
    self.expr_spans[id.0] = span;
    id
  }

  pub(crate) fn stmt(&self, id: StmtId) -> Stmt {
    self.stmt_arena[id.0].clone()
  }

  pub(crate) fn stmt_span(&self, id: StmtId) -> Span {
    self.stmt_spans[id.0].clone()
  }

  pub(crate) fn add_stmt(&mut self, stmt: Stmt, span: Span) -> StmtId {
    let stmt_id = self.stmt_id_gen.next_id();
    self.stmt_arena.push(stmt);
    self.stmt_spans.push(span);
    stmt_id
  }

  pub(crate) fn block(&self, id: BlockId) -> Block {
    self.block_arena[id.0].clone()
  }

  pub(crate) fn block_span(&self, id: BlockId) -> Span {
    self.block_spans[id.0].clone()
  }

  pub(crate) fn add_block(&mut self, block: Block, span: Span) -> BlockId {
    let block_id = self.block_id_gen.next_id();
    self.block_arena.push(block);
    self.block_spans.push(span);
    block_id
  }

  pub(crate) fn add_block_expr(&mut self, block_id: BlockId) -> ExprId {
    let span = self.block_span(block_id);
    self.add_expr(Expr::Block(block_id), span)
  }
}

#[cfg(test)]
mod tests;
