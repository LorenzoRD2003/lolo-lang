// El “Semantic Analyzer” (como el parser, pero semantico). Es el archivo principal del modulo `semantic`
// Responsabilidades:
// - Recorre AST / Blocks / Statements / Expressions
// - Gestiona scopes
// - Consulta la symbol table
// - Emite errores semanticos

use crate::{
  ast::{
    ast::{Ast, BlockId, ExprId, StmtId},
    expr::{BinaryOp, ConstValue, Expr, UnaryOp, VarId},
    program::Program,
    stmt::Stmt,
  },
  diagnostics::diagnostic::{Diagnosable, Diagnostic},
  semantic::{
    category::ExprCategory,
    error::SemanticError,
    scope::ScopeId,
    semantic_info::{SemanticBlockInfo, SemanticExprInfo, SemanticInfo, SemanticStmtInfo},
    symbol::{Mutability, SymbolId},
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
  pub(crate) fn analyze_expr(&mut self, expr_id: ExprId) {
    // El objetivo es llegar a construir un SemanticExprInfo y agregarlo a expr_info_by_id
    // Primero analizamos las expresiones que componen a la principal
    let expr = self.ast.expr(expr_id);
    match expr {
      Expr::Unary(unary) => {
        self.analyze_expr(unary.operand);
      }
      Expr::Binary(binary) => {
        self.analyze_expr(binary.lhs);
        self.analyze_expr(binary.rhs);
      }
      _ => {}
    };
    let symbol = self.analyze_expr_symbol(expr_id);
    let r#type = self.analyze_expr_type(expr_id);
    let scope = self
      .current_scope()
      .expect("la expresion debe vivir en un scope");
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
      Expr::Var(name) => match self.symbol_table.resolve(&name) {
        Some(symbol_id) => self.symbol_table.symbol(symbol_id).r#type(),
        None => {
          self.emit_error(&SemanticError::UndefinedVariable { name, span });
          Type::DefaultErrorType
        }
      },
      Expr::Unary(unary) => {
        let operand_type = self.semantic_info.expr_info(unary.operand).r#type();
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
        let lhs_type = self.semantic_info.expr_info(binary.lhs).r#type();
        let rhs_type = self.semantic_info.expr_info(binary.rhs).r#type();
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
        let value = self
          .semantic_info
          .expr_info(unary.operand)
          .compile_time_constant()?;
        match (value, unary.op) {
          (ConstValue::Int32(x), UnaryOp::Neg) => Some(ConstValue::Int32(-x)),
          (ConstValue::Bool(b), UnaryOp::Not) => Some(ConstValue::Bool(!b)),
          _ => None, // recordar que el type mismatch ya fue analizado antes
        }
      }
      Expr::Binary(binary) => {
        let lvalue = self
          .semantic_info
          .expr_info(binary.lhs)
          .compile_time_constant()?
          .clone();
        let rvalue = self
          .semantic_info
          .expr_info(binary.rhs)
          .compile_time_constant()?
          .clone();
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
  pub(crate) fn analyze_stmt(&mut self, stmt_id: StmtId) {
    let scope = self
      .current_scope()
      .expect("el statement debe vivir en un scope");

    let stmt = self.ast.stmt(stmt_id);
    let mut symbol_declared: Option<SymbolId> = None;
    match stmt {
      // aca en un futuro cuando haya funciones y tipos de retorno, habria que verificar la compatibilidad de tipos
      Stmt::Expr(expr_id) | Stmt::Return(expr_id) | Stmt::Print(expr_id) => {
        self.analyze_expr(expr_id)
      }
      Stmt::Let { var, initializer } => {
        // Ahora es seguro asumir que var es una variable
        match self.ast.expr(var) {
          Expr::Var(name) => {
            // Analizamos el inicializador
            self.analyze_expr(initializer);
            let initializer_info = self.semantic_info.expr_info(initializer);
            let (initializer_type, initializer_category) =
              (initializer_info.r#type(), initializer_info.category());
            // Chequeo de que el RHS sea ValueExpr
            if !initializer_category.is_value() {
              self.emit_error(&SemanticError::ExpectedValueExpression {
                span: self.ast.expr_span(initializer),
              })
            };
            // Chequeo de redeclaracion en el mismo scope
            if let Some(previous_symbol) = self.symbol_table.was_declared_in_current_scope(&name) {
              let previous_declaration_stmt_id = self
                .get_stmt_for_symbol_declaration(previous_symbol)
                .expect("debe existir una declaracion anterior");
              self.emit_error(&SemanticError::RedeclaredVariable {
                name,
                span: self.ast.stmt_span(stmt_id),
                previous_span: self.ast.stmt_span(previous_declaration_stmt_id),
              });
            } else {
              symbol_declared = Some(self.symbol_table.add_symbol(
                &name,
                initializer_type,
                Mutability::Immutable,
                self.ast.expr_span(var),
              ));
            }
          }
          _ => {
            // Chequeo de que sea PlaceExpr, no intentamos agregar un simbolo si el LHS no es una variable
            self.emit_error(&SemanticError::ExpectedPlaceExpression {
              span: self.ast.expr_span(var),
            });
          }
        };
      }
      Stmt::If {
        condition,
        if_block,
      } => {
        self.verify_condition_is_boolean(condition);
        self.analyze_block(if_block);
      }
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      } => {
        self.verify_condition_is_boolean(condition);
        self.analyze_block(if_block);
        self.analyze_block(else_block);
      }
    };
    let semantic_stmt_info = SemanticStmtInfo::new(scope, symbol_declared);
    self
      .semantic_info
      .insert_stmt_info(stmt_id, semantic_stmt_info);
  }

  fn get_stmt_for_symbol_declaration(&self, symbol: SymbolId) -> Option<StmtId> {
    self
      .semantic_info
      .stmt_info_by_id()
      .iter()
      .find_map(|(stmt_id, info)| (info.symbol_declared() == Some(symbol)).then_some(*stmt_id))
  }

  /// Verifica que una condicion dada es booleana (Type::Bool).
  fn verify_condition_is_boolean(&mut self, condition: ExprId) {
    self.analyze_expr(condition);
    let condition_type = self.semantic_info.expr_info(condition).r#type();
    if condition_type != Type::Bool {
      self.emit_error(&SemanticError::TypeMismatch {
        expected: Type::Bool,
        found: condition_type,
        span: self.ast.expr_span(condition),
      });
    }
  }

  /// Crea un nuevo scope, analiza statements dentro del bloque, y marca terminador del bloque.
  pub(crate) fn analyze_block(&mut self, block_id: BlockId) {
    // El chiste es que mientras dure esta funcion, todo se va a analizar para el scope de este bloque
    let block_scope = self.symbol_table.enter_scope();
    let block = self.ast.block(block_id);
    for stmt_id in block.stmts() {
      self.analyze_stmt(*stmt_id);
    }
    let terminator = block.stmts().last().copied();
    let semantic_block_info = SemanticBlockInfo::new(block_scope, terminator);
    self
      .semantic_info
      .insert_block_info(block_id, semantic_block_info);
    self.symbol_table.exit_scope();
  }

  /// Punto de entrada general, analiza el bloque principal del programa
  pub(crate) fn analyze_program(&mut self, program: Program) {
    self.analyze_block(program.main_block());
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    ast::{
      expr::{BinaryExpr, UnaryExpr, VarId},
      stmt::Block,
    },
    semantic::{scope::ScopeArena, symbol::Mutability},
  };

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
      Mutability::Immutable,
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
    sem.analyze_expr(expr_id);
    let info = sem.semantic_info.expr_info(expr_id);
    let category = info.category();
    assert!(category.is_value() && !category.is_place() && category.is_constant());
    assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(5)));
  }

  // Tests de scope
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

  // Tests de errores
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
        .any(|diag| diag.msg() == String::from("variable 'x' indefinida"))
    );
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
        .any(|diag| diag.msg() == String::from("division por cero encontrada"))
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
        .any(|diag| diag.msg().contains("overflow"))
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
        .any(|d| d.msg().contains("mismatch de tipos"))
    );
  }

  #[test]
  fn let_declares_symbol_in_scope() {
    // let x = 42;
    let mut ast = Ast::empty();
    let var = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
    let init = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 8..10);
    let stmt = ast.add_stmt(
      Stmt::Let {
        var,
        initializer: init,
      },
      0..11,
    );
    let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..11);
    let program = Program::new(block, 0..11);

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    let stmt_info = sem.semantic_info.stmt_info(stmt);
    assert!(stmt_info.symbol_declared().is_some());
    assert!(sem.diagnostics().is_empty());
  }

  // TODO: Este test cuando tenga alguna expresion que no tenga la categoria ValueExpr.
  // #[test]
  // fn let_with_non_value_expr_initializer() {
  // }

  #[test]
  fn let_with_var_non_place_expr() {
    // let x + 42 = 3;
    let mut ast = Ast::empty();
    let lhs = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 8..10);
    let sum_var = ast.add_expr(
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Add,
        lhs,
        rhs,
      }),
      6..7,
    );
    let initializer = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 13..14);
    let stmt = ast.add_stmt(
      Stmt::Let {
        var: sum_var,
        initializer,
      },
      0..15,
    );
    let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..15);
    let program = Program::new(block, 0..15);

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    let stmt_info = sem.semantic_info.stmt_info(stmt);
    assert!(stmt_info.symbol_declared().is_none());
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("place expression"))
    );
  }

  #[test]
  fn redeclaration_in_same_scope_is_error() {
    // let x = 1;
    // let x = 2;
    let mut ast = Ast::empty();
    let var1 = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
    let init1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
    let stmt1 = ast.add_stmt(
      Stmt::Let {
        var: var1,
        initializer: init1,
      },
      0..5,
    );

    let var2 = ast.add_expr(Expr::Var(VarId("x".into())), 6..7);
    let init2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 10..11);
    let stmt2 = ast.add_stmt(
      Stmt::Let {
        var: var2,
        initializer: init2,
      },
      6..11,
    );
    let block = ast.add_block(Block::with_stmts(vec![stmt1, stmt2]), 0..11);
    let program = Program::new(block, 0..11);

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);

    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("la variable 'x' ya fue declarada"))
    );
  }

  #[test]
  fn shadowing_in_inner_block_is_allowed() {
    // let x = 1;
    // if x <= 3 { let x = 2; }
    let mut ast = Ast::empty();
    let var_outer = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
    let init_outer = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 8..9);
    let stmt_outer = ast.add_stmt(
      Stmt::Let {
        var: var_outer,
        initializer: init_outer,
      },
      0..10,
    );

    let condition_lhs = ast.add_expr(Expr::Var(VarId("x".into())), 13..14);
    let condition_rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 18..19);
    let condition = ast.add_expr(
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Lte,
        lhs: condition_lhs,
        rhs: condition_rhs,
      }),
      13..19,
    );

    let var_inner = ast.add_expr(Expr::Var(VarId("x".into())), 26..27);
    let init_inner = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 30..31);
    let stmt_inner = ast.add_stmt(
      Stmt::Let {
        var: var_inner,
        initializer: init_inner,
      },
      22..32,
    );
    let if_block = ast.add_block(Block::with_stmts(vec![stmt_inner]), 20..34);
    let stmt_if = ast.add_stmt(
      Stmt::If {
        condition,
        if_block,
      },
      10..19,
    );

    let main_block = ast.add_block(Block::with_stmts(vec![stmt_outer, stmt_if]), 0..34);
    let program = Program::new(main_block, 0..14);

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);

    // No hay errores semanticos
    assert!(sem.diagnostics().is_empty());
    // Ambos lets deben haber declarado un simbolo
    let outer_info = sem.semantic_info.stmt_info(stmt_outer);
    let inner_info = sem.semantic_info.stmt_info(stmt_inner);
    let outer_symbol = outer_info
      .symbol_declared()
      .expect("outer let debe declarar símbolo");
    let inner_symbol = inner_info
      .symbol_declared()
      .expect("inner let debe declarar símbolo");
    // Los simbolos deben ser distintos (shadowing real)
    assert_ne!(outer_symbol, inner_symbol);
    // La condición del if debe referirse al símbolo externo
    dbg!(10);
    let condition_lhs_info = sem.semantic_info.expr_info(condition_lhs);
    let resolved_symbol = condition_lhs_info
      .symbol()
      .expect("la x en la condición debe resolver");
    assert_eq!(resolved_symbol, outer_symbol);
  }

  // If con condicion no booleana
  #[test]
  fn if_condition_must_be_bool() {
    let mut ast = Ast::empty();
    let condition = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
    let if_block = ast.add_block(Block::new(), 3..4);
    let stmt = ast.add_stmt(
      Stmt::If {
        condition,
        if_block,
      },
      0..2,
    );
    let outer_block = ast.add_block(Block::with_stmts(vec![stmt]), 0..4);
    let program = Program::new(outer_block, 0..4);

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("mismatch de tipos"))
    );
  }

  #[test]
  fn inner_block_variable_not_visible_outside() {
    // if true { let x = 1; }
    // return x;
    let mut ast = Ast::empty();
    let var_inner = ast.add_expr(Expr::Var(VarId("x".into())), 14..15);
    let init_inner = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 18..19);
    let stmt_inner = ast.add_stmt(
      Stmt::Let {
        var: var_inner,
        initializer: init_inner,
      },
      10..20,
    );
    let if_block = ast.add_block(Block::with_stmts(vec![stmt_inner]), 8..22);
    let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 3..7);
    let stmt_if = ast.add_stmt(
      Stmt::If {
        condition,
        if_block,
      },
      0..2,
    );

    let use_x = ast.add_expr(Expr::Var(VarId("x".into())), 29..30);
    let stmt_use = ast.add_stmt(Stmt::Return(use_x), 22..31);
    let main_block = ast.add_block(Block::with_stmts(vec![stmt_if, stmt_use]), 0..31);
    let program = Program::new(main_block, 0..31);

    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("variable 'x' indefinida"))
    );
  }

  #[test]
  fn block_terminator_is_last_statement() {
    let mut ast = Ast::empty();
    let e1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
    let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
    let e2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
    let s2 = ast.add_stmt(Stmt::Expr(e2), 2..3);
    let block = ast.add_block(Block::with_stmts(vec![s1, s2]), 0..3);
    let program = Program::new(block, 0..3);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    let block_info = sem.semantic_info.block_info(block);
    assert_eq!(block_info.terminator(), Some(s2));
  }

  #[test]
  fn analyze_program_analyzes_main_block() {
    let mut ast = Ast::empty();
    let expr = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 0..1);
    let stmt = ast.add_stmt(Stmt::Expr(expr), 0..1);
    let main_block = ast.add_block(Block::with_stmts(vec![stmt]), 0..1);
    let program = Program::new(main_block, 0..1);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert_eq!(
      sem.semantic_info.block_info(main_block).terminator(),
      Some(stmt)
    );
  }

  #[test]
  fn let_cannot_use_variable_in_its_own_initializer() {
    // let x = x + 1;
    let mut ast = Ast::empty();
    let var = ast.add_expr(Expr::Var(VarId("x".into())), 4..5);
    let lhs = ast.add_expr(Expr::Var(VarId("x".into())), 8..9);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 12..13);
    let init = ast.add_expr(
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Add,
        lhs,
        rhs,
      }),
      8..13,
    );
    let stmt = ast.add_stmt(
      Stmt::Let {
        var,
        initializer: init,
      },
      0..14,
    );
    let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..14);
    let program = Program::new(block, 0..14);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("variable 'x' indefinida"))
    );
  }

  #[test]
  fn if_condition_with_unresolved_variable() {
    // if y { }
    let mut ast = Ast::empty();
    let condition = ast.add_expr(Expr::Var(VarId("xyz".into())), 0..1);
    let if_block = ast.add_block(Block::new(), 3..4);
    let stmt = ast.add_stmt(
      Stmt::If {
        condition,
        if_block,
      },
      0..4,
    );
    let outer = ast.add_block(Block::with_stmts(vec![stmt]), 0..4);
    let program = Program::new(outer, 0..4);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("variable 'xyz' indefinida"))
    );
  }

  #[test]
  fn binary_with_error_operand_is_not_constant_folded() {
    // x + 3 donde x no existe
    let mut ast = Ast::empty();
    let lhs = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
    let rhs = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 2..3);
    let expr = ast.add_expr(
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Add,
        lhs,
        rhs,
      }),
      0..3,
    );
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_expr(expr);
    let info = sem.semantic_info.expr_info(expr);
    assert!(info.compile_time_constant().is_none());
    assert!(!sem.diagnostics().is_empty());
  }

  #[test]
  fn triple_nested_shadowing() {
    // let x = 1;
    // if x == 1 {
    //   let x = 2;
    //   if x == 2 {
    //     let x = 3;
    //   }
    // }
    let mut ast = Ast::empty();
    let v1 = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
    let i1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
    let s1 = ast.add_stmt(
      Stmt::Let {
        var: v1,
        initializer: i1,
      },
      0..5,
    );
    let v2 = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
    let i2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 14..15);
    let s2 = ast.add_stmt(
      Stmt::Let {
        var: v2,
        initializer: i2,
      },
      10..15,
    );
    let v3 = ast.add_expr(Expr::Var(VarId("x".into())), 20..21);
    let i3 = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 24..25);
    let s3 = ast.add_stmt(
      Stmt::Let {
        var: v3,
        initializer: i3,
      },
      20..25,
    );

    let inner_inner_block = ast.add_block(Block::with_stmts(vec![s3]), 18..27);
    let inner_condition_lhs = ast.add_expr(Expr::Var(VarId("x".into())), 16..17);
    let inner_condition_rhs = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 16..17);
    let inner_condition = ast.add_expr(
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Eq,
        lhs: inner_condition_lhs,
        rhs: inner_condition_rhs,
      }),
      16..17,
    );
    let inner_if = ast.add_stmt(
      Stmt::If {
        condition: inner_condition,
        if_block: inner_inner_block,
      },
      16..27,
    );

    let inner_block = ast.add_block(Block::with_stmts(vec![s2, inner_if]), 8..27);
    let outer_condition_lhs = ast.add_expr(Expr::Var(VarId("x".into())), 6..17);
    let outer_condition_rhs = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 16..7);
    let outer_condition = ast.add_expr(
      Expr::Binary(BinaryExpr {
        op: BinaryOp::Eq,
        lhs: outer_condition_lhs,
        rhs: outer_condition_rhs,
      }),
      6..7,
    );
    let outer_if = ast.add_stmt(
      Stmt::If {
        condition: outer_condition,
        if_block: inner_block,
      },
      6..27,
    );
    let main_block = ast.add_block(Block::with_stmts(vec![s1, outer_if]), 0..27);
    let program = Program::new(main_block, 0..27);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(sem.diagnostics().is_empty());
  }

  #[test]
  fn redeclaration_after_inner_scope_is_error() {
    // let x = 1;
    // if true { let x = 2; }
    // let x = 3;  // error
    let mut ast = Ast::empty();
    let v1 = ast.add_expr(Expr::Var(VarId("x".into())), 0..1);
    let i1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 4..5);
    let s1 = ast.add_stmt(
      Stmt::Let {
        var: v1,
        initializer: i1,
      },
      0..5,
    );
    let v2 = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
    let i2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 14..15);
    let s2 = ast.add_stmt(
      Stmt::Let {
        var: v2,
        initializer: i2,
      },
      10..15,
    );
    let inner_block = ast.add_block(Block::with_stmts(vec![s2]), 8..17);
    let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 6..7);
    let if_stmt = ast.add_stmt(
      Stmt::If {
        condition: condition,
        if_block: inner_block,
      },
      6..17,
    );
    let v3 = ast.add_expr(Expr::Var(VarId("x".into())), 18..19);
    let i3 = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 22..23);
    let s3 = ast.add_stmt(
      Stmt::Let {
        var: v3,
        initializer: i3,
      },
      18..23,
    );
    let main_block = ast.add_block(Block::with_stmts(vec![s1, if_stmt, s3]), 0..23);
    let program = Program::new(main_block, 0..23);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("la variable 'x' ya fue declarada"))
    );
  }

  #[test]
  fn if_else_scopes_are_independent() {
    // if true { let x = 1; }
    // else { let x = 2; }
    let mut ast = Ast::empty();
    let v_if = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
    let i_if = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 14..15);
    let s_if = ast.add_stmt(
      Stmt::Let {
        var: v_if,
        initializer: i_if,
      },
      10..15,
    );
    let v_else = ast.add_expr(Expr::Var(VarId("x".into())), 25..26);
    let i_else = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 29..30);
    let s_else = ast.add_stmt(
      Stmt::Let {
        var: v_else,
        initializer: i_else,
      },
      25..30,
    );
    let if_block = ast.add_block(Block::with_stmts(vec![s_if]), 8..17);
    let else_block = ast.add_block(Block::with_stmts(vec![s_else]), 23..32);
    let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 3..7);
    let stmt = ast.add_stmt(
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      },
      0..32,
    );
    let main = ast.add_block(Block::with_stmts(vec![stmt]), 0..32);
    let program = Program::new(main, 0..32);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(sem.diagnostics().is_empty());
  }

  #[test]
  fn variable_declared_in_if_not_visible_in_else() {
    // if true { let x = 1; }
    // else { print(x); }
    let mut ast = Ast::empty();
    let v_if = ast.add_expr(Expr::Var(VarId("x".into())), 10..11);
    let i_if = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 14..15);
    let s_if = ast.add_stmt(
      Stmt::Let {
        var: v_if,
        initializer: i_if,
      },
      10..15,
    );
    let use_x = ast.add_expr(Expr::Var(VarId("x".into())), 30..31);
    let s_else = ast.add_stmt(Stmt::Print(use_x), 24..32);
    let if_block = ast.add_block(Block::with_stmts(vec![s_if]), 8..17);
    let else_block = ast.add_block(Block::with_stmts(vec![s_else]), 22..34);
    let condition = ast.add_expr(Expr::Const(ConstValue::Bool(true)), 3..7);
    let stmt = ast.add_stmt(
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      },
      0..34,
    );
    let main = ast.add_block(Block::with_stmts(vec![stmt]), 0..34);
    let program = Program::new(main, 0..34);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("variable 'x' indefinida"))
    );
  }

  #[test]
  fn if_else_with_invalid_condition_still_analyzes_blocks() {
    // if 1 { let x = 2; } else { let y = 3; }
    let mut ast = Ast::empty();
    let v_if = ast.add_expr(Expr::Var(VarId("x".into())), 8..9);
    let i_if = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 12..13);
    let s_if = ast.add_stmt(
      Stmt::Let {
        var: v_if,
        initializer: i_if,
      },
      8..13,
    );
    let v_else = ast.add_expr(Expr::Var(VarId("y".into())), 22..23);
    let i_else = ast.add_expr(Expr::Const(ConstValue::Int32(3)), 26..27);
    let s_else = ast.add_stmt(
      Stmt::Let {
        var: v_else,
        initializer: i_else,
      },
      22..27,
    );
    let condition = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 3..4);
    let if_block = ast.add_block(Block::with_stmts(vec![s_if]), 6..15);
    let else_block = ast.add_block(Block::with_stmts(vec![s_else]), 20..30);
    let if_stmt = ast.add_stmt(
      Stmt::IfElse {
        condition,
        if_block,
        else_block,
      },
      0..30,
    );
    let main = ast.add_block(Block::with_stmts(vec![if_stmt]), 0..30);
    let program = Program::new(main, 0..30);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);

    // Debe haber error de tipo en la condición
    assert!(
      sem
        .diagnostics()
        .iter()
        .any(|d| d.msg().contains("mismatch de tipos"))
    );

    // Pero igual deben declararse x e y
    let if_info = sem.semantic_info.stmt_info(s_if);
    let else_info = sem.semantic_info.stmt_info(s_else);
    assert!(if_info.symbol_declared().is_some());
    assert!(else_info.symbol_declared().is_some());
  }

  // #[test]
  // fn print_requires_value_expression() {
  //   // Esto para cuando agreguemos alguna expresion que no sea ValueExpr
  // }

  #[test]
  fn print_constant_keeps_constant_info() {
    // print(42);
    let mut ast = Ast::empty();
    let expr = ast.add_expr(Expr::Const(ConstValue::Int32(42)), 6..8);
    let stmt = ast.add_stmt(Stmt::Print(expr), 0..8);
    let block = ast.add_block(Block::with_stmts(vec![stmt]), 0..8);
    let program = Program::new(block, 0..8);
    let mut sem = semantic_analyzer(&ast);
    sem.analyze_program(program);
    let info = sem.semantic_info.expr_info(expr);
    assert_eq!(info.compile_time_constant(), Some(&ConstValue::Int32(42)));
    assert!(sem.diagnostics().is_empty());
  }
}
