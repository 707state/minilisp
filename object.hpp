#pragma once

// Objects

#include <cassert>

#include "common.hpp"
word Object_encode_integer(word value)
{
  assert(value < kIntegerMax && "too big");
  assert(value > kIntegerMin && "too small");
  return (value << kIntegerShift) | kIntegerTag;
}

word Object_decode_integer(word value) { return (value >> kIntegerShift); }

word Object_encode_char(char value)
{
  return ((word)value << kCharShift) | kCharTag;
}

char Object_decode_char(word value)
{
  return (value >> kCharShift) & kCharMask;
}
word Object_encode_bool(bool value)
{
  return ((word)value << kBoolShift) | kBoolTag;
}

bool Object_decode_bool(word value)
{
  return (bool)((value >> kBoolShift) & 1);
}

word Object_true() { return Object_encode_bool(true); }
word Object_false() { return Object_encode_bool(false); }

word Object_nil() { return 0x2f; }

uword Object_address(void *obj) { return (uword)obj & kHeapPtrMask; }
// End Objects
