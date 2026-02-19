//! Runtime and stdlib integration for Lumina.
//! Declares external functions that generated IR will call.

/// Runtime function names for IR generation
pub const PRINTLN_I64: &str = "lumina_println_i64";
pub const PRINTLN_STR: &str = "lumina_println_str";

/// Minimal C runtime source.
///
/// main() calls lumina_main. Stdlib: println/print, sqrt, max, min, arrays, optional unwrap.
/// Uses Boehm GC for heap allocation (arrays, etc.); link with -lgc.
pub const RUNTIME_C: &str = r#"
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <gc.h>

#define LUMINA_ARRAY_INIT_CAP 8
#define INT64_MIN_AS_NULL ((int64_t)0x8000000000000000LL)

void lumina_main(void);

void lumina_println_i64(int64_t x) { printf("%lld\n", (long long)x); }
void lumina_println_str(const char* s) { printf("%s\n", s); }
void lumina_print(const char* s) { printf("%s", s); }
void lumina_print_i64(int64_t x) { printf("%lld", (long long)x); }
int64_t lumina_sqrt(int64_t x) { return (int64_t)sqrt((double)x); }
int64_t lumina_max(int64_t a, int64_t b) { return a > b ? a : b; }
int64_t lumina_min(int64_t a, int64_t b) { return a < b ? a : b; }

typedef struct { size_t cap, len; int64_t *data; } lumina_arr_i64;
typedef struct { size_t cap, len; const char **data; } lumina_arr_str;

int64_t lumina_array_i64_new(void) {
    lumina_arr_i64 *a = (lumina_arr_i64*)GC_MALLOC(sizeof(lumina_arr_i64));
    a->cap = LUMINA_ARRAY_INIT_CAP;
    a->len = 0;
    a->data = (int64_t*)GC_MALLOC((size_t)a->cap * sizeof(int64_t));
    return (int64_t)(uintptr_t)a;
}
void lumina_array_i64_append(int64_t h, int64_t v) {
    lumina_arr_i64 *a = (lumina_arr_i64*)(uintptr_t)h;
    if (a->len >= a->cap) {
        a->cap *= 2;
        a->data = (int64_t*)GC_REALLOC(a->data, (size_t)a->cap * sizeof(int64_t));
    }
    a->data[a->len++] = v;
}
int64_t lumina_array_i64_get(int64_t h, int64_t i) {
    return ((lumina_arr_i64*)(uintptr_t)h)->data[i];
}
void lumina_array_i64_set(int64_t h, int64_t i, int64_t v) {
    ((lumina_arr_i64*)(uintptr_t)h)->data[i] = v;
}
int64_t lumina_array_i64_len(int64_t h) {
    return (int64_t)((lumina_arr_i64*)(uintptr_t)h)->len;
}

int64_t lumina_array_str_new(void) {
    lumina_arr_str *a = (lumina_arr_str*)GC_MALLOC(sizeof(lumina_arr_str));
    a->cap = LUMINA_ARRAY_INIT_CAP;
    a->len = 0;
    a->data = (const char**)GC_MALLOC((size_t)a->cap * sizeof(const char*));
    return (int64_t)(uintptr_t)a;
}
void lumina_array_str_append(int64_t h, const char* s) {
    lumina_arr_str *a = (lumina_arr_str*)(uintptr_t)h;
    if (a->len >= a->cap) {
        a->cap *= 2;
        a->data = (const char**)GC_REALLOC(a->data, (size_t)a->cap * sizeof(const char*));
    }
    a->data[a->len++] = s;
}
const char* lumina_array_str_get(int64_t h, int64_t i) {
    return ((lumina_arr_str*)(uintptr_t)h)->data[i];
}
void lumina_array_str_set(int64_t h, int64_t i, const char* s) {
    ((lumina_arr_str*)(uintptr_t)h)->data[i] = s;
}
int64_t lumina_array_str_len(int64_t h) {
    return (int64_t)((lumina_arr_str*)(uintptr_t)h)->len;
}

int64_t lumina_unwrap_i64(int64_t opt) {
    return (opt == INT64_MIN_AS_NULL) ? 0 : opt;
}
const char* lumina_unwrap_str(int64_t opt) {
    return (opt == 0) ? "" : (const char*)(uintptr_t)opt;
}

int main(void) {
    GC_INIT();
    lumina_main();
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
        assert_eq!(ENTRY_FN_NAME, "lumina_main");
    }
}
