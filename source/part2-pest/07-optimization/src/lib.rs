//! Part 2: Optimization - reuses Part 1

pub use lumina_part1_optimization::OptLevel;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opt_level() {
        assert_eq!(OptLevel::O2.to_opt_arg(), "-O2");
    }
}
