use crate::ast::*;
use crate::common::*;
use crate::object::*;
use std::ptr;
use std::{mem, os::raw::c_void};

use libc::{
    MAP_ANON, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE, memcpy, mmap, mprotect,
    munmap,
};

pub struct ASMGenerator {
    cur_instruction: u32,
    cur_instr_pos: u8,
    instructions: Vec<u32>,
    memory: *mut c_void,
}

impl ASMGenerator {
    pub fn new() -> Self {
        Self {
            cur_instruction: 0,
            cur_instr_pos: 0,
            instructions: Vec::new(),
            memory: ptr::null_mut(),
        }
    }

    fn new_instruction(&mut self) {
        self.cur_instruction = 0;
        self.cur_instr_pos = 0;
    }

    fn emit_to_memory(&mut self) {
        self.instructions.push(self.cur_instruction);
        self.new_instruction();
    }

    fn write8(&mut self, value: u8) {
        assert!(self.cur_instr_pos < 4);
        self.cur_instruction |= (value as u32) << (self.cur_instr_pos * 8);
        self.cur_instr_pos += 1;
        if self.cur_instr_pos == 4 {
            self.emit_to_memory();
        }
    }

    fn write32(&mut self, instruction: u32) {
        if self.cur_instr_pos > 0 {
            self.emit_to_memory();
        }
        self.cur_instruction = instruction;
        self.emit_to_memory();
    }

    // movz
    pub fn gen_movz_instruction(&mut self, rd: RegisterX, imm: u16, hw: u8, is_64: bool) {
        let mut inst: u32 = 0x52800000;
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

    pub fn gen_ret_instruction(&mut self, reg: RegisterX) {
        let mut inst: u32 = 0xd65f0000;
        inst |= ((reg as u8 & 0x1f) as u32) << 5;
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
        let mut inst: u32 = 0x11000000;
        inst |= (is_64 as u32) << 31;
        inst |= (shift as u32) << 22;
        inst |= ((imm12 & 0xfff) as u32) << 10;
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
        let mut inst: u32 = 0x51000000;
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
        let mut inst: u32 = 0x53000000;
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
        let mut inst: u32 = 0x32000000;
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
        let mut inst: u32 = 0x12000000;
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

    pub fn gen_subs_imm_instruction(
        &mut self,
        rn: RegisterX,
        rd: RegisterX,
        imm12: u16,
        sh: bool,
        is_64: bool,
    ) {
        let mut inst: u32 = 0x71000000;
        inst |= (is_64 as u32) << 31;
        inst |= (sh as u32) << 22;
        inst |= ((imm12 & 0xfff) as u32) << 10;
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
        let mut inst: u32 = 0x1a800000;
        inst |= (is_64 as u32) << 31;
        inst |= ((rm as u8 & 0x1f) as u32) << 16;
        inst |= ((cond as u8 & 0xf) as u32) << 12;
        inst |= ((rn as u8 & 0x1f) as u32) << 5;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }

    pub fn gen_cset_instruction(&mut self, rd: RegisterX, cond: IVCond, is_64: bool) {
        let mut inst: u32 = 0x1a9f07e0;
        inst |= (is_64 as u32) << 31;
        inst |= ((cond as u8 & 0xf) as u32) << 12;
        inst |= (rd as u8 & 0x1f) as u32;
        self.write32(inst);
    }
    /// compile an expression AST node into instructions, return 0 on success
    pub fn compile_expr(&mut self, node: ASTNode) -> i32 {
        // Immediate integer
        if AST_is_integer(node) {
            // node is a tagged immediate already, so move whole tagged value into X0
            // our gen_mov_imm_instruction takes a 16-bit immediate fragment; to be consistent
            // with the original simple translation we pass the low 16 bits here.
            // (Real assembler emission may need multiple movz/orr sequences; keep this behavior
            // consistent with the earlier translation.)
            let tagged = node as u64;
            self.gen_mov_imm_instruction(RegisterX::X0, (tagged & 0xffff) as u16, 0, true);
            return 0;
        }

        // Character immediate
        if AST_is_char(node) {
            let tagged = node as u64;
            self.gen_mov_imm_instruction(RegisterX::X0, (tagged & 0xffff) as u16, 0, true);
            return 0;
        }

        // Boolean immediate
        if AST_is_bool(node) {
            let tagged = node as u64;
            self.gen_mov_imm_instruction(RegisterX::X0, (tagged & 0xffff) as u16, 0, true);
            return 0;
        }

        // Nil
        if AST_is_nil(node) {
            let nil_tagged = Object_nil() as u64;
            self.gen_mov_imm_instruction(RegisterX::X0, (nil_tagged & 0xffff) as u16, 0, true);
            return 0;
        }

        // Pair / function call
        if AST_is_pair(node) {
            let car = unsafe { AST_pair_car(node) };
            let cdr = unsafe { AST_pair_cdr(node) };
            return self.compile_call(car, cdr);
        }

        panic!("unexpected node type in compile_expr");
    }

    /// helper to create boolean/compare results as tagged objects
    pub fn compile_compare_imm32(&mut self, value: i32) {
        // Compare X0 with immediate (sets flags): implement as SUBS XZR, X0, imm
        // gen_cmp_imm_instruction maps to subs with XZR as destination in the C++ source.
        self.gen_cmp_imm_instruction(RegisterX::X0, (value as u16), false, true);

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

    /// compile a function call (only unary calls supported, like add1, sub1, ...)
    pub fn compile_call(&mut self, callable: ASTNode, args: ASTNode) -> i32 {
        // ensure unary call
        assert!(
            AST_pair_cdr(args) == AST_nil(),
            "only unary function calls supported"
        );

        if AST_is_symbol(callable) {
            if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"add1\0").unwrap(),
            ) {
                // evaluate operand, leave result in X0
                self.compile_expr(operand1(args));
                // add 1 (encoded integer immediate)
                let imm = Object_encode_integer(1) as u64;
                self.gen_add_imm_instruction(
                    RegisterX::X0,
                    RegisterX::X0,
                    (imm & 0xfff) as u16,
                    false,
                    true,
                );
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"sub1\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
                let imm = Object_encode_integer(1) as u64;
                self.gen_sub_imm_instruction(
                    RegisterX::X0,
                    RegisterX::X0,
                    (imm & 0xfff) as u16,
                    false,
                    true,
                );
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"integer->char\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
                // shift payload: integer->char uses shift difference
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
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"char->integer\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
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
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"nil?\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
                self.compile_compare_imm32(Object_nil() as i32);
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"zero?\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
                self.compile_compare_imm32(0);
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"not\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
                self.compile_compare_imm32(Object_false() as i32);
                return 0;
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"integer?\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
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
            } else if AST_symbol_matches(
                callable,
                std::ffi::CStr::from_bytes_with_nul(b"boolean?\0").unwrap(),
            ) {
                self.compile_expr(operand1(args));
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
        }

        panic!("unexpected call type");
    }

    /// compile a top-level function node: compile expression and emit ret
    pub fn compile_function(&mut self, node: ASTNode) -> i32 {
        let result = self.compile_expr(node);
        if result != 0 {
            return result;
        }
        // return: RET X30 (use X30 since it is link register)
        self.gen_ret_instruction(RegisterX::X30);
        0
    }
    /// 获取指令的拷贝
    pub fn get_instructions(&self) -> Vec<u32> {
        self.instructions.clone()
    }

    /// 计算指令占用字节大小
    pub fn get_code_size(&self) -> usize {
        self.instructions.len() * std::mem::size_of::<u32>()
    }

    /// 获取指令内存指针
    pub fn get_code_ptr(&self) -> *const u8 {
        self.instructions.as_ptr() as *const u8
    }

    /// 打印指令，按照高字节到低字节
    pub fn print_instructions(&self) {
        println!("Generated ARM64 Instructions:");
        println!("=============================");
        for instr in &self.instructions {
            let bytes = instr.to_be_bytes(); // 高字节在前
            for b in &bytes {
                print!("{:02x}", b);
            }
            println!();
        }
    }

    /// mmap 分配可写内存，并拷贝指令
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

    /// 释放 mmap 内存
    pub fn reclaim(&mut self) -> i32 {
        unsafe { munmap(self.memory, self.get_code_size()) }
    }

    /// 将内存页设置为可执行
    pub fn make_executable(&mut self) -> i32 {
        unsafe { mprotect(self.memory, self.get_code_size(), PROT_READ | PROT_EXEC) }
    }

    /// 执行 JIT 代码
    pub fn execute(&self) -> usize {
        unsafe {
            let func: extern "C" fn() -> usize = std::mem::transmute(self.memory);
            func()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;
    use crate::ast::*;
    use crate::codegen::ASMGenerator;
    use crate::object::*;

    macro_rules! setup {
        ($gen:ident) => {{
            $gen.init();
            $gen.make_executable();
        }};
    }

    #[test]
    fn compile_char_test() {
        let mut generator = ASMGenerator::new();
        let value = 'b' as i8;
        let node = AST_new_char(value as i32);
        let compile_res = generator.compile_function(node);
        assert_eq!(compile_res, 0);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(value, Object_decode_char(return_val as i64));
    }

    #[test]
    fn compile_nil_test() {
        let mut generator = ASMGenerator::new();
        let value = Object_nil();
        let node = AST_new_integer(value);
        let compile_res = generator.compile_function(node);
        assert_eq!(compile_res, 0);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_nil(), Object_decode_integer(return_val as i64));
    }

    #[test]
    fn compile_integer_test() {
        let mut generator = ASMGenerator::new();
        let node = AST_new_integer(1);
        let compile_res = generator.compile_function(node);
        assert_eq!(compile_res, 0);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(1, Object_decode_integer(return_val as i64));
    }

    #[test]
    fn test_add1_function() {
        let mut generator = ASMGenerator::new();
        generator.gen_mov_imm_instruction(RegisterX::X0, 42, 0, true);
        generator.gen_add_imm_instruction(RegisterX::X0, RegisterX::X0, 1, false, true);
        generator.gen_ret_instruction(RegisterX::X30);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(43, return_val);
    }

    #[test]
    fn test_sub1_function() {
        let mut generator = ASMGenerator::new();
        generator.gen_mov_imm_instruction(RegisterX::X0, 42, 0, true);
        generator.gen_sub_imm_instruction(RegisterX::X0, RegisterX::X0, 1, false, true);
        generator.gen_ret_instruction(RegisterX::X30);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(41, return_val);
    }

    #[test]
    fn compile_add1_function_test() {
        let name_cstring = CString::new("add1").unwrap();
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_integer(123));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_encode_integer(124), return_val as i64);
    }

    #[test]
    fn compile_boolean_question_test() {
        // integer case
        let name_cstring = CString::new("boolean?").unwrap();
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_integer(5));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_false(), return_val as i64);

        // bool case
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_bool(true));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_true(), return_val as i64);
    }

    #[test]
    fn compile_integer_to_char_test() {
        let name_cstring = CString::new("integer->char").unwrap();
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_integer(97));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_encode_char('a' as i32), return_val as i64);
    }

    #[test]
    fn compile_char_to_integer_test() {
        let name_cstring = CString::new("char->integer").unwrap();
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_char('a' as i32));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_encode_integer(97), return_val as i64);
    }

    #[test]
    fn compile_integer_question_test() {
        let name_cstring = CString::new("integer?").unwrap();
        // integer case
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_integer(9));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_true(), return_val as i64);

        // bool case
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_bool(true));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_false(), return_val as i64);
    }

    #[test]
    fn compile_sub1_function_test() {
        let name_cstring = CString::new("sub1").unwrap();
        let node = new_unary_call(name_cstring.as_c_str(), AST_new_integer(20));
        let mut generator = ASMGenerator::new();
        generator.compile_function(node);
        setup!(generator);
        let return_val = generator.execute();
        assert_eq!(Object_encode_integer(19), return_val as i64);
    }
}
