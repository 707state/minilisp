use std::io;

use minilisp::{
    ast::*,
    codegen::{ASMGenerator, CodeSink},
    common::*,
    dump::write_executable_aarch64,
    label::Labels,
    object::*,
    parser::parse_lisp,
    syscall::{SYSCALL_REGISTER, Syscall},
};

fn main() {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to readline");
        match parse_lisp(&input) {
            Ok((_, val)) => {
                let mut generator = ASMGenerator::new();
                generator.compile_function(&val);
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
                let mut main = ASMGenerator::new();
                main.gen_bl_instruction(0);
                // bl to real entry
                main.put_label_call(0, ENTRY_POINT.to_string());
                main.gen_mov_imm_instruction(SYSCALL_REGISTER, Syscall::EXIT.number(), 0, true);
                main.gen_svc_instruction(0);
                let mut labels = Labels::new();
                labels.append_label(&mut main);
                labels.append_label(&mut generator);
                labels.fixup();
                write_executable_aarch64(&labels.get_object_instructions(), "./repl.o", "repl");
                labels.print_asm();
            }
            Err(_) => {
                panic!("Parsing failed!");
            }
        }
    }
}
