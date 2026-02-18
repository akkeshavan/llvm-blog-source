//! Optimization level configuration for Lumina.
//! Maps to LLVM opt -O0, -O1, -O2, -O3.

use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    pub fn to_opt_arg(&self) -> &'static str {
        match self {
            OptLevel::O0 => "-O0",
            OptLevel::O1 => "-O1",
            OptLevel::O2 => "-O2",
            OptLevel::O3 => "-O3",
        }
    }
}

pub fn optimize_ir(input: &Path, output: &Path, level: OptLevel) -> Result<(), String> {
    let status = Command::new("opt")
        .args([level.to_opt_arg()])
        .arg(input)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|e| format!("failed to run opt: {e}"))?;

    if !status.success() {
        return Err(format!("opt failed with exit code: {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn opt_level_args() {
        assert_eq!(OptLevel::O2.to_opt_arg(), "-O2");
    }

    /// Runs opt on a minimal IR file; requires `opt` on PATH (e.g. from LLVM).
    #[test]
    fn optimize_ir_produces_output() {
        let dir = tempfile::tempdir().expect("temp dir");
        let input_path = dir.path().join("in.ll");
        let output_path = dir.path().join("out.ll");

        let ir = r#"
define i64 @lumina_entry() {
entry:
  ret i64 42
}
"#;
        std::fs::File::create(&input_path)
            .expect("create in.ll")
            .write_all(ir.trim().as_bytes())
            .expect("write ir");

        let result = optimize_ir(&input_path, &output_path, OptLevel::O2);
        if let Err(e) = &result {
            if e.contains("failed to run opt") || e.contains("No such file") {
                eprintln!("skipping: opt not on PATH or LLVM not installed: {e}");
                return;
            }
        }
        result.expect("optimize_ir");

        assert!(output_path.exists(), "opt should produce output file");
        let out_content = std::fs::read_to_string(&output_path).expect("read output");
        assert!(
            out_content.contains("lumina_entry") || out_content.contains("42"),
            "output should contain our function or constant"
        );
    }
}
