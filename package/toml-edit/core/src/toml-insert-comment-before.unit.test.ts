/**
 Tests for `tomlInsertCommentBefore`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlInsertCommentBefore,
  tomlStringify,
  TomlPathNotFoundError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlInsertCommentBefore.name,
  children: [
    it({
      name: 'inserts a single comment line before a top-level key',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        const e1 = tomlInsertCommentBefore({
          edit,
          path: ['foo',],
          comment: 'kept',
        },);
        const out = tomlStringify({ edit: e1, },);
        expect(out,).toContain('# kept\n',);
        expect(out.indexOf('# kept',) < out.indexOf('foo =',),).toBe(true,);
      },
    },),

    it({
      name: 'inserts multiple lines',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        const e1 = tomlInsertCommentBefore({
          edit,
          path: ['foo',],
          comment: ['one', 'two',],
        },);
        const out = tomlStringify({ edit: e1, },);
        expect(out,).toContain('# one\n# two\n',);
      },
    },),

    it({
      name: 'throws TomlPathNotFoundError for missing path',
      fn: async () => {
        expect(function insert() {
          tomlInsertCommentBefore({
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
