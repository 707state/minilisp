#include "asm_codegen.hpp"

#include <sys/types.h>

#include <cstdint>

#include "ast.hpp"
#include "common.hpp"
#include "object.hpp"

void ASMGenerator::new_instruction() noexcept
{
  cur_instruction_ = 0;
  cur_instr_pos_   = 0;
}
void ASMGenerator::emit_to_memory() noexcept
{
  instructions_.emplace_back(cur_instruction_);
  new_instruction();  // 重置为新的指令
}
void ASMGenerator::gen_movz_instruction(RegisterX rd, uint16_t imm, uint8_t hw,
                                        bool is_64) noexcept
{
  uint32_t new_instruction = 0x52800000;
  new_instruction |= (is_64 << 31);
  new_instruction |= ((hw & 0x3) << 21);
  new_instruction |= (imm << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}
// mov wide imm instruction
void ASMGenerator::gen_mov_imm_instruction(RegisterX rd, uint16_t imm_value,
                                           uint8_t shift, bool is_64) noexcept
{
  assert(shift == 0 || shift == 16 || shift == 32 || shift == 64);
  gen_movz_instruction(rd, imm_value, shift / 16, is_64);
}

// ret instruction
void ASMGenerator::gen_ret_instruction(RegisterX reg) noexcept
{
  uint32_t new_instruction = 0xd65f0000;
  uint8_t reg_num          = static_cast<uint8_t>(reg) & 0x1F;
  new_instruction |= (reg_num << 5);
  write32(new_instruction);
}

// add imm instruction
void ASMGenerator::gen_add_imm_instruction(RegisterX rn, RegisterX rd,
                                           uint16_t imm12, bool shift,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0x11000000;
  new_instruction |= (is_64 << 31);
  new_instruction |= (shift << 22);
  new_instruction |= ((imm12 & 0xfff) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= ((static_cast<uint8_t>(rd) & 0x1f));
  write32(new_instruction);
}

// sub imm instruction
void ASMGenerator::gen_sub_imm_instruction(RegisterX rn, RegisterX rd,
                                           uint16_t imm12, bool shift,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0x51000000;
  new_instruction |= (is_64 << 31);
  new_instruction |= (shift << 22);
  new_instruction |= ((imm12 & 0xfff) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= ((static_cast<uint8_t>(rd) & 0x1f));
  write32(new_instruction);
}
void ASMGenerator::gen_ubfm_instruction(RegisterX rn, RegisterX rd,
                                        uint8_t immr, uint8_t imms, bool N,
                                        bool is_64) noexcept
{
  uint32_t new_instruction = 0x53000000;
  new_instruction |= (is_64 << 31);
  new_instruction |= (N << 22);
  new_instruction |= ((immr & 0x3f) << 16);
  new_instruction |= ((imms & 0x3f) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}
// logical shift left imm instruction. An alias to UBFM instruction.
// TODO: 重构gen_lsl函数
void ASMGenerator::gen_lsl_imm_instruction(RegisterX rn, RegisterX rd,
                                           int shift, bool is_64) noexcept
{
  uint8_t immr = (-shift) & ((is_64 ? 0x3F : 0x1F));
  uint8_t imms = (is_64 ? 63 : 31) - shift;
  gen_ubfm_instruction(rn, rd, immr, imms, is_64, is_64);
}

// logical shift right imm instruction
void ASMGenerator::gen_lsr_imm_instruction(RegisterX rn, RegisterX rd,
                                           int shift, bool is_64) noexcept
{
  gen_ubfm_instruction(rn, rd, shift, is_64 ? 0x3f : 0x1f, is_64, is_64);
}

// bitwise or imm instruction
void ASMGenerator::gen_orr_imm_instruction(RegisterX rn, RegisterX rd,
                                           uint8_t immr, uint8_t imms, bool N,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0x32000000;
  new_instruction |= (is_64 << 31);
  new_instruction |= (N << 22);
  new_instruction |= ((immr & 0x3f) << 16);
  new_instruction |= ((imms & 0x3f) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}

// compare imm instruction
void ASMGenerator::gen_cmp_imm_instruction(RegisterX rn, uint16_t imm12,
                                           bool shift, bool is_64) noexcept
{
  gen_subs_imm_instruction(rn, XZR, imm12, shift, is_64);
}

// substract imm value & set flags instruction
void ASMGenerator::gen_subs_imm_instruction(RegisterX rn, RegisterX rd,
                                            uint16_t imm12, bool sh,
                                            bool is_64) noexcept
{
  uint32_t new_instruction = 0x71000000;
  new_instruction |= (is_64 << 31);
  new_instruction |= (sh << 22);
  new_instruction |= ((imm12 & 0xfff) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}

// conditional select instruction
void ASMGenerator::gen_csel_instruction(RegisterX rm, RegisterX rn,
                                        RegisterX rd, Cond cond,
                                        bool is_64) noexcept
{
  uint32_t new_instruction = 0x1a800000;
  new_instruction |= (is_64 << 31);
  new_instruction |= ((static_cast<uint8_t>(rm) & 0x1f) << 16);
  new_instruction |= ((static_cast<uint8_t>(cond) & 0xf) << 12);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}

// conditional set instruction
void ASMGenerator::gen_cset_instruction(RegisterX rd, IVCond cond,
                                        bool is_64) noexcept
{
  uint32_t new_instruction = 0x1a9f07e0;
  new_instruction |= (is_64 << 31);
  new_instruction |= ((static_cast<uint8_t>(cond) & 0xf) << 12);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}

// bitwise and
void ASMGenerator::gen_and_imm_instruction(RegisterX rn, RegisterX rd,
                                           uint8_t immr, uint8_t imms, bool N,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0x12000000;
  new_instruction |= (is_64 << 31);
  new_instruction |= (N << 22);
  new_instruction |= ((static_cast<uint8_t>(immr) & 0x3f) << 16);
  new_instruction |= ((static_cast<uint8_t>(imms) & 0x3f) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}

// store pair of registers
void ASMGenerator::gen_stp_instruction(RegisterX rn, RegisterX rt,
                                       RegisterX rt2, uint8_t imm,
                                       bool is_pre_index, bool is_signed_offset,
                                       bool is_64) noexcept
{
  uint32_t new_instruction = 0x28000000;
  if (is_signed_offset) {
    new_instruction |= (0x2 << 23);
  }
  else {
    new_instruction |= ((is_pre_index ? 0x3 : 0x1) << 23);
  }
  new_instruction |= (is_64 << 31);
  new_instruction |= ((imm & 0x7f) << 15);
  new_instruction |= ((static_cast<uint8_t>(rt2) & 0x1f) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rt) & 0x1f);
  write32(new_instruction);
}
// load pair of registers
void ASMGenerator::gen_ldp_instruction(RegisterX rn, RegisterX rt,
                                       RegisterX rt2, uint8_t imm,
                                       bool is_pre_index, bool is_signed_offset,
                                       bool is_64) noexcept
{
  // only difference with stp is the 22th bit
  uint32_t new_instruction = 0x28400000;
  if (is_signed_offset) {
    new_instruction |= (0x2 << 23);
  }
  else {
    new_instruction |= ((is_pre_index ? 0x3 : 0x1) << 23);
  }
  new_instruction |= (is_64 << 31);
  new_instruction |= ((imm & 0x7f) << 15);
  new_instruction |= ((static_cast<uint8_t>(rt2) & 0x1f) << 10);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rt) & 0x1f);
  write32(new_instruction);
}

// store register
void ASMGenerator::gen_str_imm_instruction(RegisterX rn, RegisterX rt,
                                           uint16_t imm, bool is_pre_index,
                                           bool is_unsigned_offset,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0xb8000000;
  new_instruction |= (is_64 << 30);
  if (is_unsigned_offset) {
    new_instruction |= (0x1 << 24);
    new_instruction |= ((imm & 0xfff) << 10);
  }
  else {
    new_instruction |= (0x1 << 10);
    new_instruction |= (is_pre_index << 11);
    new_instruction |= ((imm & 0x1ff) << 12);
  }
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rt) & 0x1f);
  write32(new_instruction);
}
// load register
void ASMGenerator::gen_ldr_imm_instruction(RegisterX rn, RegisterX rt,
                                           uint16_t imm, bool is_pre_index,
                                           bool is_unsigned_offset,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0xb8400000;
  new_instruction |= (is_64 << 30);
  if (is_unsigned_offset) {
    new_instruction |= (0x1 << 24);
    new_instruction |= ((imm & 0xfff) << 10);
  }
  else {
    new_instruction |= (0x1 << 10);
    new_instruction |= (is_pre_index << 11);
    new_instruction |= ((imm & 0x1ff) << 12);
  }
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rt) & 0x1f);
  write32(new_instruction);
}

// write to memory

void ASMGenerator::write8(int value) noexcept
{
  assert(cur_instr_pos_ < 4 && "Instruction already complete");

  cur_instruction_ |= (static_cast<uint32_t>(value) << (cur_instr_pos_ * 8));
  cur_instr_pos_++;

  if (cur_instr_pos_ == 4) {
    emit_to_memory();
  }
}

void ASMGenerator::write32(int instruction) noexcept
{
  if (cur_instr_pos_ > 0) {
    emit_to_memory();
  }

  cur_instruction_ = instruction;
  emit_to_memory();
}

// setup memory
void ASMGenerator::init()
{
  memory_ = mmap(nullptr, get_code_size(), PROT_READ | PROT_WRITE,
                 MAP_ANON | MAP_PRIVATE, -1, 0);
  if (memory_ == MAP_FAILED) {
    throw std::runtime_error("Failed to mmap");
  }
  memcpy(memory_, get_code_ptr(), get_code_size());
}
// unmap
int ASMGenerator::reclaim() { return munmap(memory_, get_code_size()); }

// set page protection
int ASMGenerator::make_executable()
{
  return mprotect(memory_, get_code_size(), PROT_READ | PROT_EXEC);
}

// since above operation, this part of memory is executable.
word ASMGenerator::execute()
{
  JITFunction function = (JITFunction)memory_;
  return function();
}
// some frontend stuff
int ASMGenerator::compile_expr(ASTNode *node)
{
  if (AST_is_integer(node)) {
    word value = AST_get_integer(node);
    gen_mov_imm_instruction(X0, Object_encode_integer(value));
    return 0;
  }
  if (AST_is_char(node)) {
    char value = AST_get_char(node);
    gen_mov_imm_instruction(X0, Object_encode_char(value));
    return 0;
  }
  if (AST_is_bool(node)) {
    bool value = AST_get_bool(node);
    gen_mov_imm_instruction(X0, Object_encode_bool(value));
    return 0;
  }
  if (AST_is_nil(node)) {
    gen_mov_imm_instruction(X0, Object_nil());
    return 0;
  }
  if (AST_is_pair(node)) {
    return compile_call(AST_pair_car(node), AST_pair_cdr(node));
  }
  assert(0 && "unexpected node type");
}
// TODO: finish this part
int ASMGenerator::compile_function(ASTNode *node)
{
  int result = compile_expr(node);
  if (result != 0) {
    return result;
  }
  gen_ret_instruction();
  return 0;
}
void ASMGenerator::compile_compare_imm32(int32_t value)
{
  // compare X0 and nil (set flags)
  gen_cmp_imm_instruction(X0, value, false);
  // set X0 based on compare result
  gen_cset_instruction(X0, IVCond::EQ);
  // move 0/1 to payload
  gen_lsl_imm_instruction(X0, X0, kBoolShift);
  BitmaskImmediate imm;
  auto ok = BitmaskImmediate::try_from(kBoolTag, imm);
  if (!ok) {
    assert(0 && "Failed to encode value in compare_imm32");
  }
  // bitwise orr, generate the Lisp object
  gen_orr_imm_instruction(X0, X0, imm.immr, imm.imms, imm.n);
}
int ASMGenerator::compile_call(ASTNode *callable, ASTNode *args)
{
  assert(AST_pair_cdr(args) == AST_nil() &&
         "only unary function calls supported");
  // TODO
  if (AST_is_symbol(callable)) {
    if (AST_symbol_matches(callable, "add1")) {
      compile_expr(operand1(args));
      // add instruction
      gen_add_imm_instruction(X0, X0, Object_encode_integer(1));
      return 0;
    }
    else if (AST_symbol_matches(callable, "sub1")) {
      compile_expr(operand1(args));
      gen_sub_imm_instruction(X0, X0, Object_encode_integer(1));
      return 0;
    }
    else if (AST_symbol_matches(callable, "integer->char")) {
      compile_expr(operand1(args));
      gen_lsl_imm_instruction(X0, X0, kCharShift - kIntegerShift);
      BitmaskImmediate imm;
      bool ok = BitmaskImmediate::try_from(kCharTag, imm);
      if (!ok) {
        assert(0 && "Unable to decode kCharShift-kIntegerShift");
      }
      gen_orr_imm_instruction(X0, X0, imm.immr, imm.imms, imm.n);
      return 0;
    }
    else if (AST_symbol_matches(callable, "char->integer")) {
      compile_expr(operand1(args));
      gen_lsr_imm_instruction(X0, X0, kCharShift);
      gen_lsl_imm_instruction(X0, X0, kIntegerShift);
      return 0;
    }
    else if (AST_symbol_matches(callable, "nil?")) {
      // TODO: 检查nil?的实现是否符合逻辑
      compile_expr(operand1(args));
      compile_compare_imm32(Object_nil());
      return 0;
    }
    else if (AST_symbol_matches(callable, "zero?")) {
      compile_expr(operand1(args));
      compile_compare_imm32(0);
      return 0;
    }
    else if (AST_symbol_matches(callable, "not")) {
      compile_expr(operand1(args));
      compile_compare_imm32(Object_false());
      return 0;
    }
    else if (AST_symbol_matches(callable, "integer?")) {
      // TODO: 还是需要再看看imms, immr的编码规则
      compile_expr(operand1(args));
      BitmaskImmediate imm;
      bool ok = BitmaskImmediate::try_from(kIntegerTagMask, imm);
      if (!ok) {
        assert(0 && "Failed to encode the immediate value");
      }
      gen_and_imm_instruction(X0, X0, imm.immr, imm.imms, imm.n);
      compile_compare_imm32(kIntegerTag);
      return 0;
    }
    else if (AST_symbol_matches(callable, "boolean?")) {
      compile_expr(operand1(args));
      BitmaskImmediate imm;
      bool ok = BitmaskImmediate::try_from(kImmediateTagMask, imm);
      if (!ok) {
        assert(0 && "Failed to encode the immediate value");
      }
      gen_and_imm_instruction(X0, X0, imm.immr, imm.imms, imm.n);
      compile_compare_imm32(kBoolTag);
      return 0;
    }
  }
  assert(0 && "unexpected call type");
}
