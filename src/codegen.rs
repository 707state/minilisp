use crate::common::*;
use crate::env::Environment;
use crate::object::*;
use crate::parser::*;
use capstone::{Capstone, arch, arch::BuildsCapstone};
use libc::{
    MAP_ANON, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE, mmap, mprotect, munmap,
};
use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr;

pub trait CodeSink {
    fn emit(&mut self);
    fn print_asm(&self);
}

pub struct ASMGenerator {
    cur_instruction: u32,
    instructions: Vec<u32>,
    memory: *mut c_void,
    // WARN: 11/22/25, Currently I don't consider these stuff usable except main function.
    // fixup table, this table contains label and it's current offset. When emitting to object file, this hashmap needs to be adjusted to proper index.
    label_offset: HashMap<String, u32>,
    // bl call to label
    label_call: HashMap<u32, String>,
    // environment
    env: Environment,
}
impl CodeSink for ASMGenerator {
    fn emit(&mut self) {
        self.instructions.push(self.cur_instruction);
        self.new_instruction();
    }
    fn print_asm(&self) {
        let mut code: Vec<u8> = Vec::new();
        for word in self.instructions.iter().copied() {
            code.extend(&word.to_le_bytes());
        }

        let cs = Capstone::new()
            .arm64()
            .mode(arch::arm64::ArchMode::Arm)
            .detail(false)
            .build()
            .unwrap();

        let instructions = cs.disasm_all(&code, 0x0).unwrap();

        println!("Disassembly:");
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
impl ASMGenerator {
    pub fn new() -> Self {
        Self {
            cur_instruction: 0,
            instructions: Vec::new(),
            label_offset: HashMap::new(),
            label_call: HashMap::new(),
            memory: ptr::null_mut(),
            env: Environment::new(),
        }
    }

    fn new_instruction(&mut self) {
        self.cur_instruction = 0;
    }
    pub fn adjust_offset(&mut self, prepend_index_size: u32) {
        // adjust each label_offset
        self.label_offset
            .values_mut()
            .for_each(|offset| *offset += prepend_index_size);
        // generate a new label_call to replace the previous one since we need to modify the key.
        self.label_call = self
            .label_call
            .iter()
            .map(|(old_idx, label)| (old_idx + prepend_index_size, label.clone()))
            .collect();
    }

    fn write32(&mut self, instruction: u32) {
        self.cur_instruction = instruction;
        self.emit();
    }

    // movz
    pub fn gen_movz_instruction(&mut self, rd: RegisterX, imm: u16, hw: u8, is_64: bool) {
        let mut inst: u32 = MOVZ;
        inst |= (is_64 as u32) << 31;
        inst |= ((hw & 0x3) as u32) << 21;
        inst |= (imm as u32) << 5;
        inst |= rd as u32;
        self.write32(inst);
    }

    pub fn gen_mov_imm_instruction(&mut self, rd: RegisterX, imm: u16, shift: u8, is_64: bool) {
        assert!(matches!(shift, 0 | 16 | 32 | 64));
        self.gen_movz_instruction(rd, imm, shift / 16, is_64);
    }
    pub fn gen_mov_reg_instruction(&mut self, rd: RegisterX, rm: RegisterX, is_64: bool) {
        self.gen_orr_shifted_reg_instruction(
            rd,
            RegisterX::MovRegister,
            rm,
            0x0,
            Shift::LSL,
            is_64,
        );
    }
    pub fn gen_orr_shifted_reg_instruction(
        &mut self,
        rd: RegisterX,
        rn: RegisterX,
        rm: RegisterX,
        imm6: u8,
        shift: Shift,
        is_64: bool,
    ) {
        let mut inst: u32 = ORR_SHIFTED_REG;
        inst |= (is_64 as u32) << 31;
        inst |= ((shift as u8 & 0x2) as u32) << 22;
        inst |= ((imm6 & 0x3f) as u32) << 10;
        inst |= (((rm as u8) & 0x1f) as u32) << 16;
        inst |= (((rn as u8) & 0x1f) as u32) << 5;
        inst |= ((rd as u8) & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_ret_instruction(&mut self, reg: RegisterX) {
        let mut inst: u32 = RET;
        inst |= ((reg as u8 & 0x1f) as u32) << 5;
        self.write32(inst);
    }
    pub fn gen_mov_to_from_sp_instruction(&mut self, rd: RegisterX, rn: RegisterX, is_64: bool) {
        self.gen_add_imm_instruction(rn, rd, 0, false, is_64);
    }
    pub fn gen_add_shifted_reg_instruction(
        &mut self,
        rd: RegisterX,
        rn: RegisterX,
        rm: RegisterX,
        imm6: u8,
        shift: Shift,
        is_64: bool,
    ) {
        let mut inst: u32 = ADD_SHIFTED_REG;
        inst |= (is_64 as u32) << 31;
        inst |= ((shift as u8 & 0x3) as u32) << 22;
        inst |= ((imm6 & 0x3f) as u32) << 10;
        inst |= ((rm as u32 & 0x1f) as u32) << 16;
        inst |= ((rn as u32 & 0x1f) as u32) << 5;
        inst |= rd as u32 & 0x1f;
        self.write32(inst);
    }
    pub fn gen_add_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        imm12: u16,
        shift: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = ADD_IMM;
        inst |= (is_64 as u32) << 31;
        inst |= (shift as u32) << 22;
        inst |= ((imm12 & 0xfff) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_sub_shifted_reg_instruction(
        &mut self,
        rd: RegisterX,
        rn: RegisterX,
        rm: RegisterX,
        imm6: u8,
        shift: Shift,
        is_64: bool,
    ) {
        let mut inst: u32 = SUB_SHIFTED_REG;
        inst |= (is_64 as u32) << 31;
        inst |= ((shift as u8 & 0x3) as u32) << 22;
        inst |= ((imm6 & 0x3f) as u32) << 10;
        inst |= ((rm as u8 & 0x1f) as u32) << 16;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_sub_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        imm12: u16,
        shift: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = SUB_IMM;
        inst |= (is_64 as u32) << 31;
        inst |= (shift as u32) << 22;
        inst |= ((imm12 & 0xfff) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_ubfm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        immr: u8,
        imms: u8,
        n: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = UBFM;
        inst |= (is_64 as u32) << 31;
        inst |= (n as u32) << 22;
        inst |= ((immr & 0x3f) as u32) << 16;
        inst |= ((imms & 0x3f) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_lsl_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        shift: i32,
        is_64: bool,
    ) {
        let immr = (-shift & if is_64 { 0x3f } else { 0x1f }) as u8;
        let imms = ((if is_64 { 63 } else { 31 }) - shift) as u8;
        self.gen_ubfm_instruction(rn, rd, immr, imms, is_64, is_64);
    }

    pub fn gen_lsr_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        shift: i32,
        is_64: bool,
    ) {
        self.gen_ubfm_instruction(
            rn,
            rd,
            shift as u8,
            if is_64 { 0x3f } else { 0x1f },
            is_64,
            is_64,
        );
    }

    pub fn gen_orr_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        immr: u8,
        imms: u8,
        n: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = ORR_IMM;
        inst |= (is_64 as u32) << 31;
        inst |= (n as u32) << 22;
        inst |= ((immr & 0x3f) as u32) << 16;
        inst |= ((imms & 0x3f) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }
    pub fn gen_and_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        immr: u8,
        imms: u8,
        n: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = AND_IMM;
        inst |= (if is_64 { 1 } else { 0 }) << 31;
        inst |= (if n { 1 } else { 0 }) << 22;
        inst |= ((immr & 0x3f) as u32) << 16;
        inst |= ((imms & 0x3f) as u32) << 10;
        inst |= ((rn as u8 as u32) & 0x1f) << 5;
        inst |= (rd as u8 as u32) & 0x1f;

        self.write32(inst);
    }

    pub fn gen_cmp_imm_instruction(&mut self, rn: RegisterX, imm12: u16, shift: bool, is_64: bool) {
        self.gen_subs_imm_instruction(rn, RegisterX::XZR, imm12, shift, is_64);
    }
    pub fn gen_cmp_shifted_reg_instruction(
        &mut self,
        rn: RegisterX,
        rm: RegisterX,
        imm6: u8,
        shift: Shift,
        is_64: bool,
    ) {
        self.gen_subs_shifted_reg_instruction(RegisterX::MovRegister, rn, rm, imm6, shift, is_64);
    }
    pub fn gen_subs_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        imm12: u16,
        sh: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = SUBS_IMM;
        inst |= (is_64 as u32) << 31;
        inst |= (sh as u32) << 22;
        inst |= ((imm12 & 0xfff) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }
    pub fn gen_subs_shifted_reg_instruction(
        &mut self,
        rd: RegisterX,
        rn: RegisterX,
        rm: RegisterX,
        imm6: u8,
        shift: Shift,
        is_64: bool,
    ) {
        let mut inst: u32 = SUBS_SHIFTED_REG;
        inst |= (is_64 as u32) << 31;
        inst |= ((shift as u8 & 0x3) as u32) << 22;
        inst |= ((imm6 & 0x3f) as u32) << 10;
        inst |= ((rm as u8 & 0x1f) as u32) << 16;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_csel_instruction(
        &mut self,
        rm: RegisterX,
        rn: RegisterX,
        rd: RegisterX,
        cond: Cond,
        is_64: bool,
    ) {
        let mut inst: u32 = CSEL;
        inst |= (is_64 as u32) << 31;
        inst |= ((rm as u8 & 0x1f) as u32) << 16;
        inst |= ((cond as u8 & 0xf) as u32) << 12;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_cset_instruction(&mut self, rd: RegisterX, cond: IVCond, is_64: bool) {
        let mut inst: u32 = CSET;
        inst |= (is_64 as u32) << 31;
        inst |= ((cond as u8 & 0xf) as u32) << 12;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }
    /// store pair (STP)
    pub fn gen_stp_instruction(
        &mut self,
        rn: RegisterX,
        rt: RegisterX,
        rt2: RegisterX,
        imm: i8,
        is_pre_index: bool,
        is_signed_offset: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = STP;
        if is_signed_offset {
            inst |= 0x2 << 23;
        } else {
            inst |= if is_pre_index { 0x3 } else { 0x1 } << 23;
        }
        inst |= (is_64 as u32) << 31;
        inst |= ((imm & 0x7f) as u32) << 15;
        inst |= ((rt2 as u8 & 0x1f) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rt as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    /// load pair (LDP)
    pub fn gen_ldp_instruction(
        &mut self,
        rn: RegisterX,
        rt: RegisterX,
        rt2: RegisterX,
        imm: i8,
        is_pre_index: bool,
        is_signed_offset: bool,
        is_64: bool,
    ) {
        // differs only in opcode constant
        let mut inst: u32 = LDP;
        if is_signed_offset {
            inst |= 0x2 << 23;
        } else {
            inst |= if is_pre_index { 0x3 } else { 0x1 } << 23;
        }
        inst |= (is_64 as u32) << 31;
        inst |= ((imm & 0x7f) as u32) << 15;
        inst |= ((rt2 as u8 & 0x1f) as u32) << 10;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rt as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    /// STR (imm)
    pub fn gen_str_imm_instruction(
        &mut self,
        rn: RegisterX,
        rt: RegisterX,
        imm: i16,
        is_pre_index: bool,
        is_post_index: bool,
        is_unsigned_offset: bool,
        is_64: bool,
    ) {
        if is_pre_index {
            let mut inst = STR_IMM_PRE_IDX;
            inst |= (is_64 as u32) << 30;
            inst |= ((imm & 0x01ff) as u32) << 12;
            inst |= ((rn as u8 & 0x1f) as u32) << 5;
            inst |= (rt as u8 & 0x1f) as u32;
            self.write32(inst);
            return;
        }
        if is_post_index {
            let mut inst = STR_IMM_POST_IDX;
            inst |= (is_64 as u32) << 30;
            inst |= ((imm & 0x01ff) as u32) << 12;
            inst |= ((rn as u8 & 0x1f) as u32) << 5;
            inst |= (rt as u8 & 0x1f) as u32;
            self.write32(inst);
            return;
        }
        if is_unsigned_offset {
            let mut inst = STR_IMM_UNSIGNED_OFFSET;
            inst |= (is_64 as u32) << 30;
            inst |= ((imm & 0x0fff) as u32) << 10;
            inst |= ((rn as u8 & 0x1f) as u32) << 5;
            inst |= (rt as u8 & 0x1f) as u32;
            self.write32(inst);
            return;
        }
    }

    /// LDR (imm)
    pub fn gen_ldr_imm_instruction(
        &mut self,
        rn: RegisterX,
        rt: RegisterX,
        imm: i16,
        is_pre_index: bool,
        is_unsigned_offset: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = LDR_IMM;
        inst |= (is_64 as u32) << 30;
        if is_unsigned_offset {
            inst |= 0x1 << 24;
            inst |= ((imm & 0x0fff) as u32) << 10;
        } else {
            inst |= 0x1 << 10;
            inst |= (is_pre_index as u32) << 11;
            inst |= ((imm & 0x01ff) as u32) << 12;
        }
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rt as u8 & 0x1f) as u32;
        self.write32(inst);
    }
    /// LDR (register)
    pub fn gen_ldr_register_instruction(
        &mut self,
        rn: RegisterX,
        rt: RegisterX,
        rm: RegisterX,
        s: bool,
        option: ExtendedShift,
        is_64: bool,
    ) {
        let mut inst = LDR_REG;
        inst |= (is_64 as u32) << 30;
        inst |= (option as u32) << 13;
        inst |= (rm as u32 & 0x1f) << 16;
        inst |= (s as u32) << 12;
        inst |= (rn as u32 & 0x1f) << 5;
        inst |= rt as u32 & 0x1f;
        self.write32(inst);
    }
    fn gen_stur_instruction(&mut self, rn: RegisterX, rt: RegisterX, imm9: i16, is_64: bool) {
        let mut inst = STUR;
        inst |= (is_64 as u32) << 30;
        inst |= (imm9 as u32 & 0x1ff) << 12;
        inst |= (rn as u32 & 0x1f) << 5;
        inst |= rt as u32 & 0x1f;
        self.write32(inst);
    }
    fn gen_ldur_instruction(&mut self, rn: RegisterX, rt: RegisterX, imm9: i16, is_64: bool) {
        let mut inst = LDUR;
        inst |= (is_64 as u32) << 30;
        inst |= (imm9 as u32 & 0x1ff) << 12;
        inst |= (rn as u32 & 0x1f) << 5;
        inst |= rt as u32 & 0x1f;
        self.write32(inst);
    }
    pub fn gen_bcond_instruction(&mut self, cond: Cond, imm19: u32) {
        let mut inst: u32 = B_COND;
        inst |= (cond as u8 & 0xf) as u32;
        inst |= ((imm19 & 0x7ffff) as u32) << 5;
        self.write32(inst);
    }
    pub fn gen_bl_instruction(&mut self, imm26: u32) {
        let mut inst: u32 = BL;
        inst |= imm26 & 0x3FFFFFF;
        self.write32(inst);
    }
    pub fn gen_svc_instruction(&mut self, imm16: u16) {
        let mut inst = SVC;
        inst |= (imm16 as u32) << 5;
        self.write32(inst);
    }
    /// compile an expression AST node into instructions, return 0 on success
    pub fn compile_expr(&mut self, node: &LispVal, stack_index: Word) -> i32 {
        match node {
            LispVal::Integer(integer) => {
                self.gen_mov_imm_instruction(
                    RegisterX::X0,
                    (object_encode_integer(*integer) & 0xffff) as u16,
                    0,
                    true,
                );
                return 0;
            }
            LispVal::Character(char) => {
                self.gen_mov_imm_instruction(
                    RegisterX::X0,
                    (object_encode_char(*char) & 0xffff) as u16,
                    0,
                    true,
                );
                return 0;
            }
            LispVal::Bool(bool) => {
                self.gen_mov_imm_instruction(
                    RegisterX::X0,
                    object_encode_bool(*bool) as u16,
                    0,
                    true,
                );
                return 0;
            }
            LispVal::Symbol(sym) => {
                // get the address of symbol and load value into X0
                if let Some(pos) = self.env.lookup(sym) {
                    // use ldr X0,[X29,#value]
                    self.gen_ldur_instruction(RegisterX::X29, RegisterX::X0, pos as i16, true);
                    0
                } else {
                    -1
                }
            }
            LispVal::List(val) => {
                if val.is_empty() {
                    self.gen_mov_imm_instruction(RegisterX::X0, object_nil() as u16, 0, true);
                    return 0;
                } else {
                    let car = &val[0];
                    let cdr = &val[1..];
                    return self.compile_call(car, cdr, stack_index);
                }
            }
        }
    }

    /// helper to create boolean/compare results as tagged objects
    fn compile_compare_imm32(&mut self, value: i32) {
        // Compare X0 with immediate (sets flags): implement as SUBS XZR, X0, imm
        // gen_cmp_imm_instruction maps to subs with XZR as destination in the C++ source.
        self.gen_cmp_imm_instruction(RegisterX::X0, value as u16, false, true);

        // Set X0 = 0/1 depending on EQ
        self.gen_cset_instruction(RegisterX::X0, IVCond::EQ, true);

        // Shift payload to boolean payload location
        self.gen_lsl_imm_instruction(RegisterX::X0, RegisterX::X0, K_BOOL_SHIFT as i32, true);

        // OR immediate tag bits to form the boolean immediate object
        if let Some(imm) = BitmaskImmediate::try_from(K_BOOL_TAG as u64) {
            self.gen_orr_imm_instruction(
                RegisterX::X0,
                RegisterX::X0,
                imm.immr,
                imm.imms,
                imm.n != 0,
                true,
            );
        } else {
            panic!("Failed to encode value in compare_imm32");
        }
    }
    fn compile_compare_reg(&mut self, rn: RegisterX, rm: RegisterX) {
        self.gen_cmp_shifted_reg_instruction(rn, rm, 0, Shift::LSL, true);
        self.gen_cset_instruction(RegisterX::X0, IVCond::LS, true);
        self.gen_lsl_imm_instruction(RegisterX::X0, RegisterX::X0, K_BOOL_SHIFT as i32, true);
        if let Some(imm) = BitmaskImmediate::try_from(K_BOOL_TAG as u64) {
            self.gen_orr_imm_instruction(
                RegisterX::X0,
                RegisterX::X0,
                imm.immr,
                imm.imms,
                imm.n != 0,
                true,
            );
        } else {
            panic!("Failed to encode value in compare_imm32");
        }
    }

    /// compile a function call (only unary calls supported, like add1, sub1, ...)
    pub fn compile_call(&mut self, callable: &LispVal, args: &[LispVal], stack_index: Word) -> i32 {
        match callable {
            LispVal::Symbol(sym) => match sym.as_str() {
                "add1" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    let imm = object_encode_integer(1) as u64;
                    self.gen_add_imm_instruction(
                        RegisterX::X0,
                        RegisterX::X0,
                        (imm & 0xfff) as u16,
                        false,
                        true,
                    );
                    0
                }
                "sub1" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    let imm = object_encode_integer(1) as u64;
                    self.gen_sub_imm_instruction(
                        RegisterX::X0,
                        RegisterX::X0,
                        (imm & 0xfff) as u16,
                        false,
                        true,
                    );
                    0
                }
                "integer->char" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    let shift = (K_CHAR_SHIFT - K_INTEGER_SHIFT) as i32;
                    self.gen_lsl_imm_instruction(RegisterX::X0, RegisterX::X0, shift, true);
                    if let Some(imm) = BitmaskImmediate::try_from(K_CHAR_TAG as u64) {
                        self.gen_orr_imm_instruction(
                            RegisterX::X0,
                            RegisterX::X0,
                            imm.immr,
                            imm.imms,
                            imm.n != 0,
                            true,
                        );
                    } else {
                        panic!("Unable to decode kCharShift-kIntegerShift");
                    }
                    0
                }
                "char->integer" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    self.gen_lsr_imm_instruction(
                        RegisterX::X0,
                        RegisterX::X0,
                        K_CHAR_SHIFT as i32,
                        true,
                    );
                    self.gen_lsl_imm_instruction(
                        RegisterX::X0,
                        RegisterX::X0,
                        K_INTEGER_SHIFT as i32,
                        true,
                    );
                    0
                }
                "nil?" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    self.compile_compare_imm32(object_nil() as i32);
                    0
                }
                "zero?" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    self.compile_compare_imm32(0);
                    0
                }
                "not" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    self.compile_compare_imm32(object_false() as i32);
                    0
                }
                "integer?" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    if let Some(imm) = BitmaskImmediate::try_from(K_INTEGER_TAG_MASK as u64) {
                        self.gen_and_imm_instruction(
                            RegisterX::X0,
                            RegisterX::X0,
                            imm.immr,
                            imm.imms,
                            imm.n != 0,
                            true,
                        );
                        self.compile_compare_imm32(K_INTEGER_TAG as i32);
                        return 0;
                    } else {
                        panic!("Failed to encode the immediate value");
                    }
                }
                "boolean?" => {
                    assert_eq!(args.len(), 1);
                    self.compile_expr(&args[0], stack_index);
                    if let Some(imm) = BitmaskImmediate::try_from(K_IMMEDIATE_TAG_MASK as u64) {
                        self.gen_and_imm_instruction(
                            RegisterX::X0,
                            RegisterX::X0,
                            imm.immr,
                            imm.imms,
                            imm.n != 0,
                            true,
                        );
                        self.compile_compare_imm32(K_BOOL_TAG as i32);
                        return 0;
                    } else {
                        panic!("Failed to encode the immediate value");
                    }
                }
                "=" => {
                    assert_eq!(args.len(), 2);
                    self.compile_expr(&args[1], stack_index);
                    self.gen_stur_instruction(
                        RegisterX::X29,
                        RegisterX::X0,
                        stack_index as i16,
                        true,
                    );
                    self.compile_expr(&args[0], stack_index - K_WORD_SIZE);
                    self.gen_ldur_instruction(
                        RegisterX::X29,
                        RegisterX::X1,
                        stack_index as i16,
                        true,
                    );
                    self.gen_cmp_shifted_reg_instruction(
                        RegisterX::X0,
                        RegisterX::X1,
                        0,
                        Shift::LSL,
                        true,
                    );
                    self.gen_cset_instruction(RegisterX::X0, IVCond::EQ, true);
                    self.gen_lsl_imm_instruction(
                        RegisterX::X0,
                        RegisterX::X0,
                        K_BOOL_SHIFT as i32,
                        true,
                    );
                    if let Some(imm) = BitmaskImmediate::try_from(K_BOOL_TAG as u64) {
                        self.gen_orr_imm_instruction(
                            RegisterX::X0,
                            RegisterX::X0,
                            imm.immr,
                            imm.imms,
                            imm.n != 0,
                            true,
                        );
                    } else {
                        panic!("Failed to encode value in compare_imm32");
                    }
                    0
                }
                "<" => {
                    assert_eq!(args.len(), 2);
                    self.compile_expr(&args[1], stack_index);
                    self.gen_stur_instruction(
                        RegisterX::X29,
                        RegisterX::X0,
                        stack_index as i16,
                        true,
                    );
                    self.compile_expr(&args[0], stack_index - K_WORD_SIZE);
                    self.gen_ldur_instruction(
                        RegisterX::X29,
                        RegisterX::X1,
                        stack_index as i16,
                        true,
                    );
                    self.compile_compare_reg(RegisterX::X0, RegisterX::X1);
                    0
                }
                "+" => {
                    assert!(args.len() > 1, "+ expects arguments more than 1");
                    self.compile_expr(&args[0], stack_index);
                    for (i, operand) in args[1..].iter().enumerate() {
                        let tmp_offset = stack_index - (i as i64 + 1) * K_WORD_SIZE;
                        // Save accumulator (X0) to stack
                        self.gen_stur_instruction(
                            RegisterX::X29,
                            RegisterX::X0,
                            tmp_offset as i16,
                            true,
                        );
                        // Compile next operand → X0
                        self.compile_expr(operand, tmp_offset - K_WORD_SIZE);
                        // Load previous accumulator into X1
                        self.gen_ldur_instruction(
                            RegisterX::X29,
                            RegisterX::X1,
                            tmp_offset as i16,
                            true,
                        );
                        // X0 = X1 + X0
                        self.gen_add_shifted_reg_instruction(
                            RegisterX::X0,
                            RegisterX::X1,
                            RegisterX::X0,
                            0,
                            Shift::LSL,
                            true,
                        );
                    }
                    0
                }
                "-" => {
                    assert!(args.len() > 1, "- expects at least 2 arguments");
                    // Compile first operand → X0 (accumulator)
                    self.compile_expr(&args[0], stack_index);
                    // For each remaining operand: acc = acc - operand
                    for (i, operand) in args[1..].iter().enumerate() {
                        let tmp_offset = stack_index - (i as i64 + 1) * K_WORD_SIZE;

                        // Save accumulator (X0) to stack
                        self.gen_stur_instruction(
                            RegisterX::X29,
                            RegisterX::X0,
                            tmp_offset as i16,
                            true,
                        );
                        // Compile next operand → X0
                        self.compile_expr(operand, tmp_offset - K_WORD_SIZE);

                        // Load previous accumulator into X1
                        self.gen_ldur_instruction(
                            RegisterX::X29,
                            RegisterX::X1,
                            tmp_offset as i16,
                            true,
                        );

                        // X0 = X1 - X0
                        self.gen_sub_shifted_reg_instruction(
                            RegisterX::X0, // dst
                            RegisterX::X1, // lhs = previous accumulator
                            RegisterX::X0, // rhs = newly computed operand
                            0,
                            Shift::LSL,
                            true,
                        );
                    }
                    0
                }
                "let" => {
                    // let expression should only have 2 argument, first is binding, second is evaluation.
                    assert_eq!(args.len(), 2);
                    self.compile_let(&args[0], &args[1], stack_index)
                }
                _ => {
                    panic!("{} is not supported!", sym);
                }
            },
            _ => {
                panic!("Unknown function or primitive!");
            }
        }
    }

    /// compile a top-level function node: compile expression and emit ret
    pub fn compile_function(&mut self, node: &LispVal) -> i32 {
        // using entry as pseudo entry point, which is only useful when dumping object file.
        self.label_offset.insert(ENTRY_POINT.to_string(), 0);
        // Function Prologue
        self.gen_stp_instruction(SP, RegisterX::X29, RegisterX::X30, -2, true, false, true);
        self.gen_mov_to_from_sp_instruction(RegisterX::X29, SP, true);
        let result = self.compile_expr(node, -K_WORD_SIZE);
        if result != 0 {
            return result;
        }
        self.gen_ldp_instruction(SP, RegisterX::X29, RegisterX::X30, 2, false, false, true);
        self.gen_ret_instruction(RegisterX::X30);
        0
    }

    // compile a let expression
    fn compile_let(&mut self, bindings: &LispVal, body: &LispVal, stack_index: Word) -> i32 {
        let binding_env = self.env.clone();
        let mut body_env = self.env.clone();
        let result =
            self.compile_let_internal(bindings, body, stack_index, &binding_env, &mut body_env);
        self.env = binding_env;
        result
    }
    fn compile_let_internal(
        &mut self,
        bindings: &LispVal,
        body: &LispVal,
        stack_index: Word,
        binding_env: &Environment,
        body_env: &mut Environment,
    ) -> i32 {
        if let LispVal::List(bindings_tuple) = bindings {
            if bindings_tuple.len() == 0 {
                // bindings is nil
                let saved_env = self.env.clone();
                self.env = body_env.clone();
                let r = self.compile_expr(body, stack_index);
                self.env = saved_env;
                r
            } else {
                //binding is not nil
                let binding = bindings_tuple.first().unwrap();
                let rest = LispVal::List(bindings_tuple[1..].into());
                if let LispVal::List(binding_list) = binding {
                    assert_eq!(binding_list.len(), 2);
                    let sym = match &binding_list[0] {
                        LispVal::Symbol(sym) => sym.clone(),
                        _ => panic!("Expecting a symbol at the beginning of binding!"),
                    };
                    let binding_expr = &binding_list[1];
                    let saved_env = self.env.clone();
                    self.env = binding_env.clone();
                    self.compile_expr(binding_expr, stack_index);
                    self.env = saved_env;
                    // store result to stack
                    self.gen_stur_instruction(
                        RegisterX::X29,
                        RegisterX::X0,
                        stack_index as i16,
                        true,
                    );
                    body_env.bind(sym, stack_index);
                    self.compile_let_internal(
                        &rest,
                        &body,
                        stack_index - K_WORD_SIZE,
                        binding_env,
                        body_env,
                    )
                } else {
                    panic!("Expecting a list inside body!");
                }
            }
        } else {
            panic!("Bindings should be a list!");
        }
    }

    pub fn put_label_offset(&mut self, label: String, offset: u32) {
        self.label_offset.insert(label, offset);
    }
    pub fn put_label_call(&mut self, offset: u32, label: String) {
        self.label_call.insert(offset, label);
    }
    /// get copy of instructions
    pub fn get_instructions(&self) -> Vec<u32> {
        self.instructions.clone()
    }
    pub fn get_label_offset(&self) -> HashMap<String, u32> {
        self.label_offset.clone()
    }
    pub fn get_label_call(&self) -> HashMap<u32, String> {
        self.label_call.clone()
    }

    pub fn get_code_size(&self) -> usize {
        self.instructions.len() * std::mem::size_of::<u32>()
    }
    pub fn get_instructions_size(&self) -> u32 {
        self.instructions.len() as u32
    }
    /// get pointer to instructions
    pub fn get_code_ptr(&self) -> *const u8 {
        self.instructions.as_ptr() as *const u8
    }

    /// mmap an executable memory region
    pub fn init(&mut self) -> Result<(), &'static str> {
        unsafe {
            let size = self.get_code_size();
            let ptr = mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_ANON | MAP_PRIVATE,
                -1,
                0,
            );
            if ptr == MAP_FAILED {
                return Err("Failed to mmap");
            }
            ptr::copy_nonoverlapping(self.get_code_ptr(), ptr as *mut u8, size);
            self.memory = ptr;
        }
        Ok(())
    }

    /// unmap the memory region
    pub fn reclaim(&mut self) -> i32 {
        unsafe { munmap(self.memory, self.get_code_size()) }
    }

    /// set memory as executable
    pub fn make_executable(&mut self) -> i32 {
        unsafe { mprotect(self.memory, self.get_code_size(), PROT_READ | PROT_EXEC) }
    }

    /// execute the binary
    pub fn execute(&self) -> usize {
        unsafe {
            let func: extern "C" fn() -> usize = std::mem::transmute(self.memory);
            func()
        }
    }
}
impl Drop for ASMGenerator {
    fn drop(&mut self) {
        self.reclaim();
    }
}

#[cfg(test)]
mod tests {
    use crate::{codegen::ASMGenerator, object::object_encode_integer, parser::parse_lisp};

    #[test]
    fn test_plus_function() {
        let src = "(+ 1 2 3)";
        match parse_lisp(src) {
            Ok((rest, val)) => {
                assert_eq!(rest, "");
                let mut generator = ASMGenerator::new();
                generator.compile_function(&val);
                let _ = generator.init();
                generator.make_executable();
                let return_val = generator.execute();
                assert_eq!(return_val, object_encode_integer(6) as usize);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
    #[test]
    fn test_sub_function() {
        let src = "(- 6 2 1)";
        match parse_lisp(src) {
            Ok((rest, val)) => {
                assert_eq!(rest, "");
                let mut generator = ASMGenerator::new();
                generator.compile_function(&val);
                let _ = generator.init();
                generator.make_executable();
                let return_val = generator.execute();
                assert_eq!(return_val, object_encode_integer(3) as usize);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
    macro_rules! let_test {
        ($name:ident, $src:expr,$expected:expr) => {
            #[test]
            fn $name() {
                match parse_lisp($src) {
                    Ok((rest, val)) => {
                        assert_eq!(rest, "");
                        let mut generator = ASMGenerator::new();
                        generator.compile_function(&val);
                        let _ = generator.init();
                        generator.make_executable();
                        assert_eq!(generator.execute(), $expected);
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        assert!(false);
                    }
                }
            }
        };
    }
    let_test!(
        test_let_simple_function,
        "(let ((a 1)) a)",
        object_encode_integer(1) as usize
    );
    let_test!(
        test_let_complex_function,
        "(let ((a 1) (b (+ 1 2 3))) (+ a b))",
        object_encode_integer(7) as usize
    );
    let_test!(
        test_let_recurse_function,
        "(let ((a 1)) (let ((a 2)) a))",
        object_encode_integer(2) as usize
    );
}
