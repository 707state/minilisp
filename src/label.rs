use std::collections::HashMap;

use capstone::{Capstone, arch, arch::BuildsCapstone};

use crate::{
    codegen::{ASMGenerator, CodeSink},
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

impl CodeSink for Labels {
    fn emit(&mut self) {}
    fn print_asm(&self) {
        let mut code: Vec<u8> = Vec::new();
        for word in self.object_instructions.iter().copied() {
            code.extend(&word.to_le_bytes());
        }

        let cs = Capstone::new()
            .arm64()
            .mode(arch::arm64::ArchMode::Arm)
            .detail(false)
            .build()
            .unwrap();

        let instructions = cs.disasm_all(&code, 0x0).unwrap();
        println!("Disassembly Object:");
        println!("============");

        for i in instructions.iter() {
            println!(
                "0x{:08x}:\t{}\t{}",
                i.address(),
                i.mnemonic().unwrap_or(""),
                i.op_str().unwrap_or("")
            );
        }
    }
}
