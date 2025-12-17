project('gpio', ['c','cpp'],
  version : '0.1',
  default_options : ['warning_level=3'])
add_project_link_arguments('-lm', language : ['c','cpp'])
c_dependencies=[
  dependency('libgpiod')
]
cxx_dependencies=[
  dependency('libgpiodcxx')
]
exe = executable('gpio', 'gpio.c',
  dependencies: c_dependencies,
  install : true)

executable('spi','spi.c',dependencies: c_dependencies)
