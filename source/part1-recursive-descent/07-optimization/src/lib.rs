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

    #[test]
    fn opt_level_args() {
        assert_eq!(OptLevel::O2.to_opt_arg(), "-O2");
    }
}
