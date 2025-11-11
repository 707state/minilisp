#pragma once
#include <sys/mman.h>

#include <cassert>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <optional>
#include <vector>

#include "ast.hpp"
#include "common.hpp"
#include "object.hpp"

enum RegisterX : uint8_t {
  X0 = 0,
  X1,
  X2,
  X3,
  X4,
  X5,
  X6,
  X7,
  X8,
  X9,
  X10,
  X11,
  X12,
  X13,
  X14,
  X15,
  X16,
  X17,
  X18,
  X19,
  X20,
  X21,
  X22,
  X23,
  X24,
  X25,
  X26,
  X27,
  X28,
  X29,
  X30,
  XZR
};

enum class IVCond : uint8_t {
  NE = 0,
  EQ,
  CC,
  CS,
  PL,
  MI,
  VC,
  VS,
  LS,
  HI,
  LT,
  GE,
  LE,
  GT,
};
enum class Cond : uint8_t {
  EQ = 0,
  NE,
  CS,
  CC,
  MI,
  PL,
  VS,
  VC,
  HI,
  LS,
  GE,
  LT,
  GT,
  LE,
  AL,
  NV
};
class ASMGenerator {
 public:
  ASMGenerator() noexcept { new_instruction(); }
  // instruction related
  void new_instruction() noexcept;
  void emit_to_memory() noexcept;
  // compile-related function
  int compile_expr(ASTNode *node);
  int compile_function(ASTNode *node);
  int compile_call(ASTNode *callable, ASTNode *args);

 private:
  void compile_compare_imm32(int32_t value);
  // codegen
  void gen_mov_imm_instruction(RegisterX reg, uint16_t imm_value,
                               uint8_t shift = 0, bool is_64 = true) noexcept;
  void gen_ret_instruction(RegisterX reg = X30) noexcept;
  void gen_add_imm_instruction(RegisterX rn, RegisterX rd, uint16_t imm12,
                               bool shift = 0, bool is_64 = true) noexcept;
  void gen_sub_imm_instruction(RegisterX rn, RegisterX rd, uint16_t imm12,
                               bool shift = 0, bool is_64 = true) noexcept;
  void gen_ubfm_instruction(RegisterX rn, RegisterX rd, uint8_t immr,
                            uint8_t imms, bool N, bool is_64 = true) noexcept;
  void gen_lsl_imm_instruction(RegisterX rn, RegisterX rd, int shift,
                               bool is_64 = true) noexcept;
  void gen_lsr_imm_instruction(RegisterX rn, RegisterX rd, int shift,
                               bool is_64 = true) noexcept;
  void gen_orr_imm_instruction(RegisterX rn, RegisterX rd, uint8_t immr,
                               uint8_t imms, bool N,
                               bool is_64 = true) noexcept;
  // cmp instruction is an alias to subs command
  void gen_cmp_imm_instruction(RegisterX rn, uint16_t imm12, bool shift,
                               bool is_64 = true) noexcept;
  // conditional select instruction
  void gen_csel_instruction(RegisterX rm, RegisterX rn, RegisterX rd, Cond cond,
                            bool is_64 = true) noexcept;
  void gen_cset_instruction(RegisterX rd, IVCond cond,
                            bool is_64 = true) noexcept;
  // substract with setting flags
  void gen_subs_imm_instruction(RegisterX rn, RegisterX rd, uint16_t imm12,
                                bool sh, bool is_64 = true) noexcept;

  void gen_and_imm_instruction(RegisterX rn, RegisterX rd, uint8_t immr,
                               uint8_t imms, bool N,
                               bool is_64 = true) noexcept;

  // low level memory write
  void write8(int value) noexcept;
  void write32(int instruction) noexcept;

  const std::vector<uint32_t> get_instructions() const noexcept
  {
    return instructions_;
  }

  size_t get_code_size() const noexcept
  {
    return instructions_.size() * sizeof(uint32_t);
  }
  const uint8_t *get_code_ptr() const noexcept
  {
    return reinterpret_cast<const uint8_t *>(instructions_.data());
  }

  /*
   * @brief: helper function!
   */
  void print_instructions() const noexcept
  {
    std::cout << "Generated ARM64 Instructions:" << std::endl;
    std::cout << "=============================" << std::endl;
    const uint8_t *byte_ptr =
        reinterpret_cast<const uint8_t *>(instructions_.data());

    for (size_t i = 0; i < instructions_.size(); i++) {
      const uint8_t *instr_bytes = byte_ptr + i * 4;

      // 从高字节到低字节打印（符合阅读习惯）
      for (int j = 3; j >= 0; j--) {
        std::cout << std::hex << std::setw(2) << std::setfill('0')
                  << static_cast<int>(instr_bytes[j]);
      }
      std::cout << std::endl;
    }
    std::cout << std::dec;
  }

 public:
  // mmap related!
  void init();
  int reclaim();
  int make_executable();
  typedef word (*JITFunction)();
  word execute();

 private:
  std::vector<uint32_t> instructions_;
  uint32_t cur_instruction_;
  size_t cur_instr_pos_;
  void *memory_ = nullptr;
};
