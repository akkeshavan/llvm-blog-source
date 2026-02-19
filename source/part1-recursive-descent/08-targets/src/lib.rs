//! Multi-target code generation.
//! Target triple constants for llc.

use std::path::Path;
use std::process::Command;

pub fn host_triple() -> &'static str {
    if cfg!(target_arch = "x86_64") && cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
        "aarch64-apple-darwin"
    } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "linux") {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "linux") {
        "aarch64-unknown-linux-gnu"
    } else {
        "x86_64-unknown-linux-gnu"
    }
}

pub fn compile_ir_to_object(ir_path: &Path, out_path: &Path, triple: &str) -> Result<(), String> {
    let status = Command::new("llc")
        .args(["-mtriple", triple, "-filetype=obj"])
        .arg(ir_path)
        .arg("-o")
        .arg(out_path)
        .status()
        .map_err(|e| format!("failed to run llc: {e}"))?;

    if !status.success() {
        return Err(format!("llc failed with exit code: {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_triple_non_empty() {
        let t = host_triple();
        assert!(!t.is_empty());
        assert!(t.contains("-"));
    }
}
