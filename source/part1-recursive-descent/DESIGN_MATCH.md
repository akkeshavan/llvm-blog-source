# Sum types and match expressions

Sum types and match expressions are implemented in Lumina.

## Sum types

- **Syntax:** `type Name = Variant1(T1, T2) | Variant2 | Variant3(T3);`
- Variants can have zero or more payload types. Constructor calls: `Variant1(e1, e2)`, `Variant2`, etc.
- **Codegen:** Sums are laid out as `{ i64 tag, i64 payload }`. Only 0 or 1 payload per variant is supported in codegen; multiple payloads are rejected.

## Match expressions

- **Syntax:** `match expr with | Variant1(x, y) -> e1 | Variant2 -> e2 ... end`
- **Rules:**
  - All variants of the sum type must appear exactly once (exhaustive match). Duplicate or missing variant is a type error.
  - Each arm may bind payload names in parentheses; the arm body sees those bindings. All arms must have the same type (the type of the match).
- **Codegen:** Tag is loaded from the sum value; a switch on the tag dispatches to per-variant blocks. Payload is loaded and stored in an alloca for the binding; arm body is compiled; phi merges results into the merge block. The phi is inserted at the start of the merge block (not in the default/unreachable block).

The lexer already had `Match`, `With`, `End`; the parser, typechecker, and codegen now support sum types and match.
