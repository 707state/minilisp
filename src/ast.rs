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

pub fn ast_heap_alloc(tag: Uword, size: Uword) -> ASTNode {
    {
        let size = size as usize;
        // calloc(size, 1) returns a pointer. We store its address ORed with tag (like the C code).
        unsafe {
            let ptr = calloc(size, 1) as usize;
            (ptr | (tag as usize)) as ASTNode
        }
    }
}

pub fn ast_is_heap_object(node: ASTNode) -> bool {
    let tag = (node & (K_HEAP_TAG_MASK as usize)) as Uword;
    // (tag & kIntegerTagMask) > 0 && (tag & kImmediateTagMask) != 0x7;
    ((tag as u32) & K_INTEGER_TAG_MASK) > 0 && ((tag as u32) & K_IMMEDIATE_TAG_MASK) != 0x7
}

// Integer

pub fn ast_new_integer(value: Word) -> ASTNode {
    object_encode_integer(value) as ASTNode
}

pub fn ast_is_integer(node: ASTNode) -> bool {
    ((node as Word) & (K_INTEGER_TAG_MASK as Word)) == (K_INTEGER_TAG as Word)
}

pub fn ast_get_integer(node: ASTNode) -> Word {
    (node as Word) >> (K_INTEGER_SHIFT as Word)
}

// Character

pub fn ast_is_char(node: ASTNode) -> bool {
    ((node as Word) & (K_IMMEDIATE_TAG_MASK as Word)) == (K_CHAR_TAG as Word)
}

pub fn ast_get_char(node: ASTNode) -> char {
    object_decode_char(node as Word) as char
}

pub fn ast_new_char(value: char) -> ASTNode {
    object_encode_char(value) as ASTNode
}

// Bool

pub fn ast_is_bool(node: ASTNode) -> bool {
    ((node as Word) & (K_IMMEDIATE_TAG_MASK as Word)) == (K_BOOL_TAG as Word)
}

pub fn ast_get_bool(node: ASTNode) -> bool {
    object_decode_bool(node as Word)
}

pub fn ast_new_bool(value: bool) -> ASTNode {
    object_encode_bool(value) as ASTNode
}

// Nil

pub fn ast_is_nil(node: ASTNode) -> bool {
    (node as Word) == object_nil()
}

pub fn ast_nil() -> ASTNode {
    object_nil() as ASTNode
}

pub fn ast_is_error(node: ASTNode) -> bool {
    (node as Word) == object_error()
}
pub fn ast_error() -> ASTNode {
    object_error() as ASTNode
}
// Pair representation (heap-allocated)
#[repr(C)]
pub struct Pair {
    pub car: ASTNode,
    pub cdr: ASTNode,
}

pub fn ast_pair_set_car(node: ASTNode, car: ASTNode) {
    unsafe {
        let p = ast_as_pair(node);
        (*p).car = car;
    }
}

pub fn ast_pair_set_cdr(node: ASTNode, cdr: ASTNode) {
    unsafe {
        let p = ast_as_pair(node);
        (*p).cdr = cdr;
    }
}

pub fn ast_new_pair(car: ASTNode, cdr: ASTNode) -> ASTNode {
    let node = ast_heap_alloc(K_PAIR_TAG as Uword, core::mem::size_of::<Pair>() as Uword);
    ast_pair_set_car(node, car);
    ast_pair_set_cdr(node, cdr);
    node
}

pub fn ast_is_pair(node: ASTNode) -> bool {
    ((node as Uword) & (K_HEAP_TAG_MASK as Uword)) == (K_PAIR_TAG as Uword)
}

pub unsafe fn ast_as_pair(node: ASTNode) -> *mut Pair {
    assert!(ast_is_pair(node));
    object_address(node) as *mut Pair
}

pub fn ast_pair_car(node: ASTNode) -> ASTNode {
    unsafe { (*ast_as_pair(node)).car }
}

pub fn ast_pair_cdr(node: ASTNode) -> ASTNode {
    unsafe { (*ast_as_pair(node)).cdr }
}

pub fn ast_heap_free(node: ASTNode) {
    if !ast_is_heap_object(node) {
        return;
    }
    if ast_is_pair(node) {
        ast_heap_free(ast_pair_car(node));
        ast_heap_free(ast_pair_cdr(node));
    }
    unsafe {
        free(object_address(node) as *mut c_void);
    }
}

// Symbol

#[repr(C)]
pub struct Symbol {
    pub length: Word,
}

pub fn ast_as_symbol(node: ASTNode) -> *mut Symbol {
    assert!(ast_is_symbol(node));
    object_address(node) as *mut Symbol
}

pub fn ast_new_symbol(r: &CStr) -> ASTNode {
    let data_len = r.to_bytes_with_nul().len() as Uword;
    let node = ast_heap_alloc(
        K_SYMBOL_TAG as Uword,
        (core::mem::size_of::<Symbol>() as Uword) + data_len,
    );
    unsafe {
        let s = ast_as_symbol(node);
        (*s).length = data_len as Word;
        let dest = (object_address(node) as *mut u8).add(core::mem::size_of::<Symbol>());
        ptr::copy_nonoverlapping(r.as_ptr() as *const u8, dest, data_len as usize);
    }
    node
}

pub fn ast_is_symbol(node: ASTNode) -> bool {
    ((node as Uword) & (K_HEAP_TAG_MASK as Uword)) == (K_SYMBOL_TAG as Uword)
}

pub fn ast_symbol_cstr(node: ASTNode) -> *const c_char {
    unsafe {
        (object_address(node) as *const u8).add(core::mem::size_of::<Symbol>()) as *const c_char
    }
}
pub fn ast_symbol_as_cstr(node: ASTNode) -> &'static CStr {
    unsafe {
        let ptr = (object_address(node) as *const u8).add(core::mem::size_of::<Symbol>())
            as *const c_char;
        CStr::from_ptr(ptr)
    }
}
pub fn ast_symbol_matches(node: ASTNode, cstr: &CStr) -> bool {
    unsafe {
        let ptr = ast_symbol_cstr(node);
        CStr::from_ptr(ptr) == cstr
    }
}

// List helpers

pub fn list1(item0: ASTNode) -> ASTNode {
    ast_new_pair(item0, ast_nil())
}

pub fn list2(item0: ASTNode, item1: ASTNode) -> ASTNode {
    ast_new_pair(item0, list1(item1))
}
pub fn list3(item0: ASTNode, item1: ASTNode, item2: ASTNode) -> ASTNode {
    ast_new_pair(item0, list2(item1, item2))
}

pub fn new_unary_call(name: &CStr, arg: ASTNode) -> ASTNode {
    list2(ast_new_symbol(name), arg)
}
pub fn new_binary_call(name: &CStr, arg0: ASTNode, arg1: ASTNode) -> ASTNode {
    list3(ast_new_symbol(name), arg0, arg1)
}

pub fn operand1(args: ASTNode) -> ASTNode {
    ast_pair_car(args)
}

pub fn operand2(args: ASTNode) -> ASTNode {
    ast_pair_car(ast_pair_cdr(args))
}
pub fn operand3(args: ASTNode) -> ASTNode {
    ast_pair_car(ast_pair_car(ast_pair_cdr(args)))
}

pub fn print_ast(node: ASTNode) {
    println!("{}", format_ast(node));
}

/// Convert ASTNode to a Lisp-readable string
pub fn format_ast(node: ASTNode) -> String {
    if ast_is_nil(node) {
        return "()".to_string();
    }

    if ast_is_pair(node) {
        return format!("({})", format_list(node));
    }

    if ast_is_symbol(node) {
        let s = ast_symbol_as_cstr(node).to_str().unwrap();
        return s.to_string();
    }

    if ast_is_integer(node) {
        return ast_get_integer(node).to_string();
    }

    if ast_is_char(node) {
        let c = ast_get_char(node) as u8 as char;
        return format!("'{}'", c);
    }

    if ast_is_bool(node) {
        return if ast_get_bool(node) { "#t" } else { "#f" }.to_string();
    }

    "<unknown>".to_string()
}

fn format_list(node: ASTNode) -> String {
    let mut parts = Vec::new();
    let mut cur = node;

    while ast_is_pair(cur) {
        let car = operand1(cur);
        parts.push(format_ast(car));
        cur = operand2(cur);
    }

    // proper list ends with nil
    if ast_is_nil(cur) {
        parts.join(" ")
    } else {
        // dotted pair: (a b . c)
        format!("{} . {}", parts.join(" "), format_ast(cur))
    }
}
