use inkwell::context::Context;

fn main() {
    let context = Context::create();
    let module = context.create_module("lumina_check");
    let i64 = context.i64_type();
    let fn_type = i64.fn_type(&[], false);
    let _main = module.add_function("main", fn_type, None);
    println!("LLVM and Inkwell: OK");
}

#[cfg(test)]
mod tests {
    use inkwell::context::Context;

    #[test]
    fn inkwell_creates_module() {
        let context = Context::create();
        let module = context.create_module("test");
        assert!(module.get_name().to_str().unwrap() == "test");
    }
}
