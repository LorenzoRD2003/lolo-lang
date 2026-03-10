// Traducir AST + informacion semantica a IR.
// el lowering no debe conocer detalles de almacenamiento interno de la IR: de eso se encarga el Builder.
// El lowering debería decir “emiti un add”, no “push a este vec, construi un ValueData, acordate del span, etc.”

use crate::{
  Diagnostic,
  ast::{
    Ast, BinaryExpr, BinaryOp, BlockId as AstBlockId, Expr, ExprId, IfExpr, Program as AstProgram,
    Stmt, StmtId, UnaryExpr, UnaryOp,
  },
  diagnostics::Diagnosable,
  ir::{
    builder::ProgramBuilder,
    error::LoweringError,
    ids::{BlockId, ValueId},
    inst::PhiInput,
    module::IrModule,
    ssa_env::SsaEnv,
    value::IrConstant,
  },
  semantic::{SemanticResult, SymbolId},
};

#[derive(Debug)]
pub(crate) struct LoweringCtx<'a> {
  ast: &'a Ast,
  semantic: &'a SemanticResult,
  builder: ProgramBuilder,
  env: SsaEnv,
  diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> LoweringCtx<'a> {
  /// Punto de entrada:
  /// - crea el ProgramBuilder
  /// - recorre el AST
  /// - bajar declaraciones globales / cuerpo del programa
  /// - Devuelve el IrModule final
  pub(crate) fn lower_to_ir(
    program: &'a AstProgram,
    ast: &'a Ast,
    semantics: &'a SemanticResult,
    diagnostics: &'a mut Vec<Diagnostic>,
  ) -> IrModule {
    let mut ctx = Self::new(program, ast, semantics, diagnostics);
    let main_block = program.main_block(ast);
    let ret = ctx.lower_block(main_block);
    ctx.builder.emit_return(Some(ret));
    ctx.builder.finish()
  }

  // ======================
  //   Metodos auxiliares
  // ======================
  fn new(
    program: &'a AstProgram,
    ast: &'a Ast,
    semantics: &'a SemanticResult,
    diagnostics: &'a mut Vec<Diagnostic>,
  ) -> Self {
    let main_block_expr_id = program.main_block_expr();
    let return_ty = semantics.type_info.type_of_expr(main_block_expr_id);
    let mut ctx = Self {
      ast,
      semantic: semantics,
      builder: ProgramBuilder::new("main".into(), return_ty.into()),
      env: SsaEnv::new(),
      diagnostics,
    };
    if return_ty.is_error() {
      ctx.emit_error(&LoweringError::CannotLowerErrorTypedExpr {
        expr_id: main_block_expr_id,
        span: ctx.ast.expr_span(main_block_expr_id),
      });
    }
    ctx
  }

  fn emit_error(&mut self, err: &LoweringError) {
    self.diagnostics.push(err.to_diagnostic());
  }

  /// Asocia un simbolo fuente con su valor SSA actual.
  fn bind_symbol(&mut self, symbol: SymbolId, value: ValueId) {
    self.env.set(symbol, value)
  }

  /// Baja un bloque del AST a instrucciones IR.
  fn lower_block(&mut self, ast_block_id: AstBlockId) -> ValueId {
    let ast_block = self.ast.block(ast_block_id);
    for stmt_id in ast_block.stmts() {
      match self.ast.stmt(stmt_id) {
        // En AST, Return determina el valor del bloque.
        Stmt::Return(Some(expr_id)) => return self.lower_expr(*expr_id),
        Stmt::Return(None) => return self.builder.emit_const(IrConstant::Unit),
        _ => self.lower_stmt(*stmt_id),
      }
    }
    self.builder.emit_const(IrConstant::Unit)
  }

  /// Baja un statement del AST a instrucciones IR.
  fn lower_stmt(&mut self, stmt_id: StmtId) {
    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding { var, initializer } | Stmt::ConstBinding { var, initializer } => {
        let Some(symbol) = self.lower_lvalue(*var) else {
          return;
        };
        let value = self.lower_expr(*initializer);
        self.bind_symbol(symbol, value);
      }
      Stmt::Assign { var, value_expr } => {
        let Some(symbol) = self.lower_lvalue(*var) else {
          return;
        };
        let value = self.lower_expr(*value_expr);
        self.bind_symbol(symbol, value);
      }
      Stmt::Expr(expr_id) => {
        self.lower_expr(*expr_id);
      }
      Stmt::Print(expr_id) => {
        let value = self.lower_expr(*expr_id);
        self.builder.emit_print(value);
      }
      // lower_block consume Return y lo interpreta como valor del bloque.
      Stmt::Return(_) => {}
    }
  }

  /// Evalua una expresion y devuelve el ValueId resultante.
  fn lower_expr(&mut self, expr_id: ExprId) -> ValueId {
    match self.ast.expr(expr_id) {
      Expr::Var(_) => {
        let Some(symbol) = self.semantic.resolution_info.symbol_of(expr_id) else {
          self.emit_error(&LoweringError::MissingSymbol {
            expr_id,
            span: self.ast.expr_span(expr_id),
          });
          // Recuperacion minima para mantener el lowering en marcha.
          return self.builder.emit_const(IrConstant::Unit);
        };
        match self.env.get(symbol) {
          Some(value_id) => value_id,
          None => {
            self.emit_error(&LoweringError::MissingSsaValueForSymbol {
              symbol,
              span: self.ast.expr_span(expr_id),
            });
            // Recuperacion minima para mantener el lowering en marcha.
            self.builder.emit_const(IrConstant::Unit)
          }
        }
      }
      Expr::Const(c) => self.builder.emit_const(c.into()),
      Expr::Unary(UnaryExpr { op, operand }) => self.lower_unary(*op, *operand),
      Expr::Binary(BinaryExpr { op, lhs, rhs }) => self.lower_binary(*op, *lhs, *rhs),
      Expr::Block(bid) => self.lower_block(*bid),
      Expr::If(IfExpr {
        condition,
        if_block,
        else_branch,
      }) => self.lower_if(expr_id, *condition, *if_block, *else_branch),
    }
  }

  /// Baja una expresion que aparece del lado izquierdo de una asignacion.
  /// Develve el SymbolId que se va a (re)definir.
  fn lower_lvalue(&mut self, expr_id: ExprId) -> Option<SymbolId> {
    match self.ast.expr(expr_id) {
      Expr::Var(_) => {
        let symbol = self.semantic.resolution_info.symbol_of(expr_id);
        if symbol.is_none() {
          self.emit_error(&LoweringError::MissingSymbol {
            expr_id,
            span: self.ast.expr_span(expr_id),
          });
        }
        symbol
      }
      _ => unreachable!(),
    }
  }

  /// Evalua una expresion unaria y devuelve el ValueId resultante.
  fn lower_unary(&mut self, op: UnaryOp, operand: ExprId) -> ValueId {
    let operand_value = self.lower_expr(operand);
    self.builder.emit_unary(op, operand_value)
  }

  /// Evalua una expresion binaria y devuelve el ValueId resultante.
  fn lower_binary(&mut self, op: BinaryOp, lhs: ExprId, rhs: ExprId) -> ValueId {
    let lhs_value = self.lower_expr(lhs);
    let rhs_value = self.lower_expr(rhs);
    self.builder.emit_binary(op, lhs_value, rhs_value)
  }

  /// Convierte una expresion If en una branch de IR (pensarlo como CFG).
  fn lower_if(
    &mut self,
    expr_id: ExprId,
    cond: ExprId,
    ast_if_block: AstBlockId,
    else_branch: Option<ExprId>,
  ) -> ValueId {
    let branch_ty = self.semantic.type_info.type_of_expr(expr_id);
    if branch_ty.is_error() {
      self.emit_error(&LoweringError::CannotLowerErrorTypedExpr {
        expr_id,
        span: self.ast.expr_span(expr_id),
      });
    }
    let has_else_branch = else_branch.is_some();

    let env_before = self.env.clone();
    let cond_value = self.lower_expr(cond);

    let if_block = self.builder.create_block();
    let else_block = self.builder.create_block();
    let merge_block = self.builder.create_block();

    self.builder.emit_branch(cond_value, if_block, else_block);

    // Rama If
    self.env = env_before.clone_for_branch();
    self.builder.switch_to_block(if_block);
    let if_val = self.lower_block(ast_if_block);
    self.builder.emit_jump(merge_block);
    let if_env_out = self.env.clone();

    // Rama Else
    self.env = env_before.clone_for_branch();
    self.builder.switch_to_block(else_block);
    let else_val = else_branch.map(|else_expr| self.lower_expr(else_expr));
    self.builder.emit_jump(merge_block);
    let else_env_out = self.env.clone();

    // Merge
    self.builder.switch_to_block(merge_block);
    self.env = self.merge_envs_with_phis(
      &env_before,
      if_block,
      &if_env_out,
      else_block,
      &else_env_out,
    );

    if has_else_branch {
      self.builder.emit_phi(
        branch_ty.into(),
        vec![
          PhiInput::new(if_block, if_val),
          PhiInput::new(
            else_block,
            else_val.expect("debe existir valor de rama else para construir el phi"),
          ),
        ],
      )
    } else {
      self.builder.emit_const(IrConstant::Unit)
    }
  }

  fn merge_envs_with_phis(
    &mut self,
    env_before: &SsaEnv,
    if_block: BlockId,
    if_env: &SsaEnv,
    else_block: BlockId,
    else_env: &SsaEnv,
  ) -> SsaEnv {
    let mut merged = SsaEnv::new();
    // El merge se hace sobre los simbolos visibles antes del if
    for (symbol, before_val) in env_before.iter().map(|(a, b)| (*a, *b)) {
      let if_val = if_env.get(symbol).unwrap_or(before_val);
      let else_val = else_env.get(symbol).unwrap_or(before_val);
      if if_val == else_val {
        merged.set(symbol, if_val);
      } else {
        let ty = self.builder.get_value_type(if_val);
        let phi_inputs = vec![
          PhiInput::new(if_block, if_val),
          PhiInput::new(else_block, else_val),
        ];
        let phi_val = self.builder.emit_phi(ty, phi_inputs);
        merged.set(symbol, phi_val);
      }
    }
    merged
  }
}

#[cfg(test)]
mod tests;
