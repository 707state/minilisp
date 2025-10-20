project(
  'learn_assembly',
  ['c','cpp'],
  meson_version : '>= 1.3.0',
  version : '0.1',
  default_options : ['warning_level=3','cpp_std=c++23'],
)
asm_codegen_library_src=[
    'asm_codegen.cc'
]

asm_codegen_lib = declare_dependency(
    include_directories: include_directories('.'),
    link_with: library('codegen',asm_codegen_library_src)
)

c_gen_asm=executable(
  'gen_asm',
  'gen_asm.cc',
  dependencies: asm_codegen_lib,
  install: true
)


# minilisp interpreter

executable(
    'minilisp',
    'minilisp.c'
)

# test


doctest_lib=subproject('doctest')
doctest_lib_dep=doctest_lib.get_variable('doctest_dep')


test_src=[
    'test/main.cc',
    'test/test_function.cc',
    'test/test_implicit_conversion.cc'
]

executable('tests',
    test_src,
    include_directories: include_directories('.'),
    dependencies: [doctest_lib_dep, asm_codegen_lib]
)
