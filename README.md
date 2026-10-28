# Cinder

**Write naturally. Compile natively. Run anywhere.**

Cinder is an experimental general-purpose systems language. Its goal is to pair readable syntax with native performance, strong safety guarantees in safe code, and low-level control when needed.

This repository is the first bootstrap prototype. It is deliberately small so the language can grow from a tested foundation instead of a pile of unimplemented promises.

## Current prototype

The compiler is written in Rust and translates the initial Cinder syntax subset into C++ source, then invokes a native C++ compiler. The generated program is a native executable, not a Cinder virtual-machine bytecode program.

Currently supported:
- `fn main() { ... }`
- `let` and `var` declarations with optional type annotations
- string, integer, boolean, and variable expressions
- `print(expression)`
- Built-in utility functions: `len`, `abs`, `min`, `max`, `clamp`, `sqrt`, `pow`, `floor`, `ceil`, `round`, `to_string`, `parse_int`, `input`, `contains`, `starts_with`, `ends_with`, `is_even`, `is_odd`, and `sign`
- `return expression`
- `if condition { ... }` and optional `else { ... }` blocks, including nesting
- Arithmetic, comparison, equality, logical, unary, and grouped expressions
- `//` comments and common string escapes

Now implemented: typed user-defined functions, parameters, return types, recursion, forward declarations, and user-function argument-count validation. Still planned: loops, a full type-aware standard library, real static type checking, ownership/borrowing, a custom intermediate representation, LLVM integration, package management, and cross-compilation. The native backend currently delegates optimization to the selected C++ compiler. A game engine would be a separate library/framework built on top of Cinder, not the language compiler itself; graphics, input, audio, asset loading, platform APIs, and a game loop still need to be built. Type annotations are accepted but not enforced, and this prototype does not yet guarantee memory safety. The current transpiler is a bootstrap, not yet a production compiler or a memory-safety system.

## Try it

Requirements: Rust stable and a C++20 compiler such as clang++ or g++.\n\nRelease builds default to native `-O3` optimization. Set `CINDER_OPT_LEVEL=0|1|2|3|s|z` to choose a compiler optimization level. Set `CINDER_LTO=1` to enable link-time optimization when your C++ toolchain supports it. Cinder does not force `-march=native`, so binaries remain more portable across CPUs by default.

    cargo test
    cargo run -- check examples/hello.cind
    cargo run -- run examples/hello.cind
    cargo run -- build examples/hello.cind -o hello
    ./hello

To select a compiler, set the CXX environment variable, for example: CXX=clang++ cargo run -- build examples/hello.cind

## Example

    fn main() {
        let language: str = "Cinder"
        let version: i32 = 1
        let score = 42

        print("Hello from Cinder!")
        print(language)
        print(version)

        if score >= 40 {
            print("Passed")
        } else {
            print("Try again")
        }
    }

## Design principles

1. Readable syntax with minimal ceremony.
2. Native compilation as the primary execution model.
3. Strong memory-safety guarantees in safe code, without requiring a garbage collector.
4. Explicit low-level capabilities for systems programming.
5. Cross-platform tooling and standard libraries, introduced incrementally.
6. Benchmark claims against optimized C++ and Rust rather than assuming superiority.

## Roadmap

See docs/roadmap.md for the staged plan and current limitations.
