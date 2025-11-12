#pragma once

// AST

#include <cstdlib>
#include <cstring>

#include "common.hpp"
#include "object.hpp"
struct ASTNode;
typedef struct ASTNode ASTNode;

// Utility functions

ASTNode *AST_heap_alloc(unsigned char tag, uword size)
{
  uword address = (uword)calloc(size, 1);
  return (ASTNode *)(address | tag);
}

bool AST_is_heap_object(ASTNode *node)
{
  unsigned char tag = (uword)node & kHeapTagMask;
  return (tag & kIntegerTagMask) > 0 && (tag & kImmediateTagMask) != 0x7;
}

// Integer

ASTNode *AST_new_integer(word value)
{
  return (ASTNode *)Object_encode_integer(value);
}

bool AST_is_integer(ASTNode *node)
{
  return ((word)node & kIntegerTagMask) == kIntegerTag;
}

word AST_get_integer(ASTNode *node) { return (word)node >> kIntegerShift; }

// Character

bool AST_is_char(ASTNode *node)
{
  return ((word)node & kImmediateTagMask) == kCharTag;
}

char AST_get_char(ASTNode *node) { return Object_decode_char((word)node); }

ASTNode *AST_new_char(char value)
{
  return (ASTNode *)Object_encode_char(value);
}

// Bool

bool AST_is_bool(ASTNode *node)
{
  return ((word)node & kImmediateTagMask) == kBoolTag;
}

bool AST_get_bool(ASTNode *node) { return Object_decode_bool((word)node); }

ASTNode *AST_new_bool(bool value)
{
  return (ASTNode *)Object_encode_bool(value);
}

// Nil

bool AST_is_nil(ASTNode *node) { return (word)node == Object_nil(); }

ASTNode *AST_nil() { return (ASTNode *)Object_nil(); }

// Pair

struct Pair {
  ASTNode *car;
  ASTNode *cdr;
};
void AST_pair_set_car(ASTNode *node, ASTNode *car);
void AST_pair_set_cdr(ASTNode *node, ASTNode *cdr);

ASTNode *AST_new_pair(ASTNode *car, ASTNode *cdr)
{
  auto node = AST_heap_alloc(kPairTag, sizeof(Pair));
  AST_pair_set_car(node, car);
  AST_pair_set_cdr(node, cdr);
  return node;
}

bool AST_is_pair(ASTNode *node)
{
  return ((uword)node & kHeapTagMask) == kPairTag;
}

Pair *AST_as_pair(ASTNode *node)
{
  assert(AST_is_pair(node));
  return (Pair *)Object_address(node);
}

ASTNode *AST_pair_car(ASTNode *node) { return AST_as_pair(node)->car; }
void AST_pair_set_car(ASTNode *node, ASTNode *car)
{
  AST_as_pair(node)->car = car;
}
ASTNode *AST_pair_cdr(ASTNode *node) { return AST_as_pair(node)->cdr; }

void AST_pair_set_cdr(ASTNode *node, ASTNode *cdr)
{
  AST_as_pair(node)->cdr = cdr;
}

void AST_heap_free(ASTNode *node)
{
  if (!AST_is_heap_object(node)) {
    return;
  }
  if (AST_is_pair(node)) {
    AST_heap_free(AST_pair_car(node));
    AST_heap_free(AST_pair_cdr(node));
  }
  free((void *)Object_address(node));
}

// Symbol

struct Symbol {
  word length;
  char cstr[];
};

Symbol *AST_as_symbol(ASTNode *node);

ASTNode *AST_new_symbol(const char *r)
{
  word data_len = strlen(r) + 1;
  ASTNode *node = AST_heap_alloc(kSymbolTag, sizeof(Symbol) + data_len);
  Symbol *s     = AST_as_symbol(node);
  s->length     = data_len;
  memcpy(s->cstr, r, data_len);
  return node;
}

bool AST_is_symbol(ASTNode *node)
{
  return ((uword)node & kHeapTagMask) == kSymbolTag;
}

Symbol *AST_as_symbol(ASTNode *node)
{
  assert(AST_is_symbol(node));
  return (Symbol *)Object_address(node);
}

const char *AST_symbol_cstr(ASTNode *node)
{
  return (const char *)AST_as_symbol(node)->cstr;
}

bool AST_symbol_matches(ASTNode *node, const char *cstr)
{
  return strcmp(AST_symbol_cstr(node), cstr) == 0;
}

// List

ASTNode *list1(ASTNode *item0) { return AST_new_pair(item0, AST_nil()); }

ASTNode *list2(ASTNode *item0, ASTNode *item1)
{
  return AST_new_pair(item0, list1(item1));
}
ASTNode *new_unary_call(const char *name, ASTNode *arg)
{
  return list2(AST_new_symbol(name), arg);
}

ASTNode *operand1(ASTNode *args) { return AST_pair_car(args); }
ASTNode *operand2(ASTNode *args) { return AST_pair_car(AST_pair_cdr(args)); }

// End AST
