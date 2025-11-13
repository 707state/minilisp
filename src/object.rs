// Translated from object.hpp
// Author: ChatGPT
// Depends on: crate::common

use crate::common::*;

/// Encode an integer into tagged object representation.
pub fn Object_encode_integer(value: word) -> word {
    assert!(value < K_INTEGER_MAX, "too big");
    assert!(value > K_INTEGER_MIN, "too small");
    (value << K_INTEGER_SHIFT) | (K_INTEGER_TAG as word)
}

/// Decode integer from tagged representation.
pub fn Object_decode_integer(value: word) -> word {
    value >> K_INTEGER_SHIFT
}

/// Encode character as tagged object.
pub fn Object_encode_char(value: i32) -> word {
    ((value as word) << K_CHAR_SHIFT) | (K_CHAR_TAG as word)
}

/// Decode character from tagged representation.
pub fn Object_decode_char(value: word) -> i8 {
    ((value >> K_CHAR_SHIFT) & (K_CHAR_MASK as word)) as i8
}

/// Encode boolean as tagged object.
pub fn Object_encode_bool(value: bool) -> word {
    (((value as word) << K_BOOL_SHIFT) | (K_BOOL_TAG as word)) as word
}

/// Decode boolean from tagged representation.
pub fn Object_decode_bool(value: word) -> bool {
    ((value >> K_BOOL_SHIFT) & 1) != 0
}

/// Return encoded true object.
pub fn Object_true() -> word {
    Object_encode_bool(true)
}

/// Return encoded false object.
pub fn Object_false() -> word {
    Object_encode_bool(false)
}

/// Return encoded nil object.
pub fn Object_nil() -> word {
    0x2f
}

/// Return the heap address part of a pointer (strip tag bits).
pub fn Object_address(obj: usize) -> uword {
    (obj as uword) & (K_HEAP_PTR_MASK as uword)
}
