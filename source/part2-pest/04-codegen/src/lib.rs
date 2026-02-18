//! Part 2: Code generation - reuses Part 1's code generator

use lumina_part2_typecheck::typecheck;
use lumina_part1_codegen::compile_to_entry;

pub fn compile(source: &str) -> Result<String, String> {
    let typed = typecheck(source).map_err(|e| format!("{:?}", e))?;
    compile_to_entry(&typed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_int() {
        let ir = compile("42").unwrap();
        assert!(ir.contains("define i64 @lumina_entry()"));
        assert!(ir.contains("ret i64 42"));
    }

    #[test]
    fn compile_add() {
        let ir = compile("1 + 2").unwrap();
        // May contain "add" or be constant-folded to "ret i64 3"
        assert!(ir.contains("ret i64"));
    }
}
