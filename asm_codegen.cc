#include "asm_codegen.hpp"

#include "ast.hpp"

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
void ASMGenerator::gen_mov_instruction(RegisterX reg, uint16_t imm_value,
                                       uint8_t shift, bool is_64) noexcept
{
  uint32_t new_instruction = 0;
  // sf字段
  if (is_64) {
    new_instruction |= (1 << 31);
  }
  // opc字段
  new_instruction |= (2 << 29);
  // 固定6位: 100101
  new_instruction |= (0x25 << 23);
  // hw字段 (bits 22-21): 移位量 (0=0, 1=16, 2=32, 3=48)
  uint8_t hw_value = shift / 16;
  new_instruction |= (hw_value << 21);
  // imm，16位
  new_instruction |= ((imm_value & 0xFFFF) << 5);
  // Rd字段 (bits 4-0): 目标寄存器
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
    gen_mov_instruction(X0, Object_encode_integer(value));
    return 0;
  }
  if (AST_is_char(node)) {
    char value = AST_get_char(node);
    gen_mov_instruction(X0, Object_encode_char(value));
    return 0;
  }
  if (AST_is_bool(node)) {
    bool value = AST_get_bool(node);
    gen_mov_instruction(X0, Object_encode_bool(value));
    return 0;
  }
  if (AST_is_nil(node)) {
    gen_mov_instruction(X0, Object_nil());
    return 0;
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
