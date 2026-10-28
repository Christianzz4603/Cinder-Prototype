# Cinder

**Write naturally. Compile natively. Run anywhere.**

Cinder is an experimental general-purpose systems language built around readable syntax, native performance, low-level control, and an intended safe-memory model.

The compiler is written in Rust. It parses Cinder, runs an initial semantic/type-checking pass, emits C++20, and invokes a native C++ compiler. Programs become native executables rather than virtual-machine bytecode.

## Implemented features

- Typed functions, parameters, return types, recursion, and forward calls
- Immutable let bindings and mutable var bindings
- Primitive types: i32, i64, u32, u64, f32, f64, bool, String, and str as an alias for String
- Decimal floating-point literals and hexadecimal, binary, and octal integers
- Generic array types such as Array<i64>, including nested array types
- Array literals, bounds-checked indexing, indexed mutation for var arrays, and dynamic push/pop
- if/else-if/else, while loops, range-based for loops, and array for-each loops
- break and continue with loop-context validation
- Arithmetic, comparisons, equality, logical operators, bitwise operators, shifts, and unary operators
- String concatenation, common string escapes, and // comments
- Built-ins: print, input, len, push, pop, abs, min, max, clamp, sqrt, pow, floor, ceil, round, to_string, parse_int, contains, starts_with, ends_with, is_even, is_odd, and sign
- Native optimization configuration and optional link-time optimization

## Example

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

## Try it

Requirements: Rust stable and a C++20 compiler such as g++ or clang++.

    cargo test
    cargo run -- check examples/hello.cind
    cargo run -- run examples/hello.cind
    cargo run -- build examples/hello.cind -o hello
    ./hello

Choose a compiler with the CXX environment variable, for example CXX=clang++ cargo run -- build examples/hello.cind.

Release builds default to native -O3 optimization. Set CINDER_OPT_LEVEL=0|1|2|3|s|z to choose another level. Set CINDER_LTO=1 to enable link-time optimization when supported. Cinder does not force -march=native, which helps keep binaries portable across CPUs.

## Current limitations

Cinder is actively evolving. The backend still translates to C++ rather than using a Cinder-specific IR or LLVM backend. The semantic checker is an initial implementation, not a complete type system. Arrays use C++ std::vector storage and bounds-checked access, but the language does not yet provide ownership, borrowing, or lifetime analysis. Structs, enums, modules, full generic functions, package management, cross-compilation, and a production-quality standard library are not implemented. **Cinder does not currently guarantee memory safety.**

A game engine would be a separate library built on top of Cinder, not part of the compiler itself.

## Design principles

1. Readable syntax with minimal ceremony.
2. Native compilation and low-level programming remain first-class goals.
3. Establish and enforce memory-safety guarantees before claiming them.
4. Measure performance against optimized C++ and Rust with reproducible benchmarks.
5. Grow through small, tested milestones.

See docs/roadmap.md for the staged plan.