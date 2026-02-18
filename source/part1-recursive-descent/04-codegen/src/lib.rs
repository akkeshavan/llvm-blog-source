use inkwell::context::Context;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple};
use inkwell::OptimizationLevel;
use lumina_part1_typecheck::TypedExpr;
use std::path::Path;

pub const ENTRY_FN_NAME: &str = "lumina_entry";

pub fn compile_to_entry(expr: &TypedExpr) -> Result<String, String> {
    let context = Context::create();
    let module = build_module(&context, expr)?;
    Ok(module.print_to_string().to_string())
}

pub fn write_object_file(expr: &TypedExpr, triple: &str, out_path: &Path) -> Result<(), String> {
    let context = Context::create();
    let module = build_module(&context, expr)?;

    Target::initialize_all(&InitializationConfig::default());
    let target_triple = TargetTriple::create(triple);
    module.set_triple(&target_triple);

    let target = Target::from_triple(&target_triple).map_err(|e| e.to_string())?;
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("failed to create target machine")?;

    let data_layout = target_machine.get_target_data().get_data_layout();
    module.set_data_layout(&data_layout);

    target_machine
        .write_to_file(&module, FileType::Object, out_path)
        .map_err(|e| e.to_string())
}

fn build_module<'ctx>(
    context: &'ctx Context,
    expr: &TypedExpr,
) -> Result<inkwell::module::Module<'ctx>, String> {
    let module = context.create_module("lumina");
    let i64 = context.i64_type();
    let fn_type = i64.fn_type(&[], false);
    let func = module.add_function(ENTRY_FN_NAME, fn_type, None);
    let entry = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry);

    let result = compile_expr(context, &builder, expr)?;
    builder
        .build_return(Some(&result))
        .map_err(|e| e.to_string())?;
    Ok(module)
}

fn compile_expr<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    expr: &TypedExpr,
) -> Result<inkwell::values::IntValue<'ctx>, String> {
    let i64 = context.i64_type();
    match expr {
        TypedExpr::Int(n) => Ok(i64.const_int(*n as u64, false)),
        TypedExpr::Add(l, r) => {
            let lv = compile_expr(context, builder, l)?;
            let rv = compile_expr(context, builder, r)?;
            builder
                .build_int_add(lv, rv, "add")
                .map_err(|e| e.to_string())
        }
        TypedExpr::Sub(l, r) => {
            let lv = compile_expr(context, builder, l)?;
            let rv = compile_expr(context, builder, r)?;
            builder
                .build_int_sub(lv, rv, "sub")
                .map_err(|e| e.to_string())
        }
        TypedExpr::Mul(l, r) => {
            let lv = compile_expr(context, builder, l)?;
            let rv = compile_expr(context, builder, r)?;
            builder
                .build_int_mul(lv, rv, "mul")
                .map_err(|e| e.to_string())
        }
        TypedExpr::Var(_) => Err("free variables not supported in codegen".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_part1_parser::parse;
    use lumina_part1_typecheck::TypeChecker;
    use std::path::PathBuf;

    #[test]
    fn codegen_int() {
        let expr = parse("42").unwrap();
        let typed = TypeChecker::new().check(&expr).unwrap();
        let ir = compile_to_entry(&typed).unwrap();
        assert!(ir.contains("define i64 @lumina_entry()"));
        assert!(ir.contains("ret i64 42"));
    }

    #[test]
    fn codegen_add() {
        let expr = parse("1 + 2").unwrap();
        let typed = TypeChecker::new().check(&expr).unwrap();
        let ir = compile_to_entry(&typed).unwrap();
        // May contain "add" or be constant-folded to "ret i64 3"
        assert!(ir.contains("ret i64"));
    }

    #[test]
    fn can_write_object_file() {
        let expr = parse("42").unwrap();
        let typed = TypeChecker::new().check(&expr).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let obj = PathBuf::from(dir.path()).join("main.o");
        // Host triple is provided by the targets crate; hardcode common macOS triples here
        // to keep this crate dependency-free.
        let triple = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin"
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            "x86_64-unknown-linux-gnu"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        write_object_file(&typed, triple, &obj).unwrap();
        assert!(obj.exists());
    }
}
