use minilisp::ast::*;
use minilisp::codegen::*;
use minilisp::object::*;

fn main() {
    let mut generator = ASMGenerator::new();

    // let value = 'a' as i32;
    // let node = ast_new_char(value);

    // generator.compile_function(node);
    // let _ = generator.init();

    // let result = generator.make_executable();
    // assert_eq!(result, 0, "mprotect failed");

    // let return_code = generator.execute();
    // assert_eq!(
    //     'a' as u8,
    //     object_decode_char(return_code as i64),
    //     "the assembly was wrong"
    // );

    // let result = generator.reclaim();
    // assert_eq!(result, 0, "munmap failed");

    // println!(
    //     "Program returned '{}'",
    //     object_decode_char(return_code as i64) as u8 as char
    // );
}
