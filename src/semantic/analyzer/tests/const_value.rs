use crate::{
  ast::{
    ast::Ast,
    expr::{ConstValue, Expr},
  },
  semantic::{analyzer::tests::semantic_analyzer, types::Type},
};

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
