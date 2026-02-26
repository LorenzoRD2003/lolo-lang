// El “Semantic Analyzer” (como el parser, pero semantico). Es el archivo principal del modulo `semantic`
// Responsabilidades:
// - Recorre AST / Blocks / Statements / Expressions
// - Gestiona scopes
// - Consulta la symbol table
// - Emite errores semanticos

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryOp, ConstValue, Expr, UnaryOp},
    program::Program,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    category::ExprCategory,
    error::SemanticError,
    scope::ScopeId,
    semantic_info::{SemanticExprInfo, SemanticInfo},
    symbol::SymbolId,
    symbol_table::SymbolTable,
    types::Type,
  },
};

/// Estructura que transforma un AST puro en algo semanticamente enriquecido y chequeado.
#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
  /// El AST. Forma parte del mundo sintactico, asi que si debe ser una referencia y no tomamos ownership.
  /// Vamos a generar mucha metadata para el AST sin tocarlo.
  ast: &'a Ast,
  /// La SymbolTable. Solamente tiene sentido en el contexto de un SemanticAnalyzer, por lo tanto
  /// no es una referencia y tomamos ownership.
  symbol_table: SymbolTable,
  /// Donde se van acumulando los errores encontrados durante el analisis semantico.
  diagnostics: Vec<Diagnostic>,
  /// Informacion semantica que va acumulando el analizador.
  semantic_info: SemanticInfo,
}

impl<'a> SemanticAnalyzer<'a> {
  pub(crate) fn new(ast: &'a Ast, symbol_table: SymbolTable) -> Self {
    let mut analyzer = Self {
      ast,
      symbol_table,
      diagnostics: Vec::new(),
      semantic_info: SemanticInfo::new(),
    };
    analyzer.symbol_table.enter_global_scope();
    analyzer
  }

  /// Obtiene el scope de analisis actual, usando la SymbolTable.
  pub(crate) fn current_scope(&self) -> Option<ScopeId> {
    self.symbol_table.current_scope()
  }

  /// Devuelve una referencia iterable a los diagnosticos de error.
  pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Convierte el error a Diagnostic, lo acumula en la lista de errores.
  fn emit_error(&mut self, err: &SemanticError) {
    self.diagnostics.push(err.to_diagnostic())
  }

  /// Para una expresion, determina su tipo, categoria, simbolo (si es un VarExpr).
  /// Agrega un diagnostic si es `UndefinedVariable`
  pub(crate) fn analyze_expr(&mut self, expr_id: ExprId) {
    // El objetivo es llegar a construir un SemanticExprInfo y agregarlo a expr_info_by_id
    let symbol = self.analyze_expr_symbol(expr_id);
    let r#type = self.analyze_expr_type(expr_id);
    let scope = self
      .current_scope()
      .expect("la expresion no vive en un scope");
    let compile_time_constant = self.analyze_compile_time_constant(expr_id);
    let category = self.analyze_expr_category(expr_id, compile_time_constant.is_some());
    let semantic_expr_info =
      SemanticExprInfo::new(symbol, r#type, category, scope, compile_time_constant);
    self
      .semantic_info
      .insert_expr_info(expr_id, semantic_expr_info);
  }

  fn analyze_expr_symbol(&mut self, expr_id: ExprId) -> Option<SymbolId> {
    let expr = self.ast.expr(expr_id);
    match expr {
      Expr::Var(name) => self.symbol_table.resolve(&name),
      Expr::Const(_) | Expr::Unary(_) | Expr::Binary(_) => None,
    }
  }

  fn analyze_expr_type(&mut self, expr_id: ExprId) -> Type {
    let expr = self.ast.expr(expr_id);
    let span = self.ast.expr_span(expr_id);
    match expr {
      Expr::Const(ConstValue::Int32(_)) => Type::Int32,
      Expr::Const(ConstValue::Bool(_)) => Type::Bool,
      Expr::Var(name) => match self.analyze_expr_symbol(expr_id) {
        Some(symbol_id) => self.symbol_table.symbol(symbol_id).r#type(),
        None => {
          self.emit_error(&SemanticError::UndefinedVariable { name, span });
          Type::DefaultErrorType
        }
      },
      Expr::Unary(unary) => {
        let operand_type = self.analyze_expr_type(unary.operand);
        let expected_operand_type = unary.op.compatible_operand_type();
        if operand_type == expected_operand_type {
          unary.op.result_type()
        } else {
          self.emit_error(&SemanticError::TypeMismatch {
            expected: expected_operand_type,
            found: operand_type,
            span: self.ast.expr_span(unary.operand),
          });
          Type::DefaultErrorType
        }
      }
      Expr::Binary(binary) => {
        let lhs_type = self.analyze_expr_type(binary.lhs);
        let rhs_type = self.analyze_expr_type(binary.rhs);
        let (lhs_expected_type, rhs_expected_type) = binary.op.compatible_operand_types();
        if lhs_type != lhs_expected_type {
          self.emit_error(&SemanticError::TypeMismatch {
            expected: lhs_expected_type,
            found: lhs_type,
            span: self.ast.expr_span(binary.lhs),
          });
        }
        if rhs_type != rhs_expected_type {
          self.emit_error(&SemanticError::TypeMismatch {
            expected: rhs_expected_type,
            found: rhs_type,
            span: self.ast.expr_span(binary.rhs),
          });
        }
        if lhs_type == lhs_expected_type && rhs_type == rhs_expected_type {
          binary.op.result_type()
        } else {
          Type::DefaultErrorType
        }
      }
    }
  }

  fn analyze_expr_category(
    &mut self,
    expr_id: ExprId,
    is_compile_time_constant: bool,
  ) -> ExprCategory {
    let expr = self.ast.expr(expr_id);
    match expr {
      Expr::Const(_) => ExprCategory::constant().with(ExprCategory::value()),
      Expr::Var(_) => ExprCategory::value().with(ExprCategory::place()),
      Expr::Unary(_) | Expr::Binary(_) => {
        let value_category = ExprCategory::value();
        if is_compile_time_constant {
          value_category.with(ExprCategory::constant())
        } else {
          value_category
        }
      }
    }
  }

  fn analyze_compile_time_constant(&mut self, expr_id: ExprId) -> Option<ConstValue> {
    let expr = self.ast.expr(expr_id);
    let span = self.ast.expr_span(expr_id);
    match expr {
      Expr::Const(v) => Some(v),
      Expr::Var(_) => None, // En un futuro cuando implemente `const` esto puede cambiar.
      // Todos los operadores que tenemos son puros asi que esto está bien
      Expr::Unary(unary) => {
        let value = self.analyze_compile_time_constant(unary.operand)?;
        match (value, unary.op) {
          (ConstValue::Int32(x), UnaryOp::Neg) => Some(ConstValue::Int32(-x)),
          (ConstValue::Bool(b), UnaryOp::Not) => Some(ConstValue::Bool(!b)),
          _ => None, // recordar que el type mismatch ya fue analizado antes
        }
      }
      Expr::Binary(binary) => {
        let lvalue = self.analyze_compile_time_constant(binary.lhs)?;
        let rvalue = self.analyze_compile_time_constant(binary.rhs)?;
        match (lvalue, binary.op, rvalue) {
          (ConstValue::Int32(x), BinaryOp::Add, ConstValue::Int32(y)) => match x.checked_add(y) {
            Some(res) => Some(ConstValue::Int32(res)),
            None => {
              self.emit_error(&SemanticError::ArithmeticOverflow {
                span,
                op: BinaryOp::Add,
                lhs: ConstValue::Int32(x),
                rhs: ConstValue::Int32(y),
              });
              None
            }
          },
          (ConstValue::Int32(x), BinaryOp::Sub, ConstValue::Int32(y)) => match x.checked_sub(y) {
            Some(res) => Some(ConstValue::Int32(res)),
            None => {
              self.emit_error(&SemanticError::ArithmeticOverflow {
                span,
                op: BinaryOp::Sub,
                lhs: ConstValue::Int32(x),
                rhs: ConstValue::Int32(y),
              });
              None
            }
          },
          (ConstValue::Int32(x), BinaryOp::Mul, ConstValue::Int32(y)) => match x.checked_mul(y) {
            Some(res) => Some(ConstValue::Int32(res)),
            None => {
              self.emit_error(&SemanticError::ArithmeticOverflow {
                span,
                op: BinaryOp::Mul,
                lhs: ConstValue::Int32(x),
                rhs: ConstValue::Int32(y),
              });
              None
            }
          },
          (ConstValue::Int32(x), BinaryOp::Div, ConstValue::Int32(y)) => {
            if y == 0 {
              let rhs_span = self.ast.expr_span(binary.rhs);
              self.emit_error(&SemanticError::ZeroDivision { span: rhs_span });
              None
            } else {
              Some(ConstValue::Int32(x / y))
            }
          }
          (ConstValue::Int32(x), BinaryOp::Eq, ConstValue::Int32(y)) => {
            Some(ConstValue::Bool(x == y))
          }
          (ConstValue::Int32(x), BinaryOp::Neq, ConstValue::Int32(y)) => {
            Some(ConstValue::Bool(x != y))
          }
          (ConstValue::Int32(x), BinaryOp::Gt, ConstValue::Int32(y)) => {
            Some(ConstValue::Bool(x > y))
          }
          (ConstValue::Int32(x), BinaryOp::Lt, ConstValue::Int32(y)) => {
            Some(ConstValue::Bool(x < y))
          }
          (ConstValue::Int32(x), BinaryOp::Gte, ConstValue::Int32(y)) => {
            Some(ConstValue::Bool(x >= y))
          }
          (ConstValue::Int32(x), BinaryOp::Lte, ConstValue::Int32(y)) => {
            Some(ConstValue::Bool(x <= y))
          }
          (ConstValue::Bool(p), BinaryOp::And, ConstValue::Bool(q)) => {
            Some(ConstValue::Bool(p && q))
          }
          (ConstValue::Bool(p), BinaryOp::Or, ConstValue::Bool(q)) => {
            Some(ConstValue::Bool(p || q))
          }
          (ConstValue::Bool(p), BinaryOp::Xor, ConstValue::Bool(q)) => {
            Some(ConstValue::Bool(p ^ q))
          }
          _ => None,
        }
      }
    }
  }

  /// Analiza un statement, chequeando redeclaraciones (Let)
  /// y asignabilidad de expresiones (Return, Print).
  /// Agrega un diagnostic si es `RedeclaredVariable`, `TypeMismatch`, o `InvalidAssignmentTarget`.
  pub(crate) fn analyze_stmt(&mut self, stmt_id: StmtId) {}

  /// Crea un nuevo scope, analiza statements dentro del bloque, y marca terminador del bloque.
  pub(crate) fn analyze_block(&mut self, block_id: BlockId) {}

  /// Punto de entrada general, itera sobre los bloques del programa
  pub(crate) fn analyze_program(&mut self, program: Program) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    ast::expr::{BinaryExpr, UnaryExpr, VarId},
    semantic::{category, scope::ScopeArena, symbol::Mutability},
  };
  use proptest::prelude::*;

  // Helper para crear SemanticAnalyzer minimal
  fn semantic_analyzer<'a>(ast: &'a Ast) -> SemanticAnalyzer<'a> {
    let scope_arena = ScopeArena::new();
    let symbol_table = SymbolTable::new(scope_arena);
    SemanticAnalyzer::new(ast, symbol_table)
  }

  // Tests basicos por tipo de expresion
  #[test]
  fn analyze_const_expr() {
    let mut ast = Ast::empty();
    let expr_id = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 0..2);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_expr(expr_id);

    let info = sem.semantic_info.expr_info(expr_id);
    assert_eq!(info.symbol(), None);
    assert_eq!(info.r#type(), Type::Int32);
    let category = info.category();
    assert!(category.is_value() && category.is_constant() && !category.is_place());
    assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(42)));
    assert!(sem.diagnostics().is_empty());
  }

  #[test]
  fn analyze_var_expr_resolved() {
    let mut ast = Ast::empty();
    let var_id = VarId("x".into());
    let expr_id = ast.add_expr(Expr::Var(var_id.clone()), 0..1);
    let mut sem = semantic_analyzer(&ast);
    sem.symbol_table.add_symbol(
      &var_id,
      Type::Bool,
      Mutability::Mutable,
      ast.expr_span(expr_id),
    );
    sem.analyze_expr(expr_id);

    let info = sem.semantic_info.expr_info(expr_id);
    assert!(info.symbol().is_some());
    assert_eq!(info.r#type(), Type::Bool);
    let category = info.category();
    assert!(category.is_value() && category.is_place() && !category.is_constant());
    assert!(info.compile_time_constant().is_none());
    assert!(sem.diagnostics().is_empty());
  }

  #[test]
  fn analyze_var_expr_unresolved() {
    let mut ast = Ast::empty();
    let expr_id = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
    let mut sem = semantic_analyzer(&ast);

    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    assert_eq!(info.symbol(), None);
    assert_eq!(info.r#type(), Type::DefaultErrorType);
    assert!(info.compile_time_constant().is_none());
    assert_eq!(sem.diagnostics().len(), 1);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|diag| diag.msg == String::from("variable 'x' indefinida"))
    );
  }

  #[test]
  fn analyze_unary_const_expr() {
    let mut ast = Ast::empty();
    let inner = ast.add_expr(Expr::Const(ConstValue::Int32(5)), 0..1);
    let expr_id = ast.add_expr(
      Expr::Unary(UnaryExpr {
        op: UnaryOp::Neg,
        operand: inner,
      }),
      0..2,
    );
    let mut sem = semantic_analyzer(&ast);

    sem.analyze_expr(inner); // es importante que el analyzer lo haga en orden
    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    assert_eq!(info.r#type(), Type::Int32);
    let category = info.category();
    assert!(category.is_value() && !category.is_place() && category.is_constant());
    assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(-5)));
    assert!(sem.diagnostics().is_empty());
  }

  #[test]
  fn analyze_binary_const_expr() {
    let mut ast = Ast::empty();
    let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 0..1);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 2..3);
    let expr_id = ast.add_expr(
      Expr::Binary(BinaryExpr {
        lhs,
        op: BinaryOp::Add,
        rhs,
      }),
      0..3,
    );

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_expr(lhs);
    sem.analyze_expr(rhs);
    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    let category = info.category();
    assert!(category.is_value() && !category.is_place() && category.is_constant());
    assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(5)));
  }

  #[test]
  fn analyze_expr_scope_is_current_scope() {
    let mut ast = Ast::empty();
    let expr_id = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
    let mut sem = semantic_analyzer(&ast);
    let scope_id = sem.current_scope().unwrap();

    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    assert_eq!(info.scope(), scope_id);
  }

  #[test]
  fn test_zero_division_error() {
    let mut ast = Ast::empty();
    let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 0..1);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(0)), 2..3);
    let expr_id = ast.add_expr(
      Expr::Binary(BinaryExpr {
        lhs,
        rhs,
        op: BinaryOp::Div,
      }),
      0..3,
    );
    let mut sem = semantic_analyzer(&ast);

    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    assert!(info.compile_time_constant().is_none());
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|diag| diag.msg == String::from("division por cero encontrada"))
    );
  }

  #[test]
  fn test_arithmetic_overflow_error() {
    let mut ast = Ast::empty();
    let lhs = ast.add_expr(Expr::Const(ConstValue::Int32(i32::MAX)), 0..1);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
    let expr_id = ast.add_expr(
      Expr::Binary(BinaryExpr {
        lhs,
        rhs,
        op: BinaryOp::Mul,
      }),
      0..3,
    );
    let mut sem = semantic_analyzer(&ast);

    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    assert!(info.compile_time_constant().is_none());
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|diag| diag.msg.contains("overflow"))
    );
  }

  #[test]
  fn test_type_mismatch_error() {
    let mut ast = Ast::empty();
    let lhs = ast.add_expr(Expr::Const(ConstValue::Bool(false)), 0..1);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 2..3);
    let expr_id = ast.add_expr(
      Expr::Binary(BinaryExpr {
        lhs,
        rhs,
        op: BinaryOp::Sub,
      }),
      0..3,
    );
    let mut sem = semantic_analyzer(&ast);

    sem.analyze_expr(expr_id);
    assert_eq!(sem.diagnostics().len(), 2);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg.contains("mismatch de tipos"))
    );
  }
}
