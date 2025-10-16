#pragma once

// AST

#include "common.hpp"
#include "object.hpp"
struct ASTNode;
typedef struct ASTNode ASTNode;

ASTNode *AST_new_integer(word value)
{
  return (ASTNode *)Object_encode_integer(value);
}

bool AST_is_integer(ASTNode *node)
{
  return ((word)node & kIntegerTagMask) == kIntegerTag;
}

word AST_get_integer(ASTNode *node) { return (word)node >> kIntegerShift; }

bool AST_is_char(ASTNode *node)
{
  return ((word)node & kImmediateTagMask) == kCharTag;
}

char AST_get_char(ASTNode *node) { return Object_decode_char((word)node); }

ASTNode *AST_new_char(char value)
{
  return (ASTNode *)Object_encode_char(value);
}

bool AST_is_bool(ASTNode *node)
{
  return ((word)node & kImmediateTagMask) == kBoolTag;
}

bool AST_get_bool(ASTNode *node) { return Object_decode_bool((word)node); }

ASTNode *AST_new_bool(bool value)
{
  return (ASTNode *)Object_encode_bool(value);
}

bool AST_is_nil(ASTNode *node) { return (word)node == Object_nil(); }

ASTNode *AST_nil() { return (ASTNode *)Object_nil(); }

// End AST
