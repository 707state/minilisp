project(
  'learn_assembly',
  ['c','cpp'],
  meson_version : '>= 1.3.0',
  version : '0.1',
  default_options : ['warning_level=3'],
)

dependencies = [
]

exe = executable(
  'learn_assembly',
  'learn_assembly.c',
  dependencies : dependencies,
  install : true,
)
c_gen_asm=executable(
  'gen_asm',
  'gen_asm.cc',
  dependencies: dependencies,
  install: true
)
