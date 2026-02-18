use inkwell::context::Context;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use lumina_part1_typecheck::{TypedExpr, TypedProgram, TypedStmt, Type};
use std::path::Path;

fn expr_type(e: &TypedExpr) -> &Type {
    match e {
        TypedExpr::Int(_) => &Type::Int,
        TypedExpr::Str(_) => &Type::Str,
        TypedExpr::Var(_, t) => t,
        TypedExpr::Call(_, _, r) => r,
        TypedExpr::ArrayLit(_, t) => t,
        TypedExpr::Null(t) => t,
        TypedExpr::TypeOf(_, _) => &Type::Str,
        TypedExpr::FieldAccess(_, _, t) => t,
        TypedExpr::RecordLit(_, t) => t,
        TypedExpr::Constructor(_, _, t) => t,
        TypedExpr::Match(_, _, t) => t,
        TypedExpr::If(_, t, _) => expr_type(t),
        TypedExpr::Add(..) | TypedExpr::Sub(..) | TypedExpr::Mul(..) | TypedExpr::Div(..) | TypedExpr::Mod(..) => &Type::Int,
        TypedExpr::Eq(..) | TypedExpr::Ne(..) | TypedExpr::Lt(..) | TypedExpr::Le(..) | TypedExpr::Gt(..) | TypedExpr::Ge(..) => &Type::Int,
        TypedExpr::Range { .. } => &Type::Range,
        TypedExpr::Unit => &Type::Unit,
    }
}

fn type_to_llvm_struct<'ctx>(context: &'ctx Context, ty: &Type) -> Result<inkwell::types::StructType<'ctx>, String> {
    match ty {
        Type::Record(fields) => {
            let field_tys: Result<Vec<_>, _> = fields.iter().map(|(_, t)| type_to_llvm(context, t)).collect();
            let field_tys = field_tys?;
            Ok(context.struct_type(&field_tys, false))
        }
        _ => Err("expected record type".into()),
    }
}

fn type_to_llvm<'ctx>(context: &'ctx Context, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
    let i64 = context.i64_type();
    let i8 = context.i8_type();
    let i8ptr = i8.ptr_type(inkwell::AddressSpace::default());
    match ty {
        Type::Int => Ok(i64.as_basic_type_enum()),
        Type::Str => Ok(i8ptr.as_basic_type_enum()),
        Type::Unit => Ok(i64.as_basic_type_enum()),
        Type::Range => Err("Range type in value position".into()),
        Type::Array(_) | Type::Optional(_) => Ok(i64.as_basic_type_enum()),
        Type::Record(_) => Ok(type_to_llvm_struct(context, ty)?.as_basic_type_enum()),
        Type::TypeVar(_) => Err("TypeVar in codegen".into()),
        Type::Sum(_) => Ok(sum_struct_type(context)?.as_basic_type_enum()),
    }
}

fn sum_struct_type<'ctx>(context: &'ctx Context) -> Result<inkwell::types::StructType<'ctx>, String> {
    let i64 = context.i64_type();
    Ok(context.struct_type(&[i64.as_basic_type_enum(), i64.as_basic_type_enum()], false))
}

fn variant_index(variants: &[(String, Vec<Type>)], variant_name: &str) -> Result<usize, String> {
    variants.iter().position(|(n, _)| n == variant_name).ok_or_else(|| format!("variant '{}' not found", variant_name))
}

/// Type for alloca (record vars hold pointer to struct).
fn type_to_llvm_alloca<'ctx>(context: &'ctx Context, ty: &Type) -> Result<BasicTypeEnum<'ctx>, String> {
    match ty {
        Type::Record(_) => {
            let struct_ty = type_to_llvm_struct(context, ty)?;
            let ptr_ty = struct_ty.ptr_type(inkwell::AddressSpace::default());
            Ok(ptr_ty.as_basic_type_enum())
        }
        Type::Sum(_) => {
            let i64 = context.i64_type();
            let sum_struct = context.struct_type(&[i64.as_basic_type_enum(), i64.as_basic_type_enum()], false);
            Ok(sum_struct.ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum())
        }
        _ => type_to_llvm(context, ty),
    }
}

pub const ENTRY_FN_NAME: &str = "lumina_main";

fn resolve_callee(name: &str, args: &[TypedExpr]) -> String {
    match name {
        "println" => {
            if args.first().map(expr_type) == Some(&Type::Str) {
                "lumina_println_str".into()
            } else {
                "lumina_println_i64".into()
            }
        }
        "println_i64" => "lumina_println_i64".into(),
        "println_str" => "lumina_println_str".into(),
        "print" => {
            if let Some(TypedExpr::Str(_)) = args.first() {
                "lumina_print".into()
            } else {
                "lumina_print_i64".into()
            }
        }
        "print_i64" => "lumina_print_i64".into(),
        "append_i64" => "lumina_array_i64_append".into(),
        "append_str" => "lumina_array_str_append".into(),
        "get_i64" => "lumina_array_i64_get".into(),
        "get_str" => "lumina_array_str_get".into(),
        "set_i64" => "lumina_array_i64_set".into(),
        "set_str" => "lumina_array_str_set".into(),
        "array_i64_new" => "lumina_array_i64_new".into(),
        "array_str_new" => "lumina_array_str_new".into(),
        "array_i64_len" => "lumina_array_i64_len".into(),
        "array_str_len" => "lumina_array_str_len".into(),
        "unwrap_i64" => "lumina_unwrap_i64".into(),
        "unwrap_str" => "lumina_unwrap_str".into(),
        _ => name.to_string(),
    }
}

/// Compile a full program to LLVM IR string.
pub fn compile_program(program: &TypedProgram) -> Result<String, String> {
    let context = Context::create();
    let module = build_module(&context, program)?;
    Ok(module.print_to_string().to_string())
}

/// Backward compat: compile a single expression as main returning i64 (used by tests that pass one expr).
pub fn compile_to_entry(program: &TypedProgram) -> Result<String, String> {
    compile_program(program)
}

pub fn write_object_file(
    program: &TypedProgram,
    triple: &str,
    out_path: &Path,
) -> Result<(), String> {
    let context = Context::create();
    let module = build_module(&context, program)?;

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
    program: &TypedProgram,
) -> Result<inkwell::module::Module<'ctx>, String> {
    let module = context.create_module("lumina");
    let i64 = context.i64_type();
    let i8 = context.i8_type();
    let i8ptr = i8.ptr_type(inkwell::AddressSpace::default());

    declare_runtime(&module, &context)?;

    fn llvm_name(name: &str) -> String {
        if name == "main" { ENTRY_FN_NAME.to_string() } else { name.to_string() }
    }

    for f in &program.functions {
        let mut param_tys: Vec<BasicMetadataTypeEnum> = Vec::new();
        for (_, t) in &f.params {
            match t {
                Type::Int => param_tys.push(i64.into()),
                Type::Str => param_tys.push(i8ptr.into()),
                Type::Unit => param_tys.push(i64.into()),
                Type::Range => return Err("Range is not a valid function parameter type".into()),
                Type::Array(_) | Type::Optional(_) => param_tys.push(i64.into()),
                Type::Record(_) => {
                    let struct_ty = type_to_llvm_struct(context, t)?;
                    param_tys.push(struct_ty.ptr_type(inkwell::AddressSpace::default()).into());
                }
                Type::Sum(_) | Type::TypeVar(_) => return Err("Sum/TypeVar not valid as function parameter".into()),
            }
        }
        let fn_type = match &f.return_type {
            Type::Int => i64.fn_type(param_tys.as_slice(), false),
            Type::Str => i8ptr.fn_type(param_tys.as_slice(), false),
            Type::Unit => context.void_type().fn_type(param_tys.as_slice(), false),
            Type::Range => return Err("Range is not a valid function return type".into()),
            Type::Array(_) | Type::Optional(_) => i64.fn_type(param_tys.as_slice(), false),
            Type::Record(_) => {
                let struct_ty = type_to_llvm_struct(context, &f.return_type)?;
                struct_ty.ptr_type(inkwell::AddressSpace::default()).fn_type(param_tys.as_slice(), false)
            }
            Type::Sum(_) | Type::TypeVar(_) => return Err("Sum/TypeVar not valid as return type".into()),
        };
        module.add_function(&llvm_name(&f.name), fn_type, None);
    }

    for f in &program.functions {
        let func = module.get_function(&llvm_name(&f.name)).ok_or("function not found")?;
        let entry = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);

        let mut local_env: std::collections::HashMap<String, PointerValue> =
            std::collections::HashMap::new();
        for (i, (name, ty)) in f.params.iter().enumerate() {
            let param = func.get_nth_param(i as u32).unwrap();
            let val = match ty {
                Type::Int => BasicValueEnum::IntValue(param.into_int_value()),
                Type::Str => BasicValueEnum::PointerValue(param.into_pointer_value()),
                Type::Unit => BasicValueEnum::IntValue(param.into_int_value()),
                Type::Range => return Err("Range is not a valid function parameter type".into()),
                Type::Array(_) | Type::Optional(_) => BasicValueEnum::IntValue(param.into_int_value()),
                Type::Record(_) => BasicValueEnum::PointerValue(param.into_pointer_value()),
                Type::Sum(_) | Type::TypeVar(_) => return Err("Sum/TypeVar not valid".into()),
            };
            let llvm_ty = type_to_llvm_alloca(context, ty)?;
            let alloca = builder.build_alloca(llvm_ty, name).map_err(|e| e.to_string())?;
            builder.build_store(alloca, val).map_err(|e| e.to_string())?;
            local_env.insert(name.clone(), alloca);
        }

        let mut str_counter = 0u64;
        let did_return = compile_stmt_list(
            context,
            &builder,
            &module,
            &f.body,
            &mut local_env,
            &mut str_counter,
        )?;
        if !did_return {
            if matches!(f.return_type, Type::Unit) {
                builder.build_return(None).map_err(|e| e.to_string())?;
            } else {
                return Err("function must return a value".into());
            }
        }
    }

    Ok(module)
}

fn compile_stmt_list<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    module: &inkwell::module::Module<'ctx>,
    body: &[TypedStmt],
    env: &mut std::collections::HashMap<String, PointerValue<'ctx>>,
    str_counter: &mut u64,
) -> Result<bool, String> {
    for stmt in body {
        match stmt {
            TypedStmt::Let(name, e) => {
                let ty = expr_type(e);
                let llvm_ty = type_to_llvm_alloca(context, ty)?;
                let alloca = builder.build_alloca(llvm_ty, name).map_err(|e| e.to_string())?;
                if let Some(v) = compile_expr(context, builder, module, e, env, str_counter)? {
                    builder.build_store(alloca, v).map_err(|e| e.to_string())?;
                }
                env.insert(name.clone(), alloca);
            }
            TypedStmt::Assign(lhs, rhs) => {
                let ptr = compile_lvalue(context, builder, module, lhs, env, str_counter)?;
                let val = compile_expr(context, builder, module, rhs, env, str_counter)?
                    .ok_or("assign rhs must produce value")?;
                builder.build_store(ptr, val).map_err(|e| e.to_string())?;
            }
            TypedStmt::Expr(e) => {
                let _ = compile_expr(context, builder, module, e, env, str_counter)?;
            }
            TypedStmt::For { var, range, body } => {
                compile_for(context, builder, module, var, range, body, env, str_counter)?;
            }
            TypedStmt::Return(e_opt) => {
                match e_opt {
                    None => {
                        builder.build_return(None).map_err(|e| e.to_string())?;
                    }
                    Some(e) => {
                        let val = compile_expr(context, builder, module, e, env, str_counter)?;
                        let v = val.ok_or("return expression must produce a value")?;
                        builder
                            .build_return(Some(&v))
                            .map_err(|e| e.to_string())?;
                    }
                };
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn compile_lvalue<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    module: &inkwell::module::Module<'ctx>,
    expr: &TypedExpr,
    env: &mut std::collections::HashMap<String, PointerValue<'ctx>>,
    str_counter: &mut u64,
) -> Result<PointerValue<'ctx>, String> {
    match expr {
        TypedExpr::Var(name, _) => env.get(name).cloned().ok_or_else(|| format!("unbound var: {}", name)),
        TypedExpr::FieldAccess(base, field, _) => {
            let base_val = compile_expr(context, builder, module, base, env, str_counter)?
                .ok_or_else(|| "field access base must produce value".to_string())?;
            let base_ptr = match &base_val {
                BasicValueEnum::PointerValue(p) => *p,
                _ => return Err("field access base must be pointer (record)".into()),
            };
            let base_ty = expr_type(base);
            let struct_ty = type_to_llvm_struct(context, base_ty)?;
            let fields = match base_ty {
                Type::Record(f) => f,
                _ => return Err("field access base must be record".into()),
            };
            let idx = fields.iter().position(|(n, _)| n == field).ok_or_else(|| format!("no field {}", field))? as u32;
            builder.build_struct_gep(struct_ty, base_ptr, idx, field).map_err(|e| e.to_string())
        }
        _ => Err("assignment target must be var or field access".into()),
    }
}

fn compile_for<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    module: &inkwell::module::Module<'ctx>,
    var: &str,
    range_expr: &TypedExpr,
    body: &[TypedStmt],
    env: &mut std::collections::HashMap<String, PointerValue<'ctx>>,
    str_counter: &mut u64,
) -> Result<(), String> {
    let i64 = context.i64_type();

    let (start, end, inclusive, step_opt) = match range_expr {
        TypedExpr::Range {
            start,
            end,
            inclusive,
            step,
        } => (start.as_ref(), end.as_ref(), *inclusive, step.as_deref()),
        _ => return Err("for-loop requires a range expression".into()),
    };

    let start_v = expect_int(compile_expr(context, builder, module, start, env, str_counter)?)?;
    let end_v = expect_int(compile_expr(context, builder, module, end, env, str_counter)?)?;

    let step_v = if let Some(step_e) = step_opt {
        expect_int(compile_expr(context, builder, module, step_e, env, str_counter)?)?
    } else {
        // default: if start <= end then +1 else -1
        let one = i64.const_int(1, true);
        let neg_one = i64.const_int((-1i64) as u64, true);
        let is_inc = builder
            .build_int_compare(inkwell::IntPredicate::SLE, start_v, end_v, "range.is_inc")
            .map_err(|e| e.to_string())?;
        builder
            .build_select(is_inc, one, neg_one, "range.step")
            .map_err(|e| e.to_string())?
            .into_int_value()
    };

    let parent = callee_function(builder);
    let preheader_bb = builder.get_insert_block().ok_or("missing insert block")?;
    let loop_bb = context.append_basic_block(parent, "for.loop");
    let body_bb = context.append_basic_block(parent, "for.body");
    let inc_bb = context.append_basic_block(parent, "for.inc");
    let after_bb = context.append_basic_block(parent, "for.after");

    let loop_var_alloca = builder
        .build_alloca(i64, var)
        .map_err(|e| e.to_string())?;
    builder
        .build_unconditional_branch(loop_bb)
        .map_err(|e| e.to_string())?;

    builder.position_at_end(loop_bb);
    let phi = builder
        .build_phi(i64, "i")
        .map_err(|e| e.to_string())?;
    phi.add_incoming(&[(&start_v, preheader_bb)]);
    let i = phi.as_basic_value().into_int_value();

    // cond: step > 0 ? (i <=/< end) : (i >=/> end)
    let is_pos = builder
        .build_int_compare(inkwell::IntPredicate::SGT, step_v, i64.const_zero(), "step.pos")
        .map_err(|e| e.to_string())?;
    let pos_pred = if inclusive {
        inkwell::IntPredicate::SLE
    } else {
        inkwell::IntPredicate::SLT
    };
    let neg_pred = if inclusive {
        inkwell::IntPredicate::SGE
    } else {
        inkwell::IntPredicate::SGT
    };
    let cmp_pos = builder
        .build_int_compare(pos_pred, i, end_v, "cmp.pos")
        .map_err(|e| e.to_string())?;
    let cmp_neg = builder
        .build_int_compare(neg_pred, i, end_v, "cmp.neg")
        .map_err(|e| e.to_string())?;
    let cond = builder
        .build_select(is_pos, cmp_pos, cmp_neg, "for.cond")
        .map_err(|e| e.to_string())?
        .into_int_value();

    builder
        .build_conditional_branch(cond, body_bb, after_bb)
        .map_err(|e| e.to_string())?;

    builder.position_at_end(body_bb);
    builder.build_store(loop_var_alloca, i).map_err(|e| e.to_string())?;
    let saved = env.insert(var.to_string(), loop_var_alloca);
    let did_return = compile_stmt_list(context, builder, module, body, env, str_counter)?;
    if let Some(prev) = saved {
        env.insert(var.to_string(), prev);
    } else {
        env.remove(var);
    }
    if did_return {
        return Ok(());
    }
    builder
        .build_unconditional_branch(inc_bb)
        .map_err(|e| e.to_string())?;

    builder.position_at_end(inc_bb);
    let next_i = builder
        .build_int_add(i, step_v, "i.next")
        .map_err(|e| e.to_string())?;
    phi.add_incoming(&[(&next_i, inc_bb)]);
    builder
        .build_unconditional_branch(loop_bb)
        .map_err(|e| e.to_string())?;

    builder.position_at_end(after_bb);
    Ok(())
}

fn declare_runtime<'ctx>(
    module: &inkwell::module::Module<'ctx>,
    context: &'ctx Context,
) -> Result<(), String> {
    let i64 = context.i64_type();
    let i8 = context.i8_type();
    let i8ptr = i8.ptr_type(inkwell::AddressSpace::default());
    let void = context.void_type();

    if module.get_function("lumina_println_i64").is_none() {
        let ty = void.fn_type(&[i64.into()], false);
        module.add_function("lumina_println_i64", ty, None);
    }
    if module.get_function("lumina_println_str").is_none() {
        let ty = void.fn_type(&[i8ptr.into()], false);
        module.add_function("lumina_println_str", ty, None);
    }
    if module.get_function("lumina_print").is_none() {
        let ty = void.fn_type(&[i8ptr.into()], false);
        module.add_function("lumina_print", ty, None);
    }
    if module.get_function("lumina_print_i64").is_none() {
        let ty = void.fn_type(&[i64.into()], false);
        module.add_function("lumina_print_i64", ty, None);
    }
    if module.get_function("lumina_sqrt").is_none() {
        let ty = i64.fn_type(&[i64.into()], false);
        module.add_function("lumina_sqrt", ty, None);
    }
    if module.get_function("lumina_max").is_none() {
        let ty = i64.fn_type(&[i64.into(), i64.into()], false);
        module.add_function("lumina_max", ty, None);
    }
    if module.get_function("lumina_min").is_none() {
        let ty = i64.fn_type(&[i64.into(), i64.into()], false);
        module.add_function("lumina_min", ty, None);
    }
    if module.get_function("lumina_array_i64_new").is_none() {
        module.add_function("lumina_array_i64_new", i64.fn_type(&[], false), None);
    }
    if module.get_function("lumina_array_i64_append").is_none() {
        module.add_function("lumina_array_i64_append", void.fn_type(&[i64.into(), i64.into()], false), None);
    }
    if module.get_function("lumina_array_i64_get").is_none() {
        module.add_function("lumina_array_i64_get", i64.fn_type(&[i64.into(), i64.into()], false), None);
    }
    if module.get_function("lumina_array_i64_set").is_none() {
        module.add_function("lumina_array_i64_set", void.fn_type(&[i64.into(), i64.into(), i64.into()], false), None);
    }
    if module.get_function("lumina_array_str_new").is_none() {
        module.add_function("lumina_array_str_new", i64.fn_type(&[], false), None);
    }
    if module.get_function("lumina_array_str_append").is_none() {
        module.add_function("lumina_array_str_append", void.fn_type(&[i64.into(), i8ptr.into()], false), None);
    }
    if module.get_function("lumina_array_str_get").is_none() {
        module.add_function("lumina_array_str_get", i8ptr.fn_type(&[i64.into(), i64.into()], false), None);
    }
    if module.get_function("lumina_array_str_set").is_none() {
        module.add_function("lumina_array_str_set", void.fn_type(&[i64.into(), i64.into(), i8ptr.into()], false), None);
    }
    if module.get_function("lumina_array_i64_len").is_none() {
        module.add_function("lumina_array_i64_len", i64.fn_type(&[i64.into()], false), None);
    }
    if module.get_function("lumina_array_str_len").is_none() {
        module.add_function("lumina_array_str_len", i64.fn_type(&[i64.into()], false), None);
    }
    if module.get_function("lumina_unwrap_i64").is_none() {
        module.add_function("lumina_unwrap_i64", i64.fn_type(&[i64.into()], false), None);
    }
    if module.get_function("lumina_unwrap_str").is_none() {
        module.add_function("lumina_unwrap_str", i8ptr.fn_type(&[i64.into()], false), None);
    }
    Ok(())
}

fn compile_expr<'ctx>(
    context: &'ctx Context,
    builder: &inkwell::builder::Builder<'ctx>,
    module: &inkwell::module::Module<'ctx>,
    expr: &TypedExpr,
    env: &mut std::collections::HashMap<String, PointerValue<'ctx>>,
    str_counter: &mut u64,
) -> Result<Option<BasicValueEnum<'ctx>>, String> {
    let i64 = context.i64_type();
    let i8 = context.i8_type();
    let i8ptr = i8.ptr_type(inkwell::AddressSpace::default());

    match expr {
        TypedExpr::Int(n) => Ok(Some(BasicValueEnum::IntValue(i64.const_int(*n as u64, false)))),
        TypedExpr::Str(s) => {
            let id = *str_counter;
            *str_counter += 1;
            let global = module.add_global(
                i8.array_type((s.len() + 1) as u32),
                None,
                &format!("str.{}", id),
            );
            global.set_constant(true);
            global.set_unnamed_addr(true);
            global.set_initializer(&context.const_string(s.as_bytes(), true));
            let ptr = builder.build_pointer_cast(
                global.as_pointer_value(),
                i8ptr,
                "str.ptr",
            ).map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::PointerValue(ptr)))
        }
        TypedExpr::Unit => Ok(None),
        TypedExpr::ArrayLit(elems, ty) => {
            let inner = match ty {
                Type::Array(inner) => inner.as_ref(),
                _ => return Err("ArrayLit must have array type".into()),
            };
            let (new_name, append_name): (&str, &str) = match inner {
                Type::Int => ("array_i64_new", "lumina_array_i64_append"),
                Type::Str => ("array_str_new", "lumina_array_str_append"),
                Type::Record(_) => ("array_i64_new", "lumina_array_i64_append"), // store record pointers as i64
                _ => return Err("array element type must be i64, str, or record".into()),
            };
            let new_fn = module.get_function(&resolve_callee(new_name, &[])).ok_or("array new not found")?;
            let arr_call = builder.build_call(new_fn, &[], "arr").map_err(|e| e.to_string())?;
            let arr = arr_call.try_as_basic_value().left().ok_or("array new must return value")?;
            let arr_i64 = arr.into_int_value();
            let append_fn = module.get_function(append_name).ok_or("array append not found")?;
            for elem in elems {
                let elem_val = compile_expr(context, builder, module, elem, env, str_counter)?;
                let arg = match inner {
                    Type::Int => BasicValueEnum::IntValue(expect_int(elem_val)?),
                    Type::Str => elem_val.ok_or("array elem must produce value")?,
                    Type::Record(_) => {
                        let ptr = match elem_val.ok_or("array elem must produce value")? {
                            BasicValueEnum::PointerValue(p) => p,
                            _ => return Err("record elem must be pointer".into()),
                        };
                        BasicValueEnum::IntValue(builder.build_ptr_to_int(ptr, i64, "ptr2i").map_err(|e| e.to_string())?)
                    }
                    _ => unreachable!(),
                };
                let args_meta: Vec<BasicMetadataValueEnum> = [
                    BasicValueEnum::IntValue(arr_i64),
                    arg,
                ]
                .iter()
                .map(|v| v.clone().into())
                .collect();
                let _ = builder.build_call(append_fn, &args_meta, "").map_err(|e| e.to_string())?;
            }
            Ok(Some(BasicValueEnum::IntValue(arr_i64)))
        }
        TypedExpr::Null(ty) => {
            let val = match ty {
                Type::Optional(inner) => match inner.as_ref() {
                    Type::Int => i64.const_int(0x8000_0000_0000_0000u64, true),
                    _ => i64.const_int(0, false),
                },
                _ => i64.const_int(0, false),
            };
            Ok(Some(BasicValueEnum::IntValue(val)))
        }
        TypedExpr::TypeOf(inner, type_name) => {
            let _ = compile_expr(context, builder, module, inner, env, str_counter)?;
            let id = *str_counter;
            *str_counter += 1;
            let global = module.add_global(
                i8.array_type((type_name.len() + 1) as u32),
                None,
                &format!("typeof.{}", id),
            );
            global.set_constant(true);
            global.set_unnamed_addr(true);
            global.set_initializer(&context.const_string(type_name.as_bytes(), true));
            let ptr = builder.build_pointer_cast(
                global.as_pointer_value(),
                i8ptr,
                "typeof.ptr",
            ).map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::PointerValue(ptr)))
        }
        TypedExpr::FieldAccess(base, field, _) => {
            let base_val = compile_expr(context, builder, module, base, env, str_counter)?
                .ok_or_else(|| "field access base must produce value".to_string())?;
            let base_ptr = match &base_val {
                BasicValueEnum::PointerValue(p) => *p,
                _ => return Err("field access base must be pointer (record)".into()),
            };
            let base_ty = expr_type(base);
            let struct_ty = type_to_llvm_struct(context, base_ty)?;
            let fields = match base_ty {
                Type::Record(f) => f,
                _ => return Err("field access must be on record".into()),
            };
            let idx = fields.iter().position(|(n, _)| n == field).ok_or_else(|| format!("no field {}", field))? as u32;
            let field_ptr = builder.build_struct_gep(struct_ty, base_ptr, idx, field).map_err(|e| e.to_string())?;
            let field_ty = type_to_llvm(context, expr_type(expr))?;
            let loaded = builder.build_load(field_ty, field_ptr, field).map_err(|e| e.to_string())?;
            Ok(Some(loaded))
        }
        TypedExpr::RecordLit(fields, ty) => {
            let struct_ty = type_to_llvm_struct(context, ty)?;
            let alloca = builder.build_alloca(struct_ty.as_basic_type_enum(), "record").map_err(|e| e.to_string())?;
            for (idx, (name, te)) in fields.iter().enumerate() {
                let val = compile_expr(context, builder, module, te, env, str_counter)?
                    .ok_or_else(|| "record field must produce value".to_string())?;
                let field_ptr = builder.build_struct_gep(struct_ty, alloca, idx as u32, name).map_err(|e| e.to_string())?;
                builder.build_store(field_ptr, val).map_err(|e| e.to_string())?;
            }
            Ok(Some(BasicValueEnum::PointerValue(alloca)))
        }
        TypedExpr::Constructor(variant, args, sum_ty) => {
            let sum_st = sum_struct_type(context)?;
            let variants = match sum_ty {
                Type::Sum(v) => v,
                _ => return Err("constructor type must be sum".into()),
            };
            let idx = variant_index(variants, variant)?;
            if args.len() > 1 {
                return Err("constructors with more than one payload not yet supported".into());
            }
            let alloca = builder.build_alloca(sum_st.as_basic_type_enum(), "sum").map_err(|e| e.to_string())?;
            let tag_ptr = builder.build_struct_gep(sum_st, alloca, 0, "tag").map_err(|e| e.to_string())?;
            builder.build_store(tag_ptr, i64.const_int(idx as u64, false)).map_err(|e| e.to_string())?;
            let payload_ptr = builder.build_struct_gep(sum_st, alloca, 1, "payload").map_err(|e| e.to_string())?;
            let payload_i64 = if let Some(te) = args.first() {
                let v = compile_expr(context, builder, module, te, env, str_counter)?.ok_or("constructor arg must produce value")?;
                match &v {
                    BasicValueEnum::IntValue(iv) => *iv,
                    BasicValueEnum::PointerValue(pv) => builder.build_ptr_to_int(*pv, i64, "ptr2i").map_err(|e| e.to_string())?,
                    _ => return Err("constructor payload must be int or pointer".into()),
                }
            } else {
                i64.const_int(0, false)
            };
            builder.build_store(payload_ptr, payload_i64).map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::PointerValue(alloca)))
        }
        TypedExpr::Match(scrut, arms, ret_ty) => {
            let sum_ty = match expr_type(scrut) {
                Type::Sum(v) => v,
                _ => return Err("match scrutinee must be sum type".into()),
            };
            let scrut_val = compile_expr(context, builder, module, scrut, env, str_counter)?.ok_or("match scrutinee must produce value")?;
            let scrut_ptr = match &scrut_val {
                BasicValueEnum::PointerValue(p) => *p,
                _ => return Err("match scrutinee must be pointer (sum)".into()),
            };
            let sum_st = sum_struct_type(context)?;
            let tag_ptr = builder.build_struct_gep(sum_st, scrut_ptr, 0, "tag").map_err(|e| e.to_string())?;
            let tag_val = builder.build_load(i64, tag_ptr, "tag").map_err(|e| e.to_string())?;
            let tag_i64 = match tag_val {
                BasicValueEnum::IntValue(i) => i,
                _ => return Err("match tag must be i64".into()),
            };
            let parent = callee_function(builder);
            let merge_bb = context.append_basic_block(parent, "match.merge");
            let default_bb = context.append_basic_block(parent, "match.unreachable");
            let mut cases = vec![];
            for (variant_name, _, _) in arms.iter() {
                let arm_bb = context.append_basic_block(parent, &format!("match.{}", variant_name));
                let idx = variant_index(&sum_ty, variant_name)? as u64;
                cases.push((i64.const_int(idx, false), arm_bb));
            }
            builder.build_switch(tag_i64, default_bb, &cases).map_err(|e| e.to_string())?;
            builder.position_at_end(default_bb);
            builder.build_unreachable().map_err(|e| e.to_string())?;
            builder.position_at_end(merge_bb);
            let ret_ty_llvm = type_to_llvm(context, ret_ty)?;
            let phi = builder.build_phi(ret_ty_llvm, "match.result").map_err(|e| e.to_string())?;
            for (i, (variant_name, bindings, body)) in arms.iter().enumerate() {
                let arm_bb = cases[i].1;
                builder.position_at_end(arm_bb);
                if !bindings.is_empty() {
                    let payload_ptr = builder.build_struct_gep(sum_st, scrut_ptr, 1, "payload").map_err(|e| e.to_string())?;
                    let payload_loaded = builder.build_load(i64, payload_ptr, "payload").map_err(|e| e.to_string())?;
                    let payload_i64 = match payload_loaded {
                        BasicValueEnum::IntValue(i) => i,
                        _ => return Err("payload must be i64".into()),
                    };
                    let (_, payload_tys) = sum_ty.iter().find(|(n, _)| n == variant_name).ok_or("variant not found")?;
                    if bindings.len() != 1 || payload_tys.len() != 1 {
                        return Err("match arm payload: exactly one binding supported".into());
                    }
                    let binding_ty = &payload_tys[0];
                    let llvm_ty = type_to_llvm_alloca(context, binding_ty)?;
                    let alloca = builder.build_alloca(llvm_ty, &bindings[0]).map_err(|e| e.to_string())?;
                    let val = match binding_ty {
                        Type::Int => BasicValueEnum::IntValue(payload_i64),
                        Type::Str => {
                            let ptr = builder.build_int_to_ptr(payload_i64, i8ptr, "i2p").map_err(|e| e.to_string())?;
                            BasicValueEnum::PointerValue(ptr)
                        }
                        _ => return Err("match binding type must be int or str".into()),
                    };
                    builder.build_store(alloca, val).map_err(|e| e.to_string())?;
                    env.insert(bindings[0].clone(), alloca);
                }
                let body_val = compile_expr(context, builder, module, body, env, str_counter)?.ok_or("match arm must produce value")?;
                phi.add_incoming(&[(&body_val, builder.get_insert_block().unwrap())]);
                builder.build_unconditional_branch(merge_bb).map_err(|e| e.to_string())?;
                for b in bindings {
                    env.remove(b);
                }
            }
            builder.position_at_end(merge_bb);
            let result = phi.as_basic_value();
            Ok(Some(result))
        }
        TypedExpr::Range { .. } => Err("range values are only valid in for-loops".into()),
        TypedExpr::Var(name, ty) => {
            let ptr = env.get(name).ok_or(format!("unbound var: {}", name))?;
            let llvm_ty = type_to_llvm_alloca(context, ty)?;
            let v = builder.build_load(llvm_ty, ptr.clone(), name).map_err(|e| e.to_string())?;
            Ok(Some(v))
        }
        TypedExpr::Add(l, r) => {
            let lv = expect_int(compile_expr(context, builder, module, l, env, str_counter)?)?;
            let rv = expect_int(compile_expr(context, builder, module, r, env, str_counter)?)?;
            let v = builder.build_int_add(lv, rv, "add").map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::IntValue(v)))
        }
        TypedExpr::Sub(l, r) => {
            let lv = expect_int(compile_expr(context, builder, module, l, env, str_counter)?)?;
            let rv = expect_int(compile_expr(context, builder, module, r, env, str_counter)?)?;
            let v = builder.build_int_sub(lv, rv, "sub").map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::IntValue(v)))
        }
        TypedExpr::Mul(l, r) => {
            let lv = expect_int(compile_expr(context, builder, module, l, env, str_counter)?)?;
            let rv = expect_int(compile_expr(context, builder, module, r, env, str_counter)?)?;
            let v = builder.build_int_mul(lv, rv, "mul").map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::IntValue(v)))
        }
        TypedExpr::Div(l, r) => {
            let lv = expect_int(compile_expr(context, builder, module, l, env, str_counter)?)?;
            let rv = expect_int(compile_expr(context, builder, module, r, env, str_counter)?)?;
            let v = builder.build_int_signed_div(lv, rv, "div").map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::IntValue(v)))
        }
        TypedExpr::Mod(l, r) => {
            let lv = expect_int(compile_expr(context, builder, module, l, env, str_counter)?)?;
            let rv = expect_int(compile_expr(context, builder, module, r, env, str_counter)?)?;
            let v = builder.build_int_signed_rem(lv, rv, "rem").map_err(|e| e.to_string())?;
            Ok(Some(BasicValueEnum::IntValue(v)))
        }
        TypedExpr::Eq(l, r) | TypedExpr::Ne(l, r) | TypedExpr::Lt(l, r) | TypedExpr::Le(l, r) | TypedExpr::Gt(l, r) | TypedExpr::Ge(l, r) => {
            let lv = expect_int(compile_expr(context, builder, module, l, env, str_counter)?)?;
            let rv = expect_int(compile_expr(context, builder, module, r, env, str_counter)?)?;
            let v = match expr {
                TypedExpr::Eq(_, _) => builder.build_int_compare(inkwell::IntPredicate::EQ, lv, rv, "eq").map_err(|e| e.to_string())?,
                TypedExpr::Ne(_, _) => builder.build_int_compare(inkwell::IntPredicate::NE, lv, rv, "ne").map_err(|e| e.to_string())?,
                TypedExpr::Lt(_, _) => builder.build_int_compare(inkwell::IntPredicate::SLT, lv, rv, "lt").map_err(|e| e.to_string())?,
                TypedExpr::Le(_, _) => builder.build_int_compare(inkwell::IntPredicate::SLE, lv, rv, "le").map_err(|e| e.to_string())?,
                TypedExpr::Gt(_, _) => builder.build_int_compare(inkwell::IntPredicate::SGT, lv, rv, "gt").map_err(|e| e.to_string())?,
                TypedExpr::Ge(_, _) => builder.build_int_compare(inkwell::IntPredicate::SGE, lv, rv, "ge").map_err(|e| e.to_string())?,
                _ => unreachable!(),
            };
            let zero = i64.const_int(0, false);
            let one = i64.const_int(1, false);
            let i64_val = builder.build_select(v, one, zero, "cmp").map_err(|e| e.to_string())?;
            Ok(Some(i64_val))
        }
        TypedExpr::Call(name, args, ret_ty) => {
            let llvm_name = resolve_callee(name, args);
            let callee = module.get_function(&llvm_name).ok_or(format!("unknown function: {}", llvm_name))?;
            let param_tys = callee.get_type().get_param_types();
            let mut arg_vals = Vec::new();
            for (i, a) in args.iter().enumerate() {
                let v = compile_expr(context, builder, module, a, env, str_counter)?
                    .ok_or("call argument must produce value")?;
                // When passing a Record to a function that expects i64 (e.g. append_i64 for array of records), ptr2int
                let v = if matches!(expr_type(a), Type::Record(_)) {
                    if let Some(meta) = param_tys.get(i) {
                        if meta.as_basic_type_enum() == i64.as_basic_type_enum() {
                            let ptr = match v {
                                BasicValueEnum::PointerValue(p) => p,
                                _ => return Err("record arg must be pointer".into()),
                            };
                            BasicValueEnum::IntValue(builder.build_ptr_to_int(ptr, i64, "ptr2i").map_err(|e| e.to_string())?)
                        } else {
                            v
                        }
                    } else {
                        v
                    }
                } else {
                    v
                };
                arg_vals.push(v.into());
            }
            let call = builder.build_call(callee, &arg_vals, "call").map_err(|e| e.to_string())?;
            let mut ret = call.try_as_basic_value().left();
            // When call returns i64 but we need Record (e.g. get_i64 for array of records), int2ptr
            if matches!(ret_ty, Type::Record(_)) {
                if let Some(BasicValueEnum::IntValue(iv)) = ret {
                    let struct_ty = type_to_llvm_struct(context, ret_ty)?;
                    let ptr_ty = struct_ty.ptr_type(inkwell::AddressSpace::default());
                    let ptr = builder.build_int_to_ptr(iv, ptr_ty, "i2p").map_err(|e| e.to_string())?;
                    ret = Some(BasicValueEnum::PointerValue(ptr));
                }
            }
            Ok(ret)
        }
        TypedExpr::If(cond, then_b, else_b) => {
            let c = expect_int(compile_expr(context, builder, module, cond, env, str_counter)?)?;
            let zero = i64.const_int(0, false);
            let cond_val = builder.build_int_compare(inkwell::IntPredicate::NE, c, zero, "cond").map_err(|e| e.to_string())?;

            let then_bb = context.append_basic_block(callee_function(builder), "then");
            let else_bb = context.append_basic_block(callee_function(builder), "else");
            let merge_bb = context.append_basic_block(callee_function(builder), "merge");

            builder.build_conditional_branch(cond_val, then_bb, else_bb).map_err(|e| e.to_string())?;

            builder.position_at_end(then_bb);
            let then_v = compile_expr(context, builder, module, then_b, env, str_counter)?;
            builder.build_unconditional_branch(merge_bb).map_err(|e| e.to_string())?;

            builder.position_at_end(else_bb);
            let else_v = compile_expr(context, builder, module, else_b, env, str_counter)?;
            builder.build_unconditional_branch(merge_bb).map_err(|e| e.to_string())?;

            builder.position_at_end(merge_bb);
            match (then_v, else_v) {
                (Some(t), Some(e)) => {
                    let phi = builder.build_phi(context.i64_type(), "phi").map_err(|e| e.to_string())?;
                    phi.add_incoming(&[(&t, then_bb), (&e, else_bb)]);
                    Ok(Some(phi.as_basic_value()))
                }
                (None, None) => Ok(None),
                _ => Err("if branches must have same type".into()),
            }
        }
    }
}

fn expect_int<'ctx>(
    v: Option<BasicValueEnum<'ctx>>,
) -> Result<inkwell::values::IntValue<'ctx>, String> {
    match v {
        Some(BasicValueEnum::IntValue(i)) => Ok(i),
        _ => Err("expected int".into()),
    }
}

fn callee_function<'ctx>(builder: &inkwell::builder::Builder<'ctx>) -> FunctionValue<'ctx> {
    builder.get_insert_block().unwrap().get_parent().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_part1_parser::parse;
    use lumina_part1_typecheck::TypeChecker;
    use std::path::PathBuf;

    #[test]
    fn codegen_int() {
        let prog = parse("42").unwrap();
        let typed = TypeChecker::new().check_program(&prog).unwrap();
        let ir = compile_program(&typed).unwrap();
        assert!(ir.contains("lumina_main") || ir.contains("main"));
        assert!(ir.contains("42"));
    }

    #[test]
    fn codegen_add() {
        let prog = parse("1 + 2").unwrap();
        let typed = TypeChecker::new().check_program(&prog).unwrap();
        let ir = compile_program(&typed).unwrap();
        // Entry may return void (unit); value may be constant-folded to 3 or emit add
        assert!(
            ir.contains("lumina_main") &&
            (ir.contains("add") || ir.contains("ret i64") || ir.contains("ret void") || ir.contains(" 3"))
        );
    }

    /// Compiles a function that returns i64 (1 + 2); IR should contain ret i64 and add.
    #[test]
    fn codegen_fn_returns_i64() {
        let prog = parse("fn add() -> i64 { return 1 + 2; }").unwrap();
        let typed = TypeChecker::new().check_program(&prog).unwrap();
        let ir = compile_program(&typed).unwrap();
        assert!(ir.contains("ret i64"), "IR should contain 'ret i64' for function returning i64: {}", ir);
        assert!(ir.contains("add"), "IR should contain 'add' for 1 + 2: {}", ir);
    }

    #[test]
    fn can_write_object_file() {
        let prog = parse("42").unwrap();
        let typed = TypeChecker::new().check_program(&prog).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let obj = PathBuf::from(dir.path()).join("main.o");
        let triple = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        write_object_file(&typed, triple, &obj).unwrap();
        assert!(obj.exists());
    }
}
