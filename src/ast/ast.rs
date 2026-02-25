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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ExprId(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StmtId(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BlockId(pub(crate) usize);

/// Esto es arena-based allocation
/// por como escalaria en un futuro, es mejor que un Vec<(Expr, Span)>.
/// Se debe cumplir el invariante de que los expr_arena, expr_spans esten asociados por el indice ExprId.
/// Se debe cumplir el invariante de que los stmt_arena, stmt_spans esten asociados por el indice StmtId.
/// Se debe cumplir el invariante de que los block_arena, block_spans esten asociados por el indice BlockId.
#[derive(Debug, Clone)]
pub struct Ast {
  expr_arena: Vec<Expr>,
  expr_spans: Vec<Span>,
  stmt_arena: Vec<Stmt>,
  stmt_spans: Vec<Span>,
  block_arena: Vec<Block>,
  block_spans: Vec<Span>,
}

impl Ast {
  pub(crate) fn empty() -> Self {
    Self {
      expr_arena: vec![],
      expr_spans: vec![],
      stmt_arena: vec![],
      stmt_spans: vec![],
      block_arena: vec![],
      block_spans: vec![],
    }
  }

  pub(crate) fn expr(&self, id: ExprId) -> Expr {
    self.expr_arena[id.0].clone()
  }

  pub(crate) fn expr_span(&self, id: ExprId) -> Span {
    self.expr_spans[id.0].clone()
  }

  pub(crate) fn add_expr(&mut self, expr: Expr, span: Span) -> ExprId {
    self.expr_arena.push(expr);
    self.expr_spans.push(span);
    ExprId(self.expr_arena.len() - 1)
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
    self.stmt_arena.push(stmt);
    self.stmt_spans.push(span);
    StmtId(self.stmt_arena.len() - 1)
  }

  pub(crate) fn update_stmt_span(&mut self, id: StmtId, span: Span) -> StmtId {
    self.stmt_spans[id.0] = span;
    id
  }

  pub(crate) fn block(&self, id: BlockId) -> Block {
    self.block_arena[id.0].clone()
  }

  pub(crate) fn block_span(&self, id: BlockId) -> Span {
    self.block_spans[id.0].clone()
  }

  pub(crate) fn add_block(&mut self, block: Block, span: Span) -> BlockId {
    self.block_arena.push(block);
    self.block_spans.push(span);
    BlockId(self.block_arena.len() - 1)
  }

  pub(crate) fn update_block_span(&mut self, id: BlockId, span: Span) -> BlockId {
    self.block_spans[id.0] = span;
    id
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::expr::ConstValue;
  use proptest::prelude::*;

  #[test]
  fn test_add_and_retrieve_expr() {
    let mut ast = Ast::empty();
    let expr = Expr::Const(ConstValue::Int(420));
    let span = 0..3;
    let id = ast.add_expr(expr.clone(), span.clone());
    assert_eq!(ast.expr(id), expr);
    assert_eq!(ast.expr_span(id), span);
  }

  #[test]
  fn test_update_expr_span() {
    let mut ast = Ast::empty();
    let span1 = 0..3;
    let span2 = 1..5;
    let expr = Expr::Const(ConstValue::Int(42));
    let id = ast.add_expr(expr, span1);
    ast.update_expr_span(id, span2.clone());
    assert_eq!(ast.expr_span(id), span2);
  }

  #[test]
  fn test_add_and_retrieve_stmt() {
    let mut ast = Ast::empty();
    let span = 0..4;
    let stmt = Stmt::Return(ExprId(0));
    let id = ast.add_stmt(stmt.clone(), span.clone());
    assert_eq!(ast.stmt(id), stmt);
    assert_eq!(ast.stmt_span(id), span);
  }

  #[test]
  fn test_update_stmt_span() {
    let mut ast = Ast::empty();
    let span1 = 0..4;
    let span2 = 2..6;
    let stmt = Stmt::Return(ExprId(0));
    let id = ast.add_stmt(stmt, span1);
    ast.update_stmt_span(id, span2.clone());
    assert_eq!(ast.stmt_span(id), span2);
  }

  #[test]
  fn test_add_and_retrieve_block() {
    let mut ast = Ast::empty();
    let span = 0..10;
    let block = Block::with_stmts(vec![StmtId(0), StmtId(1)]);
    let id = ast.add_block(block.clone(), span.clone());
    assert_eq!(ast.block(id), block.clone());
    assert_eq!(ast.block_span(id), span);
  }

  #[test]
  fn test_update_block_span() {
    let mut ast = Ast::empty();
    let span1 = 0..10;
    let span2 = 3..15;
    let block = Block::with_stmts(vec![StmtId(0), StmtId(1)]);
    let id = ast.add_block(block, span1);
    ast.update_block_span(id, span2.clone());
    assert_eq!(ast.block_span(id), span2);
  }

  #[test]
  fn test_multiple_exprs_stmts_blocks() {
    let mut ast = Ast::empty();

    let e1 = ast.add_expr(Expr::Const(ConstValue::Int(1)), 0..1);
    let e2 = ast.add_expr(Expr::Const(ConstValue::Int(2)), 2..3);
    assert_eq!(ast.expr(e1), Expr::Const(ConstValue::Int(1)));
    assert_eq!(ast.expr(e2), Expr::Const(ConstValue::Int(2)));

    let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
    let s2 = ast.add_stmt(Stmt::Expr(e2), 2..3);
    assert_eq!(ast.stmt(s1), Stmt::Expr(e1));
    assert_eq!(ast.stmt(s2), Stmt::Expr(e2));

    let b1 = ast.add_block(Block::with_stmts(vec![s1, s2]), 0..3);
    assert_eq!(ast.block(b1).stmts(), vec![s1, s2]);
  }

  #[test]
  #[should_panic]
  fn test_out_of_bounds_expr() {
    let ast = Ast::empty();
    let _ = ast.expr(ExprId(0));
  }

  #[test]
  #[should_panic]
  fn test_out_of_bounds_stmt() {
    let ast = Ast::empty();
    let _ = ast.stmt(StmtId(0));
  }

  #[test]
  #[should_panic]
  fn test_out_of_bounds_block() {
    let ast = Ast::empty();
    let _ = ast.block(BlockId(0));
  }

  #[test]
  fn all_stmt_expr_ids_are_valid() {
    let mut ast = Ast::empty();
    let e1 = ast.add_expr(Expr::Const(ConstValue::Int(1)), 0..1);
    let e2 = ast.add_expr(Expr::Const(ConstValue::Int(2)), 2..3);
    let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
    let s2 = ast.add_stmt(Stmt::Return(e2), 2..3);

    // Verifico que todos los ExprId referenciados por Stmt existen
    for stmt_id in &[s1, s2] {
      match ast.stmt(*stmt_id) {
        Stmt::Expr(eid) | Stmt::Return(eid) | Stmt::Print(eid) => {
          let _ = ast.expr(eid); // si no existe -> panic
        }
        _ => {}
      }
    }
  }

  #[test]
  fn all_block_stmt_ids_are_valid() {
    let mut ast = Ast::empty();
    let e1 = ast.add_expr(Expr::Const(ConstValue::Int(1)), 0..1);
    let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
    let s2 = ast.add_stmt(Stmt::Return(e1), 2..3);

    let block = ast.add_block(Block::with_stmts(vec![s1, s2]), 0..3);
    // Cada StmtId dentro del Block debe existir, sino panickea
    for stmt_id in &ast.block(block).stmts() {
      let _ = ast.stmt(*stmt_id);
    }
  }

  #[test]
  fn all_expr_ids_in_nested_blocks_are_valid() {
    let mut ast = Ast::empty();
    let e1 = ast.add_expr(Expr::Const(ConstValue::Int(1)), 0..1);
    let e2 = ast.add_expr(Expr::Const(ConstValue::Int(2)), 2..3);
    let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
    let s2 = ast.add_stmt(Stmt::Return(e2), 2..3);
    let inner_block = ast.add_block(Block::with_stmts(vec![s1]), 0..1);
    let outer_block = ast.add_block(Block::with_stmts(vec![s2]), 2..3);

    // Chequeo recursivo: todos los ExprId referenciados existen
    for block_id in &[inner_block, outer_block] {
      for stmt_id in &ast.block(*block_id).stmts() {
        match ast.stmt(*stmt_id) {
          Stmt::Expr(eid) | Stmt::Return(eid) | Stmt::Print(eid) => {
            let _ = ast.expr(eid);
          }
          _ => {}
        }
      }
    }
  }

  proptest! {
    #[test]
    fn expr_span_indices_proptest(exprs in 0usize..100, start in 0usize..50, end in 51usize..100) {
      let mut ast = Ast::empty();
      for i in 0..exprs {
        let id = ast.add_expr(Expr::Const(ConstValue::Int(i as i32)), start..end);
        let _ = ast.expr(id);
        let _ = ast.expr_span(id);
      }
      prop_assert_eq!(ast.expr_arena.len(), ast.expr_spans.len());
    }

    #[test]
    fn stmt_span_indices_proptest(stmts in 0usize..100, start in 0usize..50, end in 51usize..100) {
      let mut ast = Ast::empty();
      for i in 0..stmts {
        let id = ast.add_stmt(Stmt::Return(ExprId(i)), start..end);
        let _ = ast.stmt(id);
        let _ = ast.stmt_span(id);
      }
      prop_assert_eq!(ast.stmt_arena.len(), ast.stmt_spans.len());
    }

    #[test]
    fn block_span_indices_proptest(blocks in 0usize..50, stmts_per_block in 0usize..10, start in 0usize..50, end in 51usize..100) {
      let mut ast = Ast::empty();
      let mut stmt_ids = vec![];
      for i in 0..blocks * stmts_per_block {
        stmt_ids.push(ast.add_stmt(Stmt::Return(ExprId(i)), start..end));
      }
      for i in 0..blocks {
        let block_stmt_ids = stmt_ids[i*stmts_per_block..(i+1)*stmts_per_block].to_vec();
        let id = ast.add_block(Block::with_stmts(block_stmt_ids), start..end);
        let _ = ast.block(id);
        let _ = ast.block_span(id);
      }
      prop_assert_eq!(ast.stmt_arena.len(), ast.stmt_spans.len());
      prop_assert_eq!(ast.block_arena.len(), ast.block_spans.len());
    }
  }
}
