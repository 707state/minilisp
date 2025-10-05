project(
  'minilisp',
  'c',
  meson_version : '>= 1.3.0',
  version : '0.1',
  default_options : ['warning_level=3'],
)

dependencies = [
  dependency('libffi')
]

exe = executable(
  'minilisp',
  'minilisp.c',
  dependencies : dependencies,
  c_args: ['-pedantic-errors','-Werror','-Wall','-std=c11','-Wno-error=unused-parameter'],
  install : true,
)

test('basic', exe)
