//! Runtime and stdlib integration for Lumina.
//! Declares external functions that generated IR will call.

/// Runtime function names for IR generation
pub const PRINTLN_I64: &str = "lumina_println_i64";
pub const PRINTLN_STR: &str = "lumina_println_str";

/// Minimal C runtime source.
///
/// We keep this intentionally tiny: a `main()` wrapper calls the function emitted
/// by our code generator (see `ENTRY_FN_NAME`) and prints its result.
pub const RUNTIME_C: &str = r#"
#include <stdio.h>
#include <stdint.h>

// Defined in the generated LLVM module.
int64_t lumina_entry(void);

void lumina_println_i64(int64_t x) {
    printf("%lld\n", (long long)x);
}

void lumina_println_str(const char* s) {
    printf("%s\n", s);
}

int main(void) {
    printf("%lld\n", (long long)lumina_entry());
    return 0;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_part1_codegen::ENTRY_FN_NAME;

    #[test]
    fn runtime_constants_defined() {
        assert_eq!(PRINTLN_I64, "lumina_println_i64");
        assert_eq!(PRINTLN_STR, "lumina_println_str");
        assert_eq!(ENTRY_FN_NAME, "lumina_entry");
    }
}
