# Cinder roadmap

This roadmap describes intended work, not completed features.

## Milestone 0.1: Bootstrap compiler
- [x] Rust-based command-line compiler
- [x] Initial readable source syntax
- [x] Lexer and parser for a small subset
- [x] C++ source generation and native compilation
- [x] Syntax checks and automated tests
- [ ] Better diagnostics and source spans
- [ ] Arithmetic and comparison expressions

## Milestone 0.2: A real language core
- [ ] Expression grammar and operator precedence
- [ ] if, else, while, and for
- [ ] User-defined functions and parameters
- [ ] Static type checker with explicit conversion rules
- [ ] Cinder-specific intermediate representation
- [ ] Parser fuzz tests and golden diagnostics

## Milestone 0.3: Memory and safety model
- [ ] Specify ownership, borrowing, and aliasing
- [ ] Specify value/reference semantics and object lifetimes
- [ ] Define safe and unsafe boundaries
- [ ] Define region and allocator APIs
- [ ] Add compile-fail tests for invalid programs
- [ ] Document guarantees and known limitations precisely

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
