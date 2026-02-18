//! Part 2: Runtime and stdlib - same as Part 1

pub const PRINTLN_I64: &str = "lumina_println_i64";
pub const PRINTLN_STR: &str = "lumina_println_str";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_constants() {
        assert_eq!(PRINTLN_I64, "lumina_println_i64");
    }
}
