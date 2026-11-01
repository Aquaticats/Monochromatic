/**
 Tests for `tomlGetComments`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlGetComments,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlGetComments.name,
  children: [
    it({
      name: 'returns every Block comment in source order',
      fn: async () => {
        const edit = parseTomlEdit({
          source: '# one\nfoo = 1\n# two\nbar = 2\n',
        },);
        const comments = tomlGetComments({ edit, },);
        expect(comments.length,).toBe(2,);
        expect(comments.map(function values(c,) {
          return c.value;
        },),)
          .toStrictEqual([' one', ' two',],);
      },
    },),

    it({
      name: 'returns empty array when source has no comments',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        expect(tomlGetComments({ edit, },).length,).toBe(0,);
      },
    },),
  ],
},);
