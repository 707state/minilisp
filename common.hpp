
#pragma once

#include <cstdint>
#include <limits>

typedef int64_t word;
typedef uint64_t uword;
typedef int64_t word;
typedef uint64_t uword;

const int kBitsPerByte = 8;
const int kWordSize    = sizeof(word);
const int kBitsPerWord = kWordSize * kBitsPerByte;

// Integer tagging
const unsigned int kIntegerTag     = 0x0;
const unsigned int kIntegerTagMask = 0x3;  //  0b11, 可编码
const unsigned int kIntegerShift   = 2;
const unsigned int kIntegerBits    = kBitsPerWord - kIntegerShift;
const word kIntegerMax             = (1LL << (kIntegerBits - 1)) - 1;
const word kIntegerMin             = -(1LL << (kIntegerBits - 1));

// Immediate mask — 用于判断是否是 immediate（如bool, char等）
const unsigned int kImmediateTagMask = 0x1F;

// Char tagging
const unsigned int kCharTag   = 0x0F;
const unsigned int kCharMask  = 0xFF;
const unsigned int kCharShift = 8;

// Bool tagging
const unsigned int kBoolTag = 0x1F;
// 对于简单的 tag 判断，这里用 0xFF 替代也行

const unsigned int kBoolMask  = 0x1F;
const unsigned int kBoolShift = 5;

// Heap pointer tagging
const unsigned int kPairTag = 0x1;
const uword kHeapTagMask    = 0xF;
const uword kHeapPtrMask    = ~kHeapTagMask;

// Symbol tagging
const unsigned int kSymbolTag = 0x7;

// BitmaskImmediate 表示 AArch64 logical immediate 的三段编码 (n, imms, immr)
struct BitmaskImmediate {
  uint8_t n;     // 1 bit (stored in u8)
  uint8_t imms;  // 6 bits
  uint8_t immr;  // 6 bits

  // 编码为指令中的 13-bit 字段 (放在最低 13 位： n:1 imms:6 immr:6 -> Rust 用
  // (n<<12)|(immr<<6)|imms)
  uint32_t to_u32() const
  {
    return (static_cast<uint32_t>(n) << 12) |
           (static_cast<uint32_t>(immr) << 6) | static_cast<uint32_t>(imms);
  }

  // Try-from: 尝试把一个 u64 值解析成 BitmaskImmediate。
  // 如果不可编码则返回 false；可编码则填充 out 并返回 true。
  static bool try_from(uint64_t value, BitmaskImmediate &out)
  {
    if (value == 0 || value == std::numeric_limits<uint64_t>::max()) {
      return false;
    }

    auto safe_ctzll = [](uint64_t x) -> unsigned {
      if (x == 0) return 64u;
      return static_cast<unsigned>(__builtin_ctzll(x));
    };
    auto safe_clzll = [](uint64_t x) -> unsigned {
      if (x == 0) return 64u;
      return static_cast<unsigned>(__builtin_clzll(x));
    };

    auto rotate_right = [](uint64_t v, unsigned r) -> uint64_t {
      r &= 0x3Fu;  // modulo 64
      if (r == 0) return v;
      return (v >> r) | (v << ((64 - r) & 0x3F));
    };

    // rotations = trailing_zeros(value & (value + 1))
    uint64_t tmp = value & (value + 1);
    unsigned rotations =
        safe_ctzll(tmp);  // 若 tmp==0 则 rotations==64，后续 &0x3F 会变成
                          // 0，保持与 Rust 行为一致

    uint64_t normalized = rotate_right(value, rotations & 0x3F);

    // zeroes = normalized.leading_zeros()
    // ones = (!normalized).trailing_zeros()
    unsigned zeroes = safe_clzll(normalized);
    unsigned ones   = safe_ctzll(~normalized);

    unsigned size = zeroes + ones;  // size 用于表示 pattern 宽度

    // 检查 size 是否可以作为重复宽度（rotate_right(value, size) == value）
    if (rotate_right(value, size & 0x3F) != value) {
      return false;
    }

    // 计算 fields：
    // n = (size >> 6) & 1
    uint8_t n = static_cast<uint8_t>((size >> 6) & 1u);

    // imms = (((size << 1).wrapping_neg() | (ones - 1)) & 0x3F)
    // 在 C++ 中用无符号计算以模拟 wrapping behavior
    uint64_t size_shifted = static_cast<uint64_t>(size) << 1;
    uint64_t neg_size_shifted =
        static_cast<uint64_t>(0) - size_shifted;  // wrapping neg
    uint64_t imms64 =
        (neg_size_shifted | static_cast<uint64_t>(ones - 1)) & 0x3Fu;
    uint8_t imms = static_cast<uint8_t>(imms64 & 0xFFu);

    // immr = ((rotations.wrapping_neg() & (size - 1)) & 0x3F)
    // 注意 rotations 可能是 64（来自 safe_ctzll），但在 rust 里会 & (size-1)
    // 先，且后面再 &0x3F
    uint64_t neg_rot =
        static_cast<uint64_t>(0) - static_cast<uint64_t>(rotations);
    uint64_t immr64 = (neg_rot & static_cast<uint64_t>(size - 1)) & 0x3Fu;
    uint8_t immr    = static_cast<uint8_t>(immr64 & 0xFFu);

    out.n    = n;
    out.imms = imms;
    out.immr = immr;
    return true;
  }
};

// const int kBitsPerByte = 8;                         // bits
// const int kWordSize    = sizeof(word);              // bytes
// const int kBitsPerWord = kWordSize * kBitsPerByte;  // bits

// const unsigned int kIntegerTag     = 0x0;
// const unsigned int kIntegerTagMask = 0x3;
// const unsigned int kIntegerShift   = 2;
// const unsigned int kIntegerBits    = kBitsPerWord - kIntegerShift;
// const word kIntegerMax             = (1LL << (kIntegerBits - 1)) - 1;
// const word kIntegerMin             = -(1LL << (kIntegerBits - 1));

// const unsigned int kImmediateTagMask = 0x3f;

// const unsigned int kCharTag   = 0xf;   // 0b00001111
// const unsigned int kCharMask  = 0xff;  // 0b11111111
// const unsigned int kCharShift = 8;

// const unsigned int kBoolTag   = 0x1f;  // 0b0011111
// const unsigned int kBoolMask  = 0x80;  // 0b10000000
// const unsigned int kBoolShift = 7;

// // tag
// const unsigned int kPairTag = 0x1;
// const uword kHeapTagMask    = ((uword)0x7);
// const uword kHeapPtrMask    = ~kHeapTagMask;

// // symbol

// const unsigned int kSymbolTag = 0x5;
