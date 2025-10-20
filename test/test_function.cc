#include "asm_codegen.hpp"
#include "ast.hpp"
#include "doctest.h"
#include "object.hpp"
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
TEST_CASE("Test sub1 function"){
    ASMGenerator generator;
    generator.gen_mov_imm_instruction(X0, 42);
    generator.gen_sub_imm_instruction(X0, X0,1);
    generator.gen_ret_instruction();
    generator.init();
    int result = generator.make_executable();
    CHECK_EQ(result, 0);
    word return_val = generator.execute();
    CHECK_EQ(41, return_val);
}
