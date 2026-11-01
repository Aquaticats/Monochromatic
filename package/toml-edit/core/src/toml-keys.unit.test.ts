/**
 Tests for `tomlKeys`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlKeys,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlKeys.name,
  children: [
    it({
      name: 'lists top-level keys when no path given',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a = 1\nb = 2\n', },);
        expect(tomlKeys({ edit, },),).toStrictEqual(['a', 'b',],);
      },
    },),

    it({
      name: 'lists keys inside a named table',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[tools]\nbun = "1"\nnode = "22"\n', },);
        expect(tomlKeys({ edit, path: ['tools',], },),).toStrictEqual(['bun', 'node',],);
      },
    },),

    it({
      name: 'lists array indices for an inline array',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'xs = [10, 20, 30]\n', },);
        expect(tomlKeys({ edit, path: ['xs',], },),).toStrictEqual([0, 1, 2,],);
      },
    },),

    it({
      name: 'lists indices for array-of-tables',
      fn: async () => {
        const edit = parseTomlEdit({
          source: '[[fruits]]\nname = "a"\n\n[[fruits]]\nname = "b"\n',
        },);
        expect(tomlKeys({ edit, path: ['fruits',], },),).toStrictEqual([0, 1,],);
      },
    },),

    it({
      name: 'returns empty array for a scalar path',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'title = "ok"\n', },);
        expect(tomlKeys({ edit, path: ['title',], },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'returns empty array for a missing path',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        expect(tomlKeys({ edit, path: ['missing',], },),).toStrictEqual([],);
      },
    },),
  ],
},);
