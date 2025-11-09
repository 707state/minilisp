#include "asm_codegen.hpp"

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
void ASMGenerator::gen_mov_imm_instruction(RegisterX reg, uint16_t imm_value,
                                           uint8_t shift, bool is_64) noexcept
{
  // MOVZ 基础编码: sf=1, opc=10, 固定位=100101
  uint32_t new_instruction = 0x52800000;
  // sf 字段 (bit 31): 64位模式
  new_instruction |= (is_64 << 31);
  // hw 字段 (bits 22-21): 移位量 (0=0, 1=16, 2=32, 3=48)
  uint8_t hw_value = shift / 16;
  new_instruction |= (hw_value << 21);
  // imm16 字段 (bits 20-5): 16位立即数
  new_instruction |= ((imm_value & 0xFFFF) << 5);
  // Rd 字段 (bits 4-0): 目标寄存器
  new_instruction |= (static_cast<uint8_t>(reg) & 0x1F);

  write32(new_instruction);
}
void ASMGenerator::gen_ret_instruction(RegisterX reg) noexcept
{
  uint32_t new_instruction = 0xd65f0000;
  uint8_t reg_num          = static_cast<uint8_t>(reg) & 0x1F;
  new_instruction |= (reg_num << 5);
  write32(new_instruction);
}
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
void ASMGenerator::gen_lsl_imm_instruction(RegisterX rn, RegisterX rd,
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
void ASMGenerator::gen_lsr_imm_instruction(RegisterX rn, RegisterX rd,
                                           uint8_t immr, bool N,
                                           bool is_64) noexcept
{
  uint32_t new_instruction = 0x53007c00;
  new_instruction |= (is_64 << 31);
  new_instruction |= (N << 22);
  new_instruction |= (N << 15);
  new_instruction |= ((static_cast<uint8_t>(rn) & 0x1f) << 5);
  new_instruction |= (static_cast<uint8_t>(rd) & 0x1f);
  write32(new_instruction);
}

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
void ASMGenerator::gen_cmp_imm_instruction(RegisterX rn, uint16_t imm12,
                                           bool shift, bool is_64) noexcept
{
  gen_subs_imm_instruction(rn, XZR, imm12, shift, is_64);
}
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
void ASMGenerator::init()
{
  memory_ = mmap(nullptr, get_code_size(), PROT_READ | PROT_WRITE,
                 MAP_ANON | MAP_PRIVATE, -1, 0);
  if (memory_ == MAP_FAILED) {
    throw std::runtime_error("Failed to mmap");
  }
  memcpy(memory_, get_code_ptr(), get_code_size());
}
int ASMGenerator::reclaim() { return munmap(memory_, get_code_size()); }
int ASMGenerator::make_executable()
{
  return mprotect(memory_, get_code_size(), PROT_READ | PROT_EXEC);
}

word ASMGenerator::execute()
{
  JITFunction function = (JITFunction)memory_;
  return function();
}

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

int ASMGenerator::compile_function(ASTNode *node)
{
  int result = compile_expr(node);
  if (result != 0) {
    return result;
  }
  gen_ret_instruction();
  return 0;
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
    }
    else if (AST_symbol_matches(callable, "integer->char")) {
      compile_expr(operand1(args));
      gen_lsl_imm_instruction(X0, X0, 64 - (kCharShift - kIntegerShift), 0x3f);
      // TODO: 这里需要再学一下a64指令集里面imms, immr的编码规则
      gen_orr_imm_instruction(X0, X0, 0, 0x3c);
      return 0;
    }
    else if (AST_symbol_matches(callable, "char->integer")) {
      compile_expr(operand1(args));
      gen_lsr_imm_instruction(X0, X0, kCharShift - kIntegerShift, 0x3f);
      return 0;
    }
    else if (AST_symbol_matches(callable, "nil?")) {
      // TODO: 实习nil?这个函数的原型
    }
  }
  assert(0 && "unexpected call type");
}
