# Lumina Compiler — Rust Examples

This directory contains **Rust** examples for building the **Lumina** compiler using **LLVM** (via Inkwell). The blog is structured in two parts:

- **Part 1** — Recursive descent parser + custom lexer (`part1-recursive-descent/`)
- **Part 2** — Grammar-driven lexer and parser using **pest** (`part2-pest/`)

All examples use **Rust** and **Cargo**; every example has **tests** that run via `cargo test`.

## Prerequisites

- **Rust** (rustc, cargo) via rustup
- **LLVM 17 or 18** (for 00-install, 04-codegen, 09-lum-cli; set `LLVM_SYS_170_PREFIX` if needed)
- **Inkwell** crate with matching LLVM feature (llvm17-0 or llvm18-0)

## Build & Test

From this directory:

```bash
# Part 1 - all crates
cd part1-recursive-descent/00-install && cargo build && cargo test && cd ../..
cd part1-recursive-descent/01-lexer && cargo build && cargo test && cd ../..
# ... etc

# Part 1 crates that don't need LLVM (01-lexer, 02-parser, 03-typecheck, 05-08):
cd part1-recursive-descent/01-lexer && cargo test  # always works

# Part 1 crates that need LLVM (00-install, 04-codegen, 09-lum-cli):
cd part1-recursive-descent/00-install && cargo test  # requires LLVM 17
cd part1-recursive-descent/04-codegen && cargo test  # requires LLVM 17
cd part1-recursive-descent/09-lum-cli && cargo test  # requires LLVM 17

# Part 2: same pattern. 01-lexer, 02-parser, 03-typecheck, 05-08 work without LLVM.
# 00-install, 04-codegen, 09-lum-cli require LLVM.
```

## Layout

| Directory | Description |
|-----------|-------------|
| **part1-recursive-descent/** | Part 1: recursive descent + custom lexer. 00-install … 09-lum-cli. All have tests. |
| **part2-pest/** | Part 2: Grammar-driven (pest) lexer and parser. 00-install … 09-lum-cli. Grammar files in `01-lexer/grammar/`, `02-parser/grammar/`. |
