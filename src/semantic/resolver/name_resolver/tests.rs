use crate::{
  ast::{ast::Ast, expr::Expr, program::Program, stmt::Stmt},
  diagnostics::diagnostic::Diagnostic,
  lexer::lexer::Lexer,
  parser::{parser::Parser, token_stream::TokenStream},
  semantic::resolver::{name_resolver::NameResolver, resolution_info::ResolutionInfo},
};

pub(crate) fn resolve(source: &str) -> (ResolutionInfo, Vec<Diagnostic>, Ast, Program) {
  // helper hipotetico
  let mut ts = TokenStream::new(Lexer::new(source));
  let mut parser = Parser::new(&mut ts);
  let program = parser
    .parse_program()
    .expect("el codigo fuente no pudo ser parseado correctamente");
  let ast = parser.into_ast();
  let mut resolver = NameResolver::new(&ast);
  resolver.resolve_program(program.clone());
  let diagnostics = resolver.diagnostics().to_vec();
  let resolution_info = resolver.into_resolution_info();
  (resolution_info, diagnostics, ast, program)
}

#[test]
fn resolves_simple_let_binding() {
  let source = r#"
    main {
      let x = 5;
    }
  "#;

  let (resolution_info, diagnostics, ast, program) = resolve(source);
  assert!(diagnostics.is_empty());
  let main_block = program.main_block();
  let stmt_id = ast.block(main_block).stmts()[0];

  let declared_symbol = resolution_info
    .declared_symbol_of_stmt(stmt_id)
    .expect("let should declare symbol");

  // Verifica que el var tenga simbolo
  let stmt = ast.stmt(stmt_id);
  if let Stmt::LetBinding { var, .. } = stmt {
    let symbol = resolution_info
      .symbol_of(var)
      .expect("la variable deberia tener un simbolo");
    assert_eq!(symbol, declared_symbol);
  } else {
    panic!("se esperaba un statement LetBinding");
  }
}

#[test]
fn resolves_variable_usage() {
  let source = r#"
    main {
      let x = 5;
      x = 10;
    }
  "#;

  let (resolution_info, diagnostics, ast, program) = resolve(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();
  let assign_stmt = ast.stmt(stmts[1]);

  let declared_symbol = resolution_info.declared_symbol_of_stmt(stmts[0]).unwrap();
  if let Stmt::Assign { var, .. } = assign_stmt {
    let symbol = resolution_info.symbol_of(var).unwrap();
    assert_eq!(symbol, declared_symbol);
  } else {
    panic!("se esperaba un statement Assign");
  }
}

#[test]
fn detects_redeclaration_in_same_scope() {
  let source = r#"
    main {
      let x = 1;
      let x = 2;
    }
  "#;

  let (_, diagnostics, _, _) = resolve(source);
  assert_eq!(diagnostics.len(), 1);
  assert!(
    diagnostics[0]
      .msg()
      .contains("la variable 'x' ya fue declarada en este scope")
  )
}

#[test]
fn allows_shadowing_in_inner_scope() {
  let source = r#"
    main {
      let x = 1;
      if x == 1 {
        let x = 2;
      }
    }
    "#;

  let (_, diagnostics, _, _) = resolve(source);
  assert!(diagnostics.is_empty());
}

#[test]
fn detects_undefined_variable_in_assign() {
  let source = r#"
    main {
      x = 10;
    }
    "#;

  let (_, diagnostics, _, _) = resolve(source);
  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains("variable 'x' indefinida"))
}

#[test]
fn detects_undefined_variable_in_expression() {
  let source = r#"
    main {
      let x = y;
    }
    "#;

  let (_, diagnostics, _, _) = resolve(source);
  assert_eq!(diagnostics.len(), 1);
  assert!(diagnostics[0].msg().contains("variable 'y' indefinida"))
}

#[test]
fn assigns_scopes_to_block_stmt_and_expr() {
  let source = r#"
    main {
      let x = 5;
    }
  "#;

  let (resolution_info, _, ast, program) = resolve(source);
  let main_block = program.main_block();
  let block_scope = resolution_info.scope_of_block(main_block);
  let stmt_id = ast.block(main_block).stmts()[0];
  let stmt_scope = resolution_info.scope_of_stmt(stmt_id);
  assert_eq!(block_scope, stmt_scope);
}

#[test]
fn resolves_binary_expression_operands() {
  let source = r#"
    main {
      let x = 1;
      let y = x + 2;
    }
  "#;

  let (resolution_info, diagnostics, ast, program) = resolve(source);
  assert!(diagnostics.is_empty());
  let block = ast.block(program.main_block());
  let stmts = block.stmts();

  let stmt = ast.stmt(stmts[1]);
  if let Stmt::LetBinding { initializer, .. } = stmt {
    if let Expr::Binary(binary) = ast.expr(initializer) {
      let lhs_symbol = resolution_info.symbol_of(binary.lhs);
      assert!(lhs_symbol.is_some());
    } else {
      panic!("se esperaba una expresion binaria");
    }
  }
}

#[test]
fn assignment_in_inner_scope_resolves_to_the_same_symbol_id() {
  let source = r#"
    main {
      let x = 1;
      if x == 1 {
        x = 2;
      }
    }
  "#;

  let (resolution_info, diagnostics, ast, program) = resolve(source);
  assert!(diagnostics.is_empty());
  let main_block = ast.block(program.main_block());
  let main_block_stmts = main_block.stmts();
  assert!(
    resolution_info
      .declared_symbol_of_stmt(main_block_stmts[0])
      .is_some()
  );

  let symbol_outer_scope = if let Stmt::LetBinding {
    var,
    initializer: _,
  } = ast.stmt(main_block_stmts[0])
  {
    resolution_info
      .symbol_of(var)
      .expect("se esperaba un simbolo")
  } else {
    panic!("Se esperaba un statement LetBinding");
  };

  if let Stmt::If {
    condition: _,
    if_block: if_block_id,
  } = ast.stmt(main_block_stmts[1])
  {
    let if_block = ast.block(if_block_id);
    let assign_stmt_id = if_block.stmts()[0];
    assert!(
      resolution_info
        .declared_symbol_of_stmt(assign_stmt_id)
        .is_none()
    );
    
    if let Stmt::Assign { var, value_expr: _ } = ast.stmt(assign_stmt_id) {
      let symbol_inner_scope = resolution_info
        .symbol_of(var)
        .expect("se esperaba un simbolo");
      assert_eq!(symbol_outer_scope, symbol_inner_scope);
    } else {
      panic!("Se esperaba un statement If");
    }
  }
}
