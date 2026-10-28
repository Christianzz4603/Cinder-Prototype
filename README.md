# Cinder

**Write naturally. Compile natively. Run anywhere.**

Cinder is an experimental general-purpose systems language built around readable syntax, native performance, low-level control, and an intended safe-memory model.

The compiler is written in Rust. It parses Cinder, runs an initial semantic/type-checking pass, lowers the program into a backend-independent intermediate representation (IR), emits C++20, and invokes a native C++ compiler. Programs become native executables rather than virtual-machine bytecode.

## Implemented features

- Typed functions, parameters, return types, recursion, and forward calls
- Immutable `let` bindings and mutable `var` bindings
- Primitive types: `i32`, `i64`, `u32`, `u64`, `f32`, `f64`, `bool`, `String`, and `str` as an alias for String
- Decimal floating-point literals and hexadecimal, binary, and octal integers
- Generic array types such as `Array<i64>`, including nested arrays
- Array literals, bounds-checked indexing, indexed mutation for `var` arrays, and dynamic push/pop
- if/else-if/else, while loops, range loops, and array for-each loops
- break and continue with loop-context validation
- Arithmetic, comparisons, equality, logical, bitwise, shift, and unary operators
- String concatenation, common string escapes, triple-quoted multiline strings (`\"\"\"...\"\"\"`), raw strings (`r\"...\"` and `r\"\"\"...\"\"\"`), and `//` comments
- Built-ins: print, input, len, push, pop, abs, min, max, clamp, sqrt, pow, floor, ceil, round, to_string, parse_int, contains, starts_with, ends_with, is_even, is_odd, and sign
- Native optimization configuration and optional link-time optimization
- IR and generated C++ inspection commands

## Example

```cind
fn main() {
    var scores: Array<i64> = [12, 18, 25]
    push(scores, 30)
    scores[0] = 15

    var total: i64 = 0
    for score in scores {
        total = total + score
    }

    print(total)
    print(pop(scores))
}
```

## Requirements

Install Rust stable and a C++20 compiler such as `g++` or `clang++`.

## Build and run

From the repository root:

```sh
cargo test
cargo install --path .
cinder check examples/hello.cind
cinder build examples/hello.cind -o hello
cinder run examples/hello.cind
cinderun examples/hello.cind
```

The `cinderun` command is a convenience runner, like `python hello.py`: it compiles the Cinder source to a temporary native executable, runs it, then removes the temporary executable. It does not leave a compiled binary in the project directory.

You can also use Cargo without installing:

```sh
cargo run -- check examples/hello.cind
cargo run -- run examples/hello.cind
```

Choose a compiler with the `CXX` environment variable, for example `CXX=clang++`. Release builds default to native `-O3` optimization. Set `CINDER_OPT_LEVEL=0|1|2|3|s|z` to choose another level, or `CINDER_LTO=1` to request link-time optimization. Cinder does not force `-march=native`, helping binaries remain portable across CPUs.

## Current limitations

Cinder is actively evolving. The compiler lowers validated source into a backend-independent structural IR, but the current backend still emits C++20 and invokes an external native compiler; LLVM or a direct native backend remains future work. The semantic checker is an initial implementation, not a complete type system. Multiline and raw string literals are supported; string interpolation is still planned. Arrays use C++ `std::vector` storage and bounds-checked access, but Cinder does not yet provide ownership, borrowing, or lifetime analysis. Structs, enums, modules, full generic functions, package management, cross-compilation, and a production-quality standard library are not implemented. **Cinder does not currently guarantee memory safety.**

A game engine would be a separate library built on top of Cinder, not part of the compiler itself.

## Design principles

1. Readable syntax with minimal ceremony.
2. Native compilation and low-level programming remain first-class goals.
3. Establish and enforce memory-safety guarantees before claiming them.
4. Measure performance against optimized C++ and Rust with reproducible benchmarks.
5. Grow through small, tested milestones.

See `docs/roadmap.md` for the staged plan.
