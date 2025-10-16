#pragma once
#include <sys/mman.h>

#include <cassert>
#include <cstdint>
#include <iomanip>
#include <iostream>
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

class ASMGenerator {
 public:
  ASMGenerator() noexcept { new_instruction(); }
  // instruction related
  void new_instruction() noexcept;
  void emit_to_memory() noexcept;
  // compile-related function
  int compile_expr(ASTNode *node);
  int compile_function(ASTNode *node);
  // codegen
  void gen_mov_instruction(RegisterX reg, uint16_t imm_value, uint8_t shift = 0,
                           bool is_64 = true) noexcept;
  void gen_ret_instruction(RegisterX reg = X30) noexcept;
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

    for (size_t i = 0; i < instructions_.size(); i++) {
      std::cout << "[" << std::dec << i << "] 0x" << std::hex << std::setw(8)
                << std::setfill('0') << instructions_[i] << std::endl;
    }
    std::cout << std::dec;
  }
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
