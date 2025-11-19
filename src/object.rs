// Translated from object.hpp
// Author: ChatGPT
// Depends on: crate::common

use crate::common::*;

/// Encode an integer into tagged object representation.
pub fn object_encode_integer(value: Word) -> Word {
    assert!(value < K_INTEGER_MAX, "too big");
    assert!(value > K_INTEGER_MIN, "too small");
    (value << K_INTEGER_SHIFT) | (K_INTEGER_TAG as Word)
}

/// Decode integer from tagged representation.
pub fn object_decode_integer(value: Word) -> Word {
    value >> K_INTEGER_SHIFT
}

/// Encode character as tagged object.
pub fn object_encode_char(value: i32) -> Word {
    ((value as Word) << K_CHAR_SHIFT) | (K_CHAR_TAG as Word)
}

/// Decode character from tagged representation.
pub fn object_decode_char(value: Word) -> u8 {
    ((value >> K_CHAR_SHIFT) & (K_CHAR_MASK as Word)) as u8
}

/// Encode boolean as tagged object.
/// true -> 63
/// false -> 31
pub fn object_encode_bool(value: bool) -> Word {
    (((value as Word) << K_BOOL_SHIFT) | (K_BOOL_TAG as Word)) as Word
}

/// Decode boolean from tagged representation.
pub fn object_decode_bool(value: Word) -> bool {
    ((value >> K_BOOL_SHIFT) & 1) != 0
}

/// Return encoded true object.
pub fn object_true() -> Word {
    object_encode_bool(true)
}

/// Return encoded false object.
pub fn object_false() -> Word {
    object_encode_bool(false)
}

/// Return encoded nil object.
pub fn object_nil() -> Word {
    0x2f
}
pub fn object_error() -> Word {
    0x3f
}

/// Return the heap address part of a pointer (strip tag bits).
pub fn object_address(obj: usize) -> Uword {
    (obj as Uword) & (K_HEAP_PTR_MASK as Uword)
}
