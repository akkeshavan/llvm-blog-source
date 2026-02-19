//! Part 2: GC integration - same as Part 1

pub const ALLOC: &str = "lumina_alloc";
pub const PUSH_ROOT: &str = "lumina_push_root";
pub const POP_ROOT: &str = "lumina_pop_root";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gc_constants() {
        assert_eq!(ALLOC, "lumina_alloc");
    }
}
