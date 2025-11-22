use std::io;

use minilisp::{
    ast::{ast_is_bool, ast_is_char, ast_is_integer, ast_is_nil},
    codegen::ASMGenerator,
    object::{object_decode_bool, object_decode_char, object_decode_integer},
    parser::*,
};
fn main() {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to readline");
        let mut p = Parser::new(&input);
        let ast = p.parse_expr();
        let mut generator = ASMGenerator::new();
        generator.compile_function(ast);
        let _ = generator.init();
        generator.make_executable();
        let return_val = generator.execute();
        if ast_is_integer(return_val) {
            println!("Return value: {}", object_decode_integer(return_val as i64));
        } else if ast_is_bool(return_val) {
            println!("Return value: {}", object_decode_bool(return_val as i64));
        } else if ast_is_nil(return_val) {
            println!("Return value: nil");
        } else if ast_is_char(return_val) {
            println!(
                "Return value: {}",
                object_decode_char(return_val as i64) as char
            );
        } else {
            println!("Type of return value not supported to print currently!");
        }
        generator.print_instructions();
    }
}
