
#pragma once

#include <cstdint>
typedef int64_t word;
typedef uint64_t uword;

const int kBitsPerByte = 8;                         // bits
const int kWordSize    = sizeof(word);              // bytes
const int kBitsPerWord = kWordSize * kBitsPerByte;  // bits

const unsigned int kIntegerTag     = 0x0;
const unsigned int kIntegerTagMask = 0x3;
const unsigned int kIntegerShift   = 2;
const unsigned int kIntegerBits    = kBitsPerWord - kIntegerShift;
const word kIntegerMax             = (1LL << (kIntegerBits - 1)) - 1;
const word kIntegerMin             = -(1LL << (kIntegerBits - 1));

const unsigned int kImmediateTagMask = 0x3f;

const unsigned int kCharTag   = 0xf;   // 0b00001111
const unsigned int kCharMask  = 0xff;  // 0b11111111
const unsigned int kCharShift = 8;

const unsigned int kBoolTag   = 0x1f;  // 0b0011111
const unsigned int kBoolMask  = 0x80;  // 0b10000000
const unsigned int kBoolShift = 7;
