// Traducir AST + informacion semantica a IR.
// el lowering no debe conocer detalles de almacenamiento interno de la IR: de eso se encarga el Builder.
// El lowering debería decir “emiti un add”, no “push a este vec, construi un ValueData, acordate del span, etc.”

use crate::{
  Diagnostic,
  ast::{
    Ast, BinaryExpr, BinaryOp, BlockId as AstBlockId, Expr, ExprId, IfExpr, Program as AstProgram,
    Stmt, StmtId, UnaryExpr, UnaryOp,
  },
  ir::{
    builder::ProgramBuilder,
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
  #[allow(dead_code)]
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
    ctx.lower_block(main_block);
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
    let return_ty = semantics.type_info.type_of_expr(program.main_block_expr());
    Self {
      ast,
      semantic: semantics,
      builder: ProgramBuilder::new("main".into(), return_ty.into()),
      env: SsaEnv::new(),
      diagnostics,
    }
  }

  /// Asocia un simbolo fuente con su valor SSA actual.
  fn bind_symbol(&mut self, symbol: SymbolId, value: ValueId) {
    self.env.set(symbol, value)
  }

  /// Baja un bloque del AST a instrucciones IR.
  fn lower_block(&mut self, ast_block_id: AstBlockId) -> ValueId {
    let ast_block = self.ast.block(ast_block_id);
    for stmt_id in ast_block.stmts() {
      self.lower_stmt(*stmt_id);
    }

    if let Some(tail_expr) = ast_block.tail_expr() {
      self.lower_expr(tail_expr)
    } else {
      self.builder.emit_const(IrConstant::Unit)
    }
  }

  /// Baja un statement del AST a instrucciones IR.
  fn lower_stmt(&mut self, stmt_id: StmtId) {
    match self.ast.stmt(stmt_id) {
      Stmt::LetBinding { var, initializer } | Stmt::ConstBinding { var, initializer } => {
        let symbol = self.lower_lvalue(*var);
        let value = self.lower_expr(*initializer);
        self.bind_symbol(symbol, value);
      }
      Stmt::Assign { var, value_expr } => {
        let symbol = self.lower_lvalue(*var);
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
      Stmt::Return(Some(expr_id)) => {
        let value = self.lower_expr(*expr_id);
        self.builder.emit_return(Some(value));
      }
      Stmt::Return(None) => {
        self.builder.emit_return(None);
      }
    }
  }

  /// Evalua una expresion y devuelve el ValueId resultante.
  fn lower_expr(&mut self, expr_id: ExprId) -> ValueId {
    match self.ast.expr(expr_id) {
      Expr::Var(_) => {
        let symbol = self
          .semantic
          .resolution_info
          .symbol_of(expr_id)
          .expect("debe haber un simbolo para la expresion");
        self
          .env
          .get(symbol)
          .expect("debe haber un simbolo para el valor")
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
  fn lower_lvalue(&mut self, expr_id: ExprId) -> SymbolId {
    match self.ast.expr(expr_id) {
      Expr::Var(_) => self
        .semantic
        .resolution_info
        .symbol_of(expr_id)
        .expect("debe haber un simbolo para la expresion"),
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
    let else_val = if let Some(else_expr) = else_branch {
      self.lower_expr(else_expr)
    } else {
      self.builder.emit_const(IrConstant::Unit)
    };
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

    let phi_inputs = vec![
      PhiInput::new(if_block, if_val),
      PhiInput::new(else_block, else_val),
    ];
    self.builder.emit_phi(branch_ty.into(), phi_inputs)
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
