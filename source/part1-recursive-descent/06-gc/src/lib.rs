//! GC integration concepts for Lumina.
//! Allocation and root tracking function names.

pub const ALLOC: &str = "lumina_alloc";
pub const PUSH_ROOT: &str = "lumina_push_root";
pub const POP_ROOT: &str = "lumina_pop_root";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gc_functions_defined() {
        assert_eq!(ALLOC, "lumina_alloc");
        assert_eq!(PUSH_ROOT, "lumina_push_root");
        assert_eq!(POP_ROOT, "lumina_pop_root");
    }
}
