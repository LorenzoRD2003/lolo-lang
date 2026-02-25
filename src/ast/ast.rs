// Responsabilidades del AST: crear nodos, asociarles un Span, y devolver el ExprId
// Usando empty() y add_expr() puedo crear todos los nodos.
// no es exactamente un arbol ahora, sino una arena de nodos (pool)
// es importante remarcar que una Expr NO TIENE Span, sino que el compilador guarda un Span asociado a cada nodo del "arbol"

use crate::{ast::expr::Expr, common::span::Span};

pub(crate) type ExprId = usize;

// por como escalaria en un futuro, es mejor que un Vec<(Expr, Span)>
#[derive(Debug, Clone)]
pub struct Ast {
  exprs: Vec<Expr>,
  spans: Vec<Span>, // indice alineado con exprs. es el ExprId.
}

impl Ast {
  pub(crate) fn empty() -> Self {
    Self {
      exprs: vec![],
      spans: vec![],
    }
  }

  pub(crate) fn expr(&self, id: ExprId) -> Expr {
    self.exprs[id].clone()
  }

  pub(crate) fn span(&self, id: ExprId) -> Span {
    self.spans[id].clone()
  }

  pub(crate) fn add_expr(&mut self, expr: Expr, span: Span) -> ExprId {
    self.exprs.push(expr);
    self.spans.push(span);
    self.exprs.len() - 1
  }

  pub(crate) fn update_span(&mut self, expr_id: ExprId, span: Span) -> ExprId {
    self.spans[expr_id] = span;
    expr_id
  }
}
