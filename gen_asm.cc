#include <assert.h> /* for assert */
#include <stddef.h>
#include <sys/mman.h> /* for mmap and friends */

#include "asm_codegen.hpp"
#include "ast.hpp"
#include "object.hpp"

int main()
{
  ASMGenerator generator;
  char value    = 'a';
  ASTNode *node = AST_new_char(value);
  generator.compile_function(node);
  generator.init();
  int result       = generator.make_executable();
  word return_code = generator.execute();
  assert(result == 0 && "mprotect failed");
  assert('a' == Object_decode_char(return_code) && "the assembly was wrong");
  result = generator.reclaim();
  assert(result == 0 && "munmap failed");
  return return_code;
}
