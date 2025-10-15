#include <assert.h> /* for assert */
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <stddef.h> /* for NULL */
#include <stdexcept>
#include <string.h>   /* for memcpy */
#include <sys/mman.h> /* for mmap and friends */
#include <vector>
// Objects

typedef int64_t word;
typedef uint64_t uword;

const int kBitsPerByte = 8;                        // bits
const int kWordSize = sizeof(word);                // bytes
const int kBitsPerWord = kWordSize * kBitsPerByte; // bits

const unsigned int kIntegerTag = 0x0;
const unsigned int kIntegerTagMask = 0x3;
const unsigned int kIntegerShift = 2;
const unsigned int kIntegerBits = kBitsPerWord - kIntegerShift;
const word kIntegerMax = (1LL << (kIntegerBits - 1)) - 1;
const word kIntegerMin = -(1LL << (kIntegerBits - 1));

const unsigned int kImmediateTagMask = 0x3f;

const unsigned int kCharTag = 0xf;   // 0b00001111
const unsigned int kCharMask = 0xff; // 0b11111111
const unsigned int kCharShift = 8;

const unsigned int kBoolTag = 0x1f;  // 0b0011111
const unsigned int kBoolMask = 0x80; // 0b10000000
const unsigned int kBoolShift = 7;

word Object_encode_integer(word value) {
  assert(value < kIntegerMax && "too big");
  assert(value > kIntegerMin && "too small");
  return value << kIntegerShift;
}

word Object_encode_char(char value) {
  return ((word)value << kCharShift) | kCharTag;
}

char Object_decode_char(word value) {
  return (value >> kCharShift) & kCharMask;
}

word Object_encode_bool(bool value) {
  return ((word)value << kBoolShift) | kBoolTag;
}

bool Object_decode_bool(word value) { return value & kBoolMask; }

word Object_true() { return Object_encode_bool(true); }

word Object_false() { return Object_encode_bool(false); }

word Object_nil() { return 0x2f; }

// End Objects

// // AST

struct ASTNode;
typedef struct ASTNode ASTNode;

ASTNode *AST_new_integer(word value) {
  return (ASTNode *)Object_encode_integer(value);
}

bool AST_is_integer(ASTNode *node) {
  return ((word)node & kIntegerTagMask) == kIntegerTag;
}

word AST_get_integer(ASTNode *node) { return (word)node >> kIntegerShift; }

bool AST_is_char(ASTNode *node) {
  return ((word)node & kImmediateTagMask) == kCharTag;
}

char AST_get_char(ASTNode *node) { return Object_decode_char((word)node); }

ASTNode *AST_new_char(char value) {
  return (ASTNode *)Object_encode_char(value);
}

bool AST_is_bool(ASTNode *node) {
  return ((word)node & kImmediateTagMask) == kBoolTag;
}

bool AST_get_bool(ASTNode *node) { return Object_decode_bool((word)node); }

ASTNode *AST_new_bool(bool value) {
  return (ASTNode *)Object_encode_bool(value);
}

bool AST_is_nil(ASTNode *node) { return (word)node == Object_nil(); }

ASTNode *AST_nil() { return (ASTNode *)Object_nil(); }

// End AST


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
  void new_instruction() noexcept {
    cur_instruction_ = 0;
    cur_instr_pos_ = 0;
  }
  void emit_to_memory() noexcept {
    instructions_.emplace_back(cur_instruction_);
    new_instruction(); // 重置为新的指令
  }
  void gen_mov_instruction(RegisterX reg, uint16_t imm_value, uint8_t shift = 0,
                           bool is_64 = true) noexcept {
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
  void gen_ret_instruction(RegisterX reg = X30) noexcept {
    uint32_t new_instruction = 0xd65f0000;
    uint8_t reg_num = static_cast<uint8_t>(reg) & 0x1F;
    new_instruction |= (reg_num << 5);
    write32(new_instruction);
  }
  void write8(int value) noexcept {
    assert(cur_instr_pos_ < 4 && "Instruction already complete");

    cur_instruction_ |= (static_cast<uint32_t>(value) << (cur_instr_pos_ * 8));
    cur_instr_pos_++;

    if (cur_instr_pos_ == 4) {
      emit_to_memory();
    }
  }
  void write32(int instruction) noexcept {
    if (cur_instr_pos_ > 0) {
      emit_to_memory();
    }

    cur_instruction_ = instruction;
    emit_to_memory();
  }

  const std::vector<uint32_t> get_instructions() const noexcept {
    return instructions_;
  }

  size_t get_code_size() const noexcept {
    return instructions_.size() * sizeof(uint32_t);
  }
  const uint8_t *get_code_ptr() const noexcept {
    return reinterpret_cast<const uint8_t *>(instructions_.data());
  }

  void print_instructions() const noexcept {
    std::cout << "Generated ARM64 Instructions:" << std::endl;
    std::cout << "=============================" << std::endl;

    for (size_t i = 0; i < instructions_.size(); i++) {
      std::cout << "[" << std::dec << i << "] 0x" << std::hex << std::setw(8)
                << std::setfill('0') << instructions_[i] << std::endl;
    }
    std::cout << std::dec;
  }
  void init() {
    memory_ = mmap(nullptr, get_code_size(), PROT_READ | PROT_WRITE,
                   MAP_ANON | MAP_PRIVATE, -1, 0);
    if (memory_ == MAP_FAILED) {
      throw std::runtime_error("Failed to mmap");
    }
    memcpy(memory_, get_code_ptr(), get_code_size());
  }
  int reclaim() { return munmap(memory_, get_code_size()); }
  int make_executable() {
    return mprotect(memory_, get_code_size(), PROT_READ | PROT_EXEC);
  }
  typedef int (*JITFunction)();
  int execute() {
    JITFunction function = (JITFunction)memory_;
    return function();
  }

private:
  std::vector<uint32_t> instructions_;
  uint32_t cur_instruction_;
  size_t cur_instr_pos_;
  void *memory_ = nullptr;
};

constexpr static int TARGET = 20;
int main() {
  ASMGenerator generator;
  generator.gen_mov_instruction(X0, TARGET);
  generator.gen_ret_instruction();
  generator.init();
  int result = generator.make_executable();
  int return_code = generator.execute();
  assert(result == 0 && "mprotect failed");
  assert(return_code == TARGET && "the assembly was wrong");

  result = generator.reclaim();
  assert(result == 0 && "munmap failed");

  return return_code;
}
