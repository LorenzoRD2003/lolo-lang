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
  pub fn new(ast: &'a Ast, symbol_table: SymbolTable) -> Self {
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
  pub fn current_scope(&self) -> Option<ScopeId> {
    self.symbol_table.current_scope()
  }

  /// Devuelve una referencia iterable a los diagnosticos de error.
  pub fn diagnostics(&self) -> &[Diagnostic] {
    &self.diagnostics
  }

  /// Convierte el error a Diagnostic, lo acumula en la lista de errores.
  fn emit_error(&mut self, err: &SemanticError) {
    self.diagnostics.push(err.to_diagnostic())
  }

  /// Para una expresion, determina su tipo, categoria, simbolo (si es un VarExpr).
  pub fn analyze_expr(&mut self, expr_id: ExprId) {
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
  pub fn analyze_stmt(&mut self, stmt_id: StmtId) {
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
  pub fn analyze_block(&mut self, block_id: BlockId) {
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
  pub fn analyze_program(&mut self, program: Program) {
    self.analyze_block(program.main_block());
  }
}

#[cfg(test)]
mod tests;
