# lolo-lang

`lolo-lang` is a toy language and compiler project focused on compiler engineering discipline:
clear architecture, strong invariants, semantic rigor, and explicit diagnostics.

## Current State

Implemented and tested:

- Lexer
- Parser (Pratt-style expressions + statements)
- AST arena model (`ExprId`, `StmtId`, `BlockId`)
- Diagnostics + source rendering
- Frontend staged pipeline
- Semantic analysis pipeline
- IR model + pretty printer
- AST -> IR lowering
- IR verification and CFG utilities

## Language Features

### Expressions

- `Int32` and `Bool` literals
- Unary: `!`, unary `-`
- Binary arithmetic: `+`, `-`, `*`, `/`
- Binary comparisons: `<`, `<=`, `>`, `>=`, `==`, `!=`
- Binary logical: `&&`, `||`, `^^`
- Block expressions
- `if / else` expressions

### Statements

- `let` bindings
- `const` bindings
- assignment
- `print`
- `return`

## Semantic Pipeline

Default semantic graph:

1. `NameResolver`
2. `TypeChecker` (depends on `NameResolver`)
3. `MutabilityChecker` (depends on `NameResolver`)
4. `CompileTimeConstantChecker` (depends on `NameResolver`)
5. `CategoryChecker` (depends on `CompileTimeConstantChecker`)

Highlights:

- Name and scope resolution with shadowing/redeclaration checks
- Static type checking for operators, assignments, `if` conditions/branches, blocks
- Mutability checks for assignments
- Compile-time constant evaluation with overflow/div-by-zero diagnostics
- Category checks (`place/value/constant`) for assignment and `const` rules

## IR Lowering Notes

Current lowering includes integration with semantic compile-time constants:

- `const` bindings that have compile-time values are materialized lazily on first `Expr::Var` use.
- This avoids emitting dead IR constants for intermediate const chains.
- Example: `const x = 5; const y = x + 3; const z = 2 * y; return z;` lowers to a single `const 16` + `return`.
- `if` expressions with compile-time constant condition are pruned during lowering.
- In those pruned cases, lowering does not emit `branch`, `jump`, or `phi`.
- `branch`/`phi` are emitted only when the condition is not compile-time constant.

## CLI Usage

Input files are read from `files-lang/`.

```bash
cargo run -- <archivo.lolo> [--timings] [--dump-ir]
```

Examples:

```bash
cargo run -- const_propagation_chain.lolo
cargo run -- const_propagation_chain.lolo --timings --dump-ir
cargo run --features ir-verify -- const_propagation_chain.lolo
```

Flags:

- `--timings`: prints stage timings
- `--dump-ir`: prints the generated IR (debug format)

Compile-time feature flags:

- `ir-verify`: enables expensive IR invariants verification (useful for debugging)

## Testing

Run all tests:

```bash
cargo test
```

Useful focused suites:

```bash
cargo test semantic::
cargo test ir::lowering::tests
```

Optional coverage:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov
```

## Public API (crate)

- `Frontend`
- `FrontendConfig`
- `FrontendResult`
- `Renderer`
- `CliOptions`

## Roadmap

- Optimization passes over IR
- Optional backend / interpreter exploration
- Continued semantic and IR invariants hardening

## Author

Lorenzo Ruiz Diaz

- [LinkedIn](https://www.linkedin.com/in/lorenzo-ruiz-diaz-6bb61521a/)
- [GitHub](https://github.com/LorenzoRD2003)
- [Eryx](https://eryx.co)
