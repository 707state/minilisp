use minilisp::ast::*;
use minilisp::codegen::*;
use minilisp::object::*;
use minilisp::parser::parse_lisp;

fn main() {
    let src = "(add1 1)";
    match parse_lisp(src) {
        Ok((_, val)) => {
            let mut generator = ASMGenerator::new();
            generator.compile_function(&val);
            let _ = generator.init();
            generator.make_executable();
            let return_code = generator.execute();
            let result = generator.reclaim();
            assert_eq!(result, 0, "munmap failed");
            println!(
                "Program returned '{}'",
                object_decode_integer(return_code as i64)
            );
        }
        Err(_) => {
            panic!("Parsing failed!");
        }
    }
}
