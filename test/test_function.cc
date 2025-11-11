#ifndef private
#define private public
#include "asm_codegen.hpp"
#undef private
#endif
#include "ast.hpp"
#include "doctest.h"
#include "object.hpp"
#define SETUP()     \
  generator.init(); \
  generator.make_executable();

TEST_CASE("Compile char test")
{
  ASMGenerator generator;
  char value      = 'b';
  ASTNode *node   = AST_new_char(value);
  int compile_res = generator.compile_function(node);
  CHECK_EQ(compile_res, 0);
  generator.init();
  int result = generator.make_executable();
  CHECK_EQ(result, 0);
  word return_val = generator.execute();
  CHECK_EQ('b', Object_decode_char(return_val));
}
TEST_CASE("Compile nil test")
{
  ASMGenerator generator;
  word value         = Object_nil();
  ASTNode *node      = AST_new_integer(value);
  int compile_result = generator.compile_function(node);
  CHECK_EQ(compile_result, 0);
  generator.init();
  int result = generator.make_executable();
  CHECK_EQ(result, 0);
  word return_val = generator.execute();
  CHECK_EQ(Object_nil(), Object_decode_integer(return_val));
}
TEST_CASE("Compile integer test")
{
  ASMGenerator generator;
  ASTNode *node      = AST_new_integer(1);
  int compile_result = generator.compile_function(node);
  CHECK_EQ(compile_result, 0);
  generator.init();
  int result = generator.make_executable();
  CHECK_EQ(result, 0);
  word return_val = generator.execute();
  CHECK_EQ(1, Object_decode_integer(return_val));
}
TEST_CASE("Test add1 function")
{
  ASMGenerator generator;
  generator.gen_mov_imm_instruction(X0, 42);
  generator.gen_add_imm_instruction(X0, X0, 1);
  generator.gen_ret_instruction();
  generator.init();
  int result = generator.make_executable();
  CHECK_EQ(result, 0);
  word return_val = generator.execute();
  CHECK_EQ(43, return_val);
}

TEST_CASE("Test sub1 function")
{
  ASMGenerator generator;

  generator.gen_mov_imm_instruction(X0, 42);
  generator.gen_sub_imm_instruction(X0, X0, 1);
  generator.gen_ret_instruction();
  generator.init();
  int result = generator.make_executable();
  CHECK_EQ(result, 0);
  word return_val = generator.execute();
  CHECK_EQ(41, return_val);
}
TEST_CASE("compile add1 function test")
{
  ASTNode *node = new_unary_call("add1", AST_new_integer(123));
  ASMGenerator generator;
  generator.compile_function(node);
  generator.init();
  generator.make_executable();
  word return_val = generator.execute();
  CHECK_EQ(return_val, Object_encode_integer(124));
}
TEST_CASE("compile boolean? function test")
{
  {
    ASTNode *node = new_unary_call("boolean?", AST_new_integer(5));
    ASMGenerator generator;
    generator.compile_function(node);
    generator.init();
    generator.make_executable();
    generator.print_instructions();
    word return_val = generator.execute();
    CHECK_EQ(return_val, Object_false());
  }
  {
    ASTNode *node = new_unary_call("boolean?", AST_new_bool(true));
    ASMGenerator generator;
    generator.compile_function(node);
    generator.init();
    generator.make_executable();
    generator.print_instructions();
    word return_val = generator.execute();
    CHECK_EQ(return_val, Object_true());
  }
}
TEST_CASE("compile integer->char function test")
{
  ASTNode *node = new_unary_call("integer->char", AST_new_integer(97));
  ASMGenerator generator;
  generator.compile_function(node);
  generator.init();
  generator.make_executable();
  generator.print_instructions();
  word return_val = generator.execute();
  CHECK_EQ(return_val, Object_encode_char('a'));
}
TEST_CASE("compile char->integer function test")
{
  ASTNode *node = new_unary_call("char->integer", AST_new_char('a'));
  ASMGenerator generator;
  generator.compile_function(node);
  generator.init();
  generator.make_executable();
  generator.print_instructions();
  word return_val = generator.execute();
  CHECK_EQ(return_val, Object_encode_integer(97));
}

TEST_CASE("compile integer? function test")
{
  {
    ASTNode *node = new_unary_call("integer?", AST_new_integer(9));
    ASMGenerator generator;
    generator.compile_function(node);
    SETUP();
    word return_val = generator.execute();
    CHECK_EQ(return_val, Object_true());
  }
  {
    ASTNode *node = new_unary_call("integer?", AST_new_bool(true));
    ASMGenerator generator;
    generator.compile_function(node);
    SETUP();
    generator.print_instructions();
    word return_val = generator.execute();
    CHECK_EQ(return_val, Object_false());
  }
}

TEST_CASE("compile sub1 function test")
{
  {
    ASTNode *node = new_unary_call("sub1", AST_new_integer(20));
    ASMGenerator generator;
    generator.compile_function(node);
    SETUP();
    word return_val = generator.execute();
    CHECK_EQ(return_val, Object_encode_integer(19));
  }
}
