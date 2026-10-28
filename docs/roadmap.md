# Cinder roadmap

This roadmap describes intended work, not completed features.

## Milestone 0.1: Bootstrap compiler
- [x] Rust-based command-line compiler
- [x] Initial readable source syntax
- [x] Lexer and parser for a small subset
- [x] C++ source generation and native compilation
- [x] Syntax checks and automated tests
- [x] Arithmetic, comparison, equality, and logical expressions
- [ ] Better diagnostics and source spans
- [x] Triple-quoted multiline strings and raw string literals
- [x] String interpolation with expression evaluation and escaped literal braces
- [x] Accept `str` as an alias for `String`

## Milestone 0.2: A real language core
- [x] Expression grammar and operator precedence
- [x] if and else blocks, including nested conditionals
- [x] Initial built-in utility and math functions
- [x] while loops, `break`, and `continue`
- [x] Range-based for loops and array for-each iteration
- [x] Array literals, explicitly typed empty arrays, indexing, indexed mutation, push, pop, indexed removal, clear/reserve, first/last access, contains, emptiness checks, and capacity queries
- [x] Array<T> type annotations, including function parameters
- [x] User-defined functions and parameters
- [x] Initial static type checker for variables, assignments, calls, operators, and boolean conditions
- [x] Basic return-path analysis for non-main functions
- [ ] Structs, enums, and field access
- [x] Initial exhaustive literal-pattern `match` statements
- [ ] Enum/destructuring patterns and match expressions
- [x] Basic generic functions with inferred type parameters
- [ ] Generic constraints, bounds, and specialization
- [x] Function overloading by parameter types
- [ ] Full operator overloading
- [x] Typed expression closures with value capture
- [ ] Block closures, mutable captures, and closure escape analysis
- [ ] Advanced compile-time evaluation
- [ ] Complete collection library and custom collection types
- [ ] Explicit ownership and borrowing of collections
- [ ] Array bounds safety and explicit collection semantics
- [x] Initial backend-independent structural IR and AST lowering
- [x] IR-based C++ backend
- [ ] Typed/SSA IR
- [ ] Parser fuzz tests and golden diagnostics

## Milestone 0.3: Memory and safety model
- [ ] Specify ownership, borrowing, and aliasing
- [ ] Specify value/reference semantics and object lifetimes
- [ ] Define safe and unsafe boundaries
- [ ] Define region and allocator APIs
- [ ] Add compile-fail tests for invalid programs
- [ ] Document guarantees and known limitations precisely

## Milestone 0.35: Runtime and performance foundations
- [x] Emit the runtime helpers required by built-in functions
- [x] Exercise every example through native compilation in CI
- [x] Use optimized native compiler output with configurable optimization and optional LTO
- [ ] Add benchmarks for compile time, executable size, and runtime throughput
- [ ] Add allocation-aware strings and explicit error handling for runtime conversions
- [x] Add typed user-defined functions, recursion, and forward-call support
- [x] Add `while` loops and variable assignment
- [x] Add `break` and `continue` with loop-context validation
- [ ] Add `for` loops, `break`, and `continue` before engine-scale examples

## Milestone 0.4: Native backend
- [ ] Evaluate LLVM integration for supported targets
- [ ] Add target triples and cross-compilation configuration
- [ ] Add debug and release build profiles
- [ ] Produce native libraries and executables
- [ ] Benchmark against optimized C++ and Rust baselines

## Milestone 0.5+: Ecosystem
- [ ] Standard library and test framework
- [ ] Package/build tool
- [ ] C ABI interoperability
- [ ] Linux, Windows, and macOS CI targets
- [ ] Android NDK and WebAssembly targets
- [ ] Desktop/mobile frameworks and platform bindings

## Non-negotiable design constraints
- Never claim memory safety before the type system enforces the stated guarantees.
- Never claim performance leadership without reproducible benchmarks.
- Keep native compilation and low-level programming as first-class goals.
- Prefer small, tested milestones over a huge untestable compiler rewrite.
