/**
 Tests for `tomlInsertCommentAfter`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlInsertCommentAfter,
  tomlStringify,
  TomlPathNotFoundError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlInsertCommentAfter.name,
  children: [
    it({
      name: 'appends an inline trailing comment on the same line',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        const e1 = tomlInsertCommentAfter({
          edit,
          path: ['foo',],
          comment: 'note',
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('foo = 1  # note\n',);
      },
    },),

    it({
      name: 'appends a trailing comment at end of file without adding a newline',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1', },);
        const updated = tomlInsertCommentAfter({
          edit,
          path: ['foo',],
          comment: 'note',
        },);
        expect(tomlStringify({ edit: updated, },),).toBe('foo = 1  # note',);
      },
    },),

    it({
      name: 'throws TomlPathNotFoundError for missing path',
      fn: async () => {
        expect(function insert() {
          tomlInsertCommentAfter({
            edit: parseTomlEdit({ source: 'foo = 1\n', },),
            path: ['missing',],
            comment: 'x',
          },);
        },)
          .toThrow(TomlPathNotFoundError,);
      },
    },),
  ],
},);
