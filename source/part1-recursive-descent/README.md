# Part 1: Recursive-descent Lumina compiler

This directory contains the **Part 1** implementation of the Lumina compiler (lexer, parser, type checker, codegen, runtime, CLI) as described in the blog chapters 2–12.

## Building and testing

There is **no workspace root** in this directory. Build and test **each crate** separately:

```bash
# Build the CLI (lum binary)
cd 09-lum-cli
cargo build --release

# Run tests in each crate (from this directory)
cd 01-lexer    && cargo test && cd ..
cd 02-parser   && cargo test && cd ..
cd 03-typecheck && cargo test && cd ..
cd 04-codegen  && cargo test && cd ..
cd 09-lum-cli  && cargo test && cd ..
```

The `09-lum-cli` tests require LLVM (and on macOS, a suitable toolchain) for `build_produces_ir` and related tests.

## Verifying the blog examples

After building `lum` in `09-lum-cli`, you can verify the three main examples (Hello World, FizzBuzz, array-of-records) as in **Chapter 12, Section 12.6**:

1. `lum init my-project && cd my-project`
2. Put `println("Hello, World!");` in `src/main.lum`, run `lum run` → expect `Hello, World!`
3. Replace with the FizzBuzz program from the blog, run `lum run` → expect lines 1, 2, Fizz, 4, Buzz, … FizzBuzz, … 19, Buzz
4. Replace with the array-of-records (filter_adults) program, run `lum run` → expect `2`

Optionally, run the verification script (from this directory):

```bash
./scripts/verify-examples.sh
```

This script builds `lum`, creates a temporary project, runs each example, and checks the expected output.
