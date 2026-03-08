# lolo-lang

> A small, well-engineered toy programming language written in Rust — built with discipline.

`lolo-lang` is a small experimental language developed as a compiler construction project.  
Although the language itself is intentionally minimal, the implementation emphasizes:

- Clean architecture
- Strong invariants
- Rigorous semantic analysis
- Extensive test coverage
- Explicit error diagnostics

This project is not about building a large language — it is about building a _correct_ one.

---

# 📌 Why This Project Exists

`lolo-lang` is a vehicle for:

- Learning compiler architecture deeply
- Practicing rigorous semantic design
- Applying engineering discipline to language tooling
- Exploring IR design and lowering strategies

This is not a scripting language project. It is a compiler engineering project.

---

# ✨ Current Status

The following modules are complete and conceptually stable:

- ✅ Lexer
- ✅ Parser
- ✅ AST
- ✅ Diagnostics
- ✅ Semantic analysis (NameResolver -> TypeChecker -> ConstantTimeCompileChecker -> MutabilityChecker -> CategoryChecker)
- ✅ Frontend (Lexer → Parser → Semantic Pipeline)

The frontend is implemented as a configurable, extensible pipeline with
well-defined compilation stages.

The next planned phase is **lowering to an intermediate representation (IR)**.

---

# 🧠 Language Features (Current)

`lolo-lang` currently supports:

### Expressions

- Integer literals (`Int32`)
- Boolean literals
- Unary operators:
  - Logical NOT (`!`)
  - Arithmetic negation (`-`)
- Binary operators:
  - Arithmetic: `+`, `-`, `*`, `/`
  - Comparisons: `<`, `<=`, `>`, `>=`, `==`, `!=`
  - Logical: `&&`, `||`
- Block expressions

### Statements

- `let` and `const` bindings
- `assign` to already declared variables
- `return`
- `if`
- `if/else`
- `print`

### Semantic Analysis Phases

The semantic pipeline is structured as independent passes:

1. Name Resolution

- Scoped symbol resolution
- Shadowing rules
- Redeclaration diagnostics

2. Type Checking

- Strict static typing
- Unary and Binary operators type validation
- Boolean conditions check
- Type mismatch diagnostics

3. Mutability Analysis

- Mutable by default bindings
- Assignment validation

4. Compile-Time Constant Analysis

- Constant propagation
- Arithmetic overflow detection
- Division by zero detection

5. Category Analysis

- L-value / R-value classification
- Assignment target validation

Each pass operates over the AST using a visitor-based traversal model.

### Compilation Modes

The frontend supports configurable compilation policies:

- Strict mode (fail-fast)
- CLI/tolerant mode (collect diagnostics)
- IDE-oriented mode

This enables future integration with CLI tools and future IDE support.

---

# 🏗 Architecture Overview

The compiler is structured as a clean pipeline:

```
source code
↓
Frontend Pipeline
↓
Parsing Stage
↓
Semantic Stages:
  - Name Resolution
  - Type Checking
  - Mutability Analysis
  - Compile-Time Constant Analysis
  - Category Analysis
↓
(coming next: IR lowering stage)
```

Each stage is clearly separated and tested independently.

## Frontend Architecture

The frontend is implemented as a staged pipeline.

- The compilation pipeline uses trait-based dynamic dispatch for extensibility.

- Individual semantic passes use a statically-dispatched visitor pattern for AST traversal.

This separation ensures:

- Extensibility at the pipeline level

- Performance and clarity at the AST traversal level

Current frontend stages:

1. Parsing
2. Semantic Analysis:
   - Name Resolution
   - Type Checking
   - Mutability Checking
   - Compile-Time Constant Analysis
   - Category Checking

## Extensibility

The frontend pipeline is designed to be open for extension. Compilation phases are modeled as trait-based stages:

- Each stage implements a common interface.
- Stages are dynamically dispatched.
- Adding a new phase (e.g., IR lowering, optimizations) requires no modification to existing stages.

---

# 📦 Public API

The crate intentionally exposes a minimal public API:

- `Frontend`

- `FrontendConfig`

- `FrontendResult`

- `Renderer`

The AST, semantic internals, and compiler passes remain encapsulated.

---

# 🧪 Testing Strategy

The project includes:

- Extensive unit tests for all modules
- Negative tests (invalid programs)
- Positive tests (valid programs)
- Edge-case coverage
- Constant-folding tests
- Overflow detection tests
- Scope interaction tests
- Shadowing and redeclaration tests

### Coverage Metrics

Measured using `cargo llvm-cov`.

Final coverage:

- **Functions:** 95.83%
- **Lines:** 94.17%
- **Regions:** 93.66%
- **Branches:** 89.29%

Uncovered code has been manually reviewed and confirmed to be either:

- Structurally unreachable
- Defensive invariants
- Currently unused internal properties

---

# 🚀 Running Tests

```bash
cargo test
```

With coverage:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov
```

For branch coverage (requires nightly):

```bash
cargo +nightly llvm-cov --html --branch
```

---

# 🖥 Running the Compiler (CLI)

lolo-lang includes a simple CLI driver for compiling source files.

Example:

```bash
cargo run -- archivo.lolo
```

Source files are expected to be placed inside:

```
files-lang/
```

If the program contains no diagnostics:

```
No se encontraron diagnosticos sobre el programa. O sea, todo piola!
```

Otherwise, formatted diagnostics are rendered using the internal `Renderer`.
This is a code snippet example from `lolo-lang` that produces a semantic error:

```lolo
main {
  const x = 10;
  x = 20;
  return x + 1;
}
```

---

# 🛣 Roadmap

Next milestones:

- Design a simple intermediate representation (IR)
- Implement lowering from AST to IR
- Introduce control flow graph (CFG)
- Optimization passes
- Optional interpreter or backend

---

### Author

Lorenzo Ruiz Diaz from Eryx

- [LinkedIn](https://www.linkedin.com/in/lorenzo-ruiz-diaz-6bb61521a/)
- [GitHub](https://github.com/LorenzoRD2003)
- [Eryx Website](https://eryx.co/)
