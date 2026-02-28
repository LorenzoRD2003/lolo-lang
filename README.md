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
- ✅ Semantic analysis

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

### Statements

- `let` bindings
- `assign` to already declared variables
- `return`
- `if`
- `if/else`
- `print`

### Semantic Capabilities

- Scoped symbol resolution
- Shadowing rules
- Type checking
- Compile-time constant evaluation
- Arithmetic overflow detection
- Division by zero detection
- Structured diagnostics with spans
- Error recovery (analysis continues after errors)

---

# 🏗 Architecture Overview

The compiler is structured as a clean pipeline:

```
source code
↓
lexer
↓
parser
↓
AST
↓
semantic analysis
↓
(coming next: IR lowering)
```

Each stage is clearly separated and tested independently.

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

- **Functions:** 95.65%
- **Lines:** 95.85%
- **Regions:** 94.90%
- **Branches:** 91.23%

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

# 🔍 Project Structure

```
src/
 ├── lexer/
 ├── parser/
 ├── ast/
 ├── diagnostics/
 ├── semantic/
 └── lib.rs
```

Tests are organized within modules and structured by concern.

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
