import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlGetNode,
  tomlGetRaw,
  TomlPathNotFoundError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'clean nested read paths',
  children: [
    it({
      name: 'selects the matching standard table when siblings share a key name',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a]\nx=1\n[b]\nx=2\n', },);
        expect(tomlGetRaw({ edit, path: ['a', 'x',], },),).toBe('1',);
        expect(tomlGetRaw({ edit, path: ['b', 'x',], },),).toBe('2',);
        const node = tomlGetNode({ edit, path: ['b', 'x',], },);
        if ((!('type' in node)) || (node.type !== 'TOMLValue'))
          throw new Error('Expected parsed table value node',);
        expect(node.value,).toBe(2,);
      },
    },),
    it({
      name: 'descends through array elements to clean inline-table values',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'arr = [{a=1},{a=2}]\n', },);
        expect(tomlGetRaw({ edit, path: ['arr', 1, 'a',], },),).toBe('2',);
        expect(tomlGetRaw({ edit, path: ['arr', 0,], },),).toBe('{a=1}',);
        expect(() => tomlGetNode({ edit, path: ['arr', 2,], },),)
          .toThrow(TomlPathNotFoundError,);
      },
    },),
    it({
      name: 'descends through nested inline-table key segments',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = { a = { b = 1 }, c = 2 }\n', },);
        expect(tomlGetRaw({ edit, path: ['foo', 'a', 'b',], },),).toBe('1',);
        const node = tomlGetNode({ edit, path: ['foo', 'a', 'b',], },);
        if ((!('type' in node)) || (node.type !== 'TOMLValue'))
          throw new Error('Expected nested inline-table value node',);
        expect(node.value,).toBe(1,);
      },
    },),
    it({
      name: 'descends through a nested indexed array-of-tables header',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[a.b]]\nx=1\n[[a.b]]\nx=2\n', },);
        expect(tomlGetRaw({ edit, path: ['a', 'b', 1, 'x',], },),).toBe('2',);
      },
    },),
  ],
},);
