use std::collections::HashMap;

use crate::{
    codegen::ASMGenerator,
    common::{B_COND, BL, calc_b_imm26, calc_bcond_imm19},
};

pub struct Labels {
    global_label_offset: HashMap<String, u32>,
    global_label_call: HashMap<u32, String>,
    global_index: u32,
    object_instructions: Vec<u32>,
}

impl Labels {
    pub fn new() -> Labels {
        Self {
            global_label_offset: HashMap::new(),
            global_label_call: HashMap::new(),
            global_index: 0,
            object_instructions: Vec::new(),
        }
    }
    pub fn append_label(&mut self, generator: &mut ASMGenerator) {
        // adjust offset for specific generator
        generator.adjust_offset(self.global_index);
        // store all label_offset
        for (k, v) in generator.get_label_offset() {
            self.global_label_offset.insert(k, v);
        }
        // store all label being called.
        for (k, v) in generator.get_label_call() {
            self.global_label_call.insert(k, v);
        }
        // store all instructions?
        self.object_instructions
            .append(&mut generator.get_instructions());
        self.global_index += generator.get_instructions_size();
    }
    pub fn get_object_instructions(&self) -> Vec<u32> {
        self.object_instructions.clone()
    }
    pub fn fixup(&mut self) {
        for (k, v) in self.global_label_call.iter() {
            if let Some(callee) = self.global_label_offset.get(v) {
                // Indicates this symbol is local and doesn't require dynamic linking.
                if let Some(inst) = self.object_instructions.get(*k as usize) {
                    // instruction, caller, callee can be used to calculate the proper PC-relative offset.
                    let fixup_instruction = self.fixup_internal(*inst, *k, *callee);
                    if let Some(inst) = self.object_instructions.get_mut(*k as usize) {
                        *inst = fixup_instruction.clone();
                    } else {
                        // Doesn't have the proper index, which should not happen!
                        panic!(
                            "The index does not exist in object_instructions, something is wrong!"
                        );
                    }
                }
            } else {
                // Otherwise, it's UNDEFINED and will require relocation.
                todo!();
            }
        }
    }
    // detect instruction type to identify if imm19 or imm26 is used.
    fn fixup_internal(&self, inst: u32, caller: u32, callee: u32) -> u32 {
        match inst {
            x if x & BL == BL => {
                let (_, offset) = calc_b_imm26(caller as usize * 4, callee as usize * 4);
                BL | ((offset & 0x7ffff) & 0x3FFFFFF)
            }
            x if x & B_COND == B_COND => {
                let (_, offset) = calc_bcond_imm19(caller as usize * 4, callee as usize * 4);
                inst | (offset << 5)
            }
            _ => {
                panic!("Not supported opcode in fixup!");
            }
        }
    }
}

mod tests {
    use crate::ast::*;
    use crate::codegen::*;
    use crate::common::*;
    use crate::dump::write_executable_aarch64;
    use crate::label::*;
    use crate::object::*;
    use crate::parser::*;
    use crate::syscall::*;
    #[test]
    fn test_simple_dump() {
        let mut p = Parser::new("(+ 1 2)");
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
        // before has finished executing in JIT-like mode.
        // Will dump binary afterwards.
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
    }
}
