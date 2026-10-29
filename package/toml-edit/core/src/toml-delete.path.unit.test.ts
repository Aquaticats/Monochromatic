import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlDelete,
  tomlGetValue,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'nested deletion paths',
  children: [
    it({
      name: 'treats an empty path as a no-op without changing the original state',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a = 1\n[b]\nx = 2\n', },);
        const deleted = tomlDelete({ edit, path: [], },);
        expect(tomlStringify({ edit: deleted, },),).toBe('a = 1\n[b]\nx = 2\n',);
        expect(tomlStringify({ edit, },),).toBe('a = 1\n[b]\nx = 2\n',);
      },
    },),
    it({
      name: 'removes a field from only the selected array-of-tables instance',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[foo]]\nname="a"\n[[foo]]\nname="b"\n', },);
        const deleted = tomlDelete({ edit, path: ['foo', 1, 'name',], },);
        expect(tomlGetValue({ edit: deleted, path: ['foo',], },),).toEqual([
          { name: 'a', },
          {},
        ],);
        expect(tomlStringify({ edit: deleted, },),).toBe('[[foo]]\nname="a"\n[[foo]]\n',);
      },
    },),
    it({
      name: 'descends through nested standard-table headers',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a.b]\nx=1\ny=2\n[a.c]\nz=3\n', },);
        const deleted = tomlDelete({ edit, path: ['a', 'b', 'x',], },);
        expect(tomlStringify({ edit: deleted, },),).toBe('[a.b]\ny=2\n[a.c]\nz=3\n',);
        expect(tomlGetValue({ edit: deleted, path: ['a', 'c', 'z',], },),).toBe(3,);
      },
    },),
    it({
      name: 'does not delete the same key under a different table header',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a]\nx=1\n[b]\nx=2\n', },);
        const deleted = tomlDelete({ edit, path: ['b', 'x',], },);
        expect(tomlStringify({ edit: deleted, },),).toBe('[a]\nx=1\n[b]\n',);
      },
    },),
    it({
      name: 'descends through an inline-table entry before removing a nested field',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = { a = { b = 1, c = 2 }, d = 3 }\n', },);
        const deleted = tomlDelete({ edit, path: ['foo', 'a', 'b',], },);
        expect(tomlGetValue({ edit: deleted, path: ['foo',], },),).toEqual({
          a: { c: 2, },
          d: 3,
        },);
        expect(tomlStringify({ edit: deleted, },),).toBe('foo = { a = { c = 2, }, d = 3, }\n',);
      },
    },),
    it({
      name: 'leaves a sibling inline-table entry intact during nested deletion',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = { a = { x = 1 }, b = { x = 2 } }\n', },);
        const deleted = tomlDelete({ edit, path: ['foo', 'b', 'x',], },);
        expect(tomlGetValue({ edit: deleted, path: ['foo',], },),).toEqual({
          a: { x: 1, },
          b: {},
        },);
      },
    },),
    it({
      name: 'ignores array indices outside the current bounds',
      fn: async () => {
        const source = 'arr = [10,20]\n';
        const edit = parseTomlEdit({ source, },);
        for (const index of [-1, 2, 3,])
          expect(tomlStringify({ edit: tomlDelete({ edit, path: ['arr', index,], },), },),).toBe(source,);
      },
    },),
  ],
},);
