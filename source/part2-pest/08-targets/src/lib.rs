//! Part 2: Multi-target - reuses Part 1

pub use lumina_part1_targets::host_triple;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_triple_non_empty() {
        assert!(!host_triple().is_empty());
    }
}
