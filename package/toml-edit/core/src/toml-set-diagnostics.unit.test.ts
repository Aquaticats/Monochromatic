import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlSet,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'set rejection diagnostics',
  children: [
    it({
      name: 'names the failing path and the unsupported edit',
      fn: async () => {
        const cases = [
          {
            source: 'foo = {}\n',
            path: ['foo', 0, 'bar',],
            value: 3,
            message: 'Cannot set numeric segment 0 in foo[0].bar',
          },
          {
            source: '',
            path: ['foo', 0, 'bar',],
            value: 3,
            message: 'Cannot path-create at numeric segment 0 in foo[0].bar',
          },
          {
            source: 'foo = { a.b.c = 1 }\n',
            path: ['foo', 'a', 'b',],
            value: 3,
            message: 'tomlSet at foo.a.b would create invalid TOML: inline-table key a.b.c overlaps a.b',
          },
          {
            source: 'foo = 1\n',
            path: ['foo', 'bar',],
            value: 3,
            message: 'Cannot path-create through scalar at path foo.bar',
          },
          {
            source: '[[foo]]\nx=1\n',
            path: ['foo',],
            value: 42,
            message: 'tomlSet on an array-of-tables at foo requires an array value; pass [] to clear all instances',
          },
          {
            source: '[[foo]]\nx=1\n',
            path: ['foo',],
            value: [42,],
            message: 'tomlSet on an array-of-tables at foo requires every element to be a plain object; index 0 is not',
          },
          {
            source: '[a.b]\nx=1\n[a.c]\ny=2\n',
            path: ['a',],
            value: {},
            message: 'tomlSet on the sibling tables at a is not supported; set per sub-table instead',
          },
          {
            source: '[foo]\nx=1\n',
            path: ['foo',],
            value: 42,
            message: 'tomlSet at foo requires a plain object to replace a table body',
          },
          {
            source: 'a=1\n',
            path: [],
            value: 42,
            message: 'tomlSet at document root requires a plain object to replace document body',
          },
        ] as const;
        for (const { source, path, value, message, } of cases) {
          const edit = parseTomlEdit({ source, },);
          expect(() => tomlSet({ edit, path, value, },),).toThrow(message,);
        }
      },
    },),
  ],
},);
