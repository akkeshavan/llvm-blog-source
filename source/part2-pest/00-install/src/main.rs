// Part 2: grammar-driven compiler (pest) - installation verification

use inkwell::context::Context;

fn main() {
    let context = Context::create();
    let _module = context.create_module("lumina_part2_check");
    println!("Part 2: LLVM and Inkwell OK");
}

#[cfg(test)]
mod tests {
    use inkwell::context::Context;

    #[test]
    fn inkwell_works() {
        let context = Context::create();
        let module = context.create_module("test");
        assert!(module.get_name().to_str().unwrap() == "test");
    }
}
