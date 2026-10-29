import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { _formatPath, } from '@monochromatic-dev/toml-edit-core';

await describe({
  name: _formatPath.name,
  children: [
    it({
      name: 'formats string and numeric segments without extra separators',
      fn: async () => {
        expect(_formatPath({ path: ['fruits', 0, 'name',], },),).toBe('fruits[0].name',);
        expect(_formatPath({ path: ['a', 'b',], },),).toBe('a.b',);
        expect(_formatPath({ path: ['fruits', 10,], },),).toBe('fruits[10]',);
      },
    },),
    it({
      name: 'formats empty and single-segment paths',
      fn: async () => {
        expect(_formatPath({ path: [], },),).toBe('',);
        expect(_formatPath({ path: ['root',], },),).toBe('root',);
      },
    },),
  ],
},);
