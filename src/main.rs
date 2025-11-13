mod ast;
mod codegen;
mod common;
mod object;
use crate::ast::*;
use crate::codegen::*;
use crate::object::*;

fn main() {
    let mut generator = ASMGenerator::new();

    let value = 'a' as i32;
    let node = AST_new_char(value);

    generator.compile_function(node);
    generator.init();

    let result = generator.make_executable();
    assert_eq!(result, 0, "mprotect failed");

    let return_code = generator.execute();
    assert_eq!(
        'a' as i8,
        Object_decode_char(return_code as i64),
        "the assembly was wrong"
    );

    let result = generator.reclaim();
    assert_eq!(result, 0, "munmap failed");

    println!(
        "Program returned '{}'",
        Object_decode_char(return_code as i64) as u8 as char
    );
}
