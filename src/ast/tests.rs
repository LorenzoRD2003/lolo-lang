use crate::{
  ast::{
    Ast, BlockId, ExprId, StmtId,
    block::Block,
    expr::{ConstValue, Expr},
    stmt::Stmt,
  },
  parser::parse_program,
};
use proptest::prelude::*;

// // Helpers para generar AST arbitrarios

// fn arb_const() -> impl Strategy<Value = Expr> {
//   prop_oneof![
//     any::<i32>().prop_map(|v| Expr::Const(v.into())),
//     any::<bool>().prop_map(|v| Expr::Const(v.into())),
//   ]
// }

// use proptest::prelude::*;

// type ExprGen = Box<dyn Fn(&mut Ast) -> ExprId>;

// fn arb_expr(depth: u32) -> BoxedStrategy<ExprGen> {
//   if depth == 0 {
//     return Some(prop_oneof![
//       any::<i32>().prop_map(|v| {
//         Box::new(move |ast: &mut Ast| ast.add_expr(Expr::Const(v.into()))) as ExprGen
//       }),
//       any::<bool>().prop_map(|v| {
//         Box::new(move |ast: &mut Ast| ast.add_expr(Expr::Const(v.into()))) as ExprGen
//       }),
//     ]
//     .boxed();
//   }

//   let leaf = arb_expr(0);

//   let unary = arb_expr(depth - 1).prop_map(|inner| {
//     Box::new(move |ast: &mut Ast| {
//       let operand_id = inner(ast);
//       ast.add_expr(Expr::Unary(UnaryExpr {
//         op: UnaryOp::Neg,
//         operand: operand_id,
//       }))
//     }) as ExprGen
//   });

//   let binary = (arb_expr(depth - 1), arb_expr(depth - 1)).prop_map(|(lhs_gen, rhs_gen)| {
//     Box::new(move |ast: &mut Ast| {
//       let lhs_id = lhs_gen(ast);
//       let rhs_id = rhs_gen(ast);
//       ast.add_expr(Expr::Binary(BinaryExpr {
//         op: BinaryOp::Add,
//         lhs: lhs_id,
//         rhs: rhs_id,
//       }))
//     }) as ExprGen
//   });

//   prop_oneof![leaf, unary, binary].boxed()
// }

// Test suite
#[test]
fn test_add_and_retrieve_expr() {
  let mut ast = Ast::empty();
  let expr = Expr::Const(ConstValue::Int32(420));
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
  let expr = Expr::Const(ConstValue::Int32(42));
  let id = ast.add_expr(expr, span1);
  ast.update_expr_span(id, span2.clone());
  assert_eq!(ast.expr_span(id), span2);
}

#[test]
fn test_add_and_retrieve_stmt() {
  let mut ast = Ast::empty();
  let span = 0..4;
  let stmt = Stmt::Return(Some(ExprId(0)));
  let id = ast.add_stmt(stmt.clone(), span.clone());
  assert_eq!(ast.stmt(id), stmt);
  assert_eq!(ast.stmt_span(id), span);
}

#[test]
fn test_multiple_exprs_stmts_blocks() {
  let mut ast = Ast::empty();

  let e1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let e2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
  assert_eq!(ast.expr(e1), Expr::Const(ConstValue::Int32(1)));
  assert_eq!(ast.expr(e2), Expr::Const(ConstValue::Int32(2)));

  let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
  let s2 = ast.add_stmt(Stmt::Expr(e2), 2..3);
  assert_eq!(ast.stmt(s1), Stmt::Expr(e1));
  assert_eq!(ast.stmt(s2), Stmt::Expr(e2));

  let b1 = ast.add_block(Block::with_stmts(&ast, vec![s1, s2]), 0..3);
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
  let e1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let e2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
  let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
  let s2 = ast.add_stmt(Stmt::Return(Some(e2)), 2..3);

  // Verifico que todos los ExprId referenciados por Stmt existen
  for stmt_id in &[s1, s2] {
    match ast.stmt(*stmt_id) {
      Stmt::Expr(eid) | Stmt::Return(Some(eid)) | Stmt::Print(eid) => {
        let _ = ast.expr(eid); // si no existe -> panic
      }
      _ => {}
    }
  }
}

#[test]
fn all_block_stmt_ids_are_valid() {
  let mut ast = Ast::empty();
  let e1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
  let s2 = ast.add_stmt(Stmt::Return(Some(e1)), 2..3);

  let block = ast.add_block(Block::with_stmts(&ast, vec![s1, s2]), 0..3);
  // Cada StmtId dentro del Block debe existir, sino panickea
  for stmt_id in ast.block(block).stmts() {
    let _ = ast.stmt(*stmt_id);
  }
}

#[test]
fn all_expr_ids_in_nested_blocks_are_valid() {
  let mut ast = Ast::empty();
  let e1 = ast.add_expr(Expr::Const(ConstValue::Int32(1)), 0..1);
  let e2 = ast.add_expr(Expr::Const(ConstValue::Int32(2)), 2..3);
  let s1 = ast.add_stmt(Stmt::Expr(e1), 0..1);
  let s2 = ast.add_stmt(Stmt::Return(Some(e2)), 2..3);
  let inner_block = ast.add_block(Block::with_stmts(&ast, vec![s1]), 0..1);
  let outer_block = ast.add_block(Block::with_stmts(&ast, vec![s2]), 2..3);

  // Chequeo recursivo: todos los ExprId referenciados existen
  for block_id in &[inner_block, outer_block] {
    for stmt_id in ast.block(*block_id).stmts() {
      match ast.stmt(*stmt_id) {
        Stmt::Expr(eid) | Stmt::Return(Some(eid)) | Stmt::Print(eid) => {
          let _ = ast.expr(eid);
        }
        _ => {}
      }
    }
  }
}

#[test]
fn block_with_return_has_tail_expr() {
  let source = r#"
    main {
      let x = {
        const a = 5;
        return a + 2;
      };
    }
  "#;
  let (ast, program) = parse_program(source);
  let main_block = ast.block(program.main_block(&ast));
  let stmt_id = main_block.stmts()[0];

  let block_id = match ast.stmt(stmt_id) {
    Stmt::LetBinding { initializer, .. } => match ast.expr(initializer) {
      Expr::Block(bid) => bid,
      _ => panic!("Expected block expression"),
    },
    _ => panic!("Expected let binding"),
  };

  let block = ast.block(block_id);
  assert!(block.tail_expr().is_some());
  let tail_expr = block.tail_expr().unwrap();
  match ast.expr(tail_expr) {
    Expr::Binary(_)
    | Expr::Var(_)
    | Expr::Const(_)
    | Expr::Unary(_)
    | Expr::Block(_)
    | Expr::If(_) => {}
  }
}

#[test]
fn block_without_return_has_no_tail_expr() {
  let source = r#"
    main {
      let x = {
        const a = 5;
      };
    }
  "#;
  let (ast, program) = parse_program(source);
  let main_block = ast.block(program.main_block(&ast));
  let stmt_id = main_block.stmts()[0];

  let block_id = match ast.stmt(stmt_id) {
    Stmt::LetBinding { initializer, .. } => match ast.expr(initializer) {
      Expr::Block(bid) => bid,
      _ => panic!("Expected block expression"),
    },
    _ => panic!("Expected let binding"),
  };
  let block = ast.block(block_id);
  assert!(block.tail_expr().is_none());
}

#[test]
fn block_tail_expr_matches_return_expression() {
  let source = r#"
    main {
      let x = {
        const a = 5;
        return a + 1;
      };
    }
  "#;
  let (ast, program) = parse_program(source);
  let main_block = ast.block(program.main_block(&ast));
  let stmt_id = main_block.stmts()[0];

  let block_id = match ast.stmt(stmt_id) {
    Stmt::LetBinding { initializer, .. } => match ast.expr(initializer) {
      Expr::Block(bid) => bid,
      _ => panic!("Expected block expression"),
    },
    _ => panic!("Expected let binding"),
  };
  let block = ast.block(block_id);
  let tail_expr = block.tail_expr().unwrap();
  let last_stmt = *block.stmts().last().unwrap();
  let expected_tail_expr = match ast.stmt(last_stmt) {
    Stmt::Return(Some(expr_id)) => *expr_id,
    _ => panic!("Expected return statement at end of block"),
  };
  assert_eq!(tail_expr, expected_tail_expr);
}

#[test]
fn nested_block_has_independent_tail_expr() {
  let source = r#"
    main {
      let x = {
        const y = {
          return 5;
        };
        return y;
      };
    }
  "#;
  let (ast, program) = parse_program(source);
  let main_block = ast.block(program.main_block(&ast));
  let stmt_id = main_block.stmts()[0];

  let outer_block_id = match ast.stmt(stmt_id) {
    Stmt::LetBinding { initializer, .. } => match ast.expr(initializer) {
      Expr::Block(bid) => bid,
      _ => panic!("Expected outer block"),
    },
    _ => panic!("Expected let binding"),
  };
  let outer_block = ast.block(outer_block_id);
  assert!(outer_block.tail_expr().is_some());

  // Buscar el inner block
  let first_stmt = outer_block.stmts()[0];
  let inner_block_id = match ast.stmt(first_stmt) {
    Stmt::ConstBinding { initializer, .. } => match ast.expr(initializer) {
      Expr::Block(bid) => bid,
      _ => panic!("Expected inner block"),
    },
    _ => panic!("Expected const binding"),
  };

  let inner_block = ast.block(inner_block_id);
  assert!(inner_block.tail_expr().is_some());
}

proptest! {
  #[test]
  fn expr_span_indices_proptest(exprs in 0usize..100, start in 0usize..50, end in 51usize..100) {
    let mut ast = Ast::empty();
    for i in 0..exprs {
      let id = ast.add_expr(Expr::Const(ConstValue::Int32(i as i32)), start..end);
      let _ = ast.expr(id);
      let _ = ast.expr_span(id);
    }
    prop_assert_eq!(ast.expr_arena.len(), ast.expr_spans.len());
  }

  #[test]
  fn stmt_span_indices_proptest(stmts in 0usize..100, start in 0usize..50, end in 51usize..100) {
    let mut ast = Ast::empty();
    for i in 0..stmts {
      let id = ast.add_stmt(Stmt::Return(Some(ExprId(i))), start..end);
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
      stmt_ids.push(ast.add_stmt(Stmt::Return(Some(ExprId(i))), start..end));
    }
    for i in 0..blocks {
      let block_stmt_ids = stmt_ids[i*stmts_per_block..(i+1)*stmts_per_block].to_vec();
      let id = ast.add_block(Block::with_stmts(&ast, block_stmt_ids), start..end);
      let _ = ast.block(id);
      let _ = ast.block_span(id);
    }
    prop_assert_eq!(ast.stmt_arena.len(), ast.stmt_spans.len());
    prop_assert_eq!(ast.block_arena.len(), ast.block_spans.len());
  }
}
