// Translated from the provided C++ AST implementation to Rust.
// This file expects `common.rs` and `object.rs` (or equivalents) to exist in the crate
// and provide the Object_* helpers used below (Object_address, Object_decode_char, etc.).

use core::ffi::c_void;
use core::ptr;
use std::ffi::CStr;
use std::os::raw::c_char;

use libc::{calloc, free};

use crate::common::*;
use crate::object::*; // expected to provide Object_encode_integer, Object_address, etc.

// Represent an AST node as a tagged usize (same as the C code's pointer-with-tags).
pub type ASTNode = usize;

// Utility functions

pub fn AST_heap_alloc(tag: uword, size: uword) -> ASTNode {
    {
        let size = size as usize;
        // calloc(size, 1) returns a pointer. We store its address ORed with tag (like the C code).
        unsafe {
            let ptr = calloc(size, 1) as usize;
            (ptr | (tag as usize)) as ASTNode
        }
    }
}

pub fn AST_is_heap_object(node: ASTNode) -> bool {
    let tag = (node & (K_HEAP_TAG_MASK as usize)) as uword;
    // (tag & kIntegerTagMask) > 0 && (tag & kImmediateTagMask) != 0x7;
    ((tag as u32) & K_INTEGER_TAG_MASK) > 0 && ((tag as u32) & K_IMMEDIATE_TAG_MASK) != 0x7
}

// Integer

pub fn AST_new_integer(value: word) -> ASTNode {
    Object_encode_integer(value) as ASTNode
}

pub fn AST_is_integer(node: ASTNode) -> bool {
    ((node as word) & (K_INTEGER_TAG_MASK as word)) == (K_INTEGER_TAG as word)
}

pub fn AST_get_integer(node: ASTNode) -> word {
    (node as word) >> (K_INTEGER_SHIFT as word)
}

// Character

pub fn AST_is_char(node: ASTNode) -> bool {
    ((node as word) & (K_IMMEDIATE_TAG_MASK as word)) == (K_CHAR_TAG as word)
}

pub fn AST_get_char(node: ASTNode) -> i8 {
    Object_decode_char(node as word)
}

pub fn AST_new_char(value: i32) -> ASTNode {
    Object_encode_char(value) as ASTNode
}

// Bool

pub fn AST_is_bool(node: ASTNode) -> bool {
    ((node as word) & (K_IMMEDIATE_TAG_MASK as word)) == (K_BOOL_TAG as word)
}

pub fn AST_get_bool(node: ASTNode) -> bool {
    Object_decode_bool(node as word)
}

pub fn AST_new_bool(value: bool) -> ASTNode {
    Object_encode_bool(value) as ASTNode
}

// Nil

pub fn AST_is_nil(node: ASTNode) -> bool {
    (node as word) == Object_nil()
}

pub fn AST_nil() -> ASTNode {
    Object_nil() as ASTNode
}

// Pair representation (heap-allocated)
#[repr(C)]
pub struct Pair {
    pub car: ASTNode,
    pub cdr: ASTNode,
}

pub fn AST_pair_set_car(node: ASTNode, car: ASTNode) {
    unsafe {
        let p = AST_as_pair(node);
        (*p).car = car;
    }
}

pub fn AST_pair_set_cdr(node: ASTNode, cdr: ASTNode) {
    unsafe {
        let p = AST_as_pair(node);
        (*p).cdr = cdr;
    }
}

pub fn AST_new_pair(car: ASTNode, cdr: ASTNode) -> ASTNode {
    let node = AST_heap_alloc(K_PAIR_TAG as uword, core::mem::size_of::<Pair>() as uword);
    AST_pair_set_car(node, car);
    AST_pair_set_cdr(node, cdr);
    node
}

pub fn AST_is_pair(node: ASTNode) -> bool {
    ((node as uword) & (K_HEAP_TAG_MASK as uword)) == (K_PAIR_TAG as uword)
}

pub unsafe fn AST_as_pair(node: ASTNode) -> *mut Pair {
    assert!(AST_is_pair(node));
    Object_address(node) as *mut Pair
}

pub fn AST_pair_car(node: ASTNode) -> ASTNode {
    unsafe { (*AST_as_pair(node)).car }
}

pub fn AST_pair_cdr(node: ASTNode) -> ASTNode {
    unsafe { (*AST_as_pair(node)).cdr }
}

pub fn AST_heap_free(node: ASTNode) {
    if !AST_is_heap_object(node) {
        return;
    }
    if AST_is_pair(node) {
        AST_heap_free(AST_pair_car(node));
        AST_heap_free(AST_pair_cdr(node));
    }
    unsafe {
        free(Object_address(node) as *mut c_void);
    }
}

// Symbol

#[repr(C)]
pub struct Symbol {
    pub length: word,
    // flexible array member - we allocate extra bytes after this struct
}

pub fn AST_as_symbol(node: ASTNode) -> *mut Symbol {
    assert!(AST_is_symbol(node));
    unsafe { Object_address(node) as *mut Symbol }
}

pub fn AST_new_symbol(r: &CStr) -> ASTNode {
    let data_len = unsafe { r.to_bytes_with_nul().len() } as uword;
    let node = AST_heap_alloc(
        K_SYMBOL_TAG as uword,
        (core::mem::size_of::<Symbol>() as uword) + data_len,
    );
    unsafe {
        let s = AST_as_symbol(node);
        (*s).length = data_len as word;
        let dest = (Object_address(node) as *mut u8).add(core::mem::size_of::<Symbol>());
        ptr::copy_nonoverlapping(r.as_ptr() as *const u8, dest, data_len as usize);
    }
    node
}

pub fn AST_is_symbol(node: ASTNode) -> bool {
    ((node as uword) & (K_HEAP_TAG_MASK as uword)) == (K_SYMBOL_TAG as uword)
}

pub fn AST_symbol_cstr(node: ASTNode) -> *const c_char {
    unsafe {
        let s = AST_as_symbol(node);
        (Object_address(node) as *const u8).add(core::mem::size_of::<Symbol>()) as *const c_char
    }
}

pub fn AST_symbol_matches(node: ASTNode, cstr: &CStr) -> bool {
    unsafe {
        let ptr = AST_symbol_cstr(node);
        CStr::from_ptr(ptr) == cstr
    }
}

// List helpers

pub fn list1(item0: ASTNode) -> ASTNode {
    AST_new_pair(item0, AST_nil())
}

pub fn list2(item0: ASTNode, item1: ASTNode) -> ASTNode {
    AST_new_pair(item0, list1(item1))
}
pub fn list3(item0: ASTNode, item1: ASTNode, item2: ASTNode) -> ASTNode {
    AST_new_pair(item0, list2(item1, item2))
}

pub fn new_unary_call(name: &CStr, arg: ASTNode) -> ASTNode {
    list2(AST_new_symbol(name), arg)
}
pub fn new_binary_call(name: &CStr, arg0: ASTNode, arg1: ASTNode) -> ASTNode {
    list3(AST_new_symbol(name), arg0, arg1)
}

pub fn operand1(args: ASTNode) -> ASTNode {
    AST_pair_car(args)
}

pub fn operand2(args: ASTNode) -> ASTNode {
    AST_pair_car(AST_pair_cdr(args))
}
