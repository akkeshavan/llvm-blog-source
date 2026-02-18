//! Part 2: Lum CLI - init, build, run for Lumina projects (grammar-driven front-end)

use lumina_part2_codegen::compile;
use lumina_part1_runtime::RUNTIME_C;
use lumina_part1_targets::host_triple;
use std::path::PathBuf;
use std::path::{Path};
use std::process::{Command, Stdio};

pub fn init_project(dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(
        dir.join("lum.toml"),
        "[project]\nname = \"my-lumina\"\nversion = \"0.1.0\"\n",
    )?;
    std::fs::write(dir.join("src/main.lum"), "42\n")?;
    println!("Created Lumina project (Part 2) in {:?}", dir);
    Ok(())
}

fn ensure_artifacts_dir(project_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = project_dir.join(".lum");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_runtime_c(artifacts_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = artifacts_dir.join("runtime.c");
    std::fs::write(&path, RUNTIME_C)?;
    Ok(path)
}

fn compile_runtime_c(runtime_c: &Path, out_obj: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("clang")
        .args(["-c"])
        .arg(runtime_c)
        .arg("-o")
        .arg(out_obj)
        .status()?;
    if !status.success() {
        return Err(format!("clang failed: {status}").into());
    }
    Ok(())
}

fn link_executable(main_obj: &Path, runtime_obj: &Path, out_exe: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("clang")
        .arg(main_obj)
        .arg(runtime_obj)
        .arg("-o")
        .arg(out_exe)
        .status()?;
    if !status.success() {
        return Err(format!("link failed: {status}").into());
    }
    Ok(())
}

/// On macOS, compile IR to object with clang so the Mach-O gets a proper
/// LC_BUILD_VERSION load command and the linker does not warn.
fn ir_to_object_macos(ir_path: &Path, obj_path: &Path, triple: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("clang")
        .args(["-c", "-target", triple])
        .arg(ir_path)
        .arg("-o")
        .arg(obj_path)
        .status()?;
    if !status.success() {
        return Err("clang -c (IR to object) failed".into());
    }
    Ok(())
}

pub fn build_project(project_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let main_lum = project_dir.join("src/main.lum");
    let source = if main_lum.exists() {
        std::fs::read_to_string(&main_lum)?
    } else {
        "42".to_string()
    };
    let ir = compile(&source)?;

    let artifacts_dir = ensure_artifacts_dir(project_dir)?;
    let ir_path = artifacts_dir.join("main.ll");
    std::fs::write(&ir_path, &ir)?;

    let obj_path = artifacts_dir.join("main.o");
    let triple = host_triple();
    if cfg!(target_os = "macos") {
        ir_to_object_macos(&ir_path, &obj_path, triple)?;
    } else {
        let typed = lumina_part2_typecheck::typecheck(&source).map_err(|e| format!("{e:?}"))?;
        lumina_part1_codegen::write_object_file(&typed, triple, &obj_path)
            .map_err(|e| format!("object emission failed: {e}"))?;
    }

    let runtime_c = write_runtime_c(&artifacts_dir)?;
    let runtime_obj = artifacts_dir.join("runtime.o");
    compile_runtime_c(&runtime_c, &runtime_obj)?;

    let exe_path = artifacts_dir.join("main");
    link_executable(&obj_path, &runtime_obj, &exe_path)?;

    println!("Build succeeded (Part 2): {}", exe_path.display());
    Ok(exe_path)
}

pub fn run_project(project_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let exe = build_project(project_dir)?;
    let output = Command::new(&exe)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        return Err(format!("program exited with: {}", output.status).into());
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn have_tool(name: &str) -> bool {
        Command::new(name)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }

    #[test]
    fn init_creates_project() {
        let dir = tempfile::tempdir().unwrap();
        init_project(&dir.path().to_path_buf()).unwrap();
        assert!(dir.path().join("src/main.lum").exists());
        assert!(dir.path().join("lum.toml").exists());
    }

    #[test]
    fn build_produces_ir() {
        if !have_tool("clang") {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        init_project(&dir.path().to_path_buf()).unwrap();
        let exe = build_project(&dir.path().to_path_buf()).unwrap();
        assert!(exe.exists());
    }

    #[test]
    fn run_prints_result() -> Result<(), io::Error> {
        if !have_tool("clang") {
            return Ok(());
        }
        let dir = tempfile::tempdir().unwrap();
        init_project(&dir.path().to_path_buf()).unwrap();
        std::fs::write(dir.path().join("src/main.lum"), "42\n")?;
        let exe = build_project(&dir.path().to_path_buf()).unwrap();
        let output = Command::new(&exe).output()?;
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");
        Ok(())
    }
}
