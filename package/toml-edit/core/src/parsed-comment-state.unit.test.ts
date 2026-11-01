import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseTomlEdit, } from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'parsed comment state',
  children: [
    it({
      name: 'retains attached lines and a hash adjacent to the value',
      fn: async () => {
        const edit = parseTomlEdit({ source: '# one\n# two\nkey = 1#tail\n', },);
        const block = edit.blocks.find(function isKeyValue(entry,) {
          return entry.kind === 'keyvalue';
        },);
        if ((block === undefined) || (block.kind !== 'keyvalue'))
          throw new Error('Expected parsed key-value block',);
        expect(block.commentsBefore,).toStrictEqual([' one', ' two',],);
        expect(block.commentAfter,).toBe('tail',);
      },
    },),
    it({
      name: 'retains attached lines without requiring a trailing comment',
      fn: async () => {
        const edit = parseTomlEdit({ source: '# one\n# two\nkey = 1\n', },);
        const [block,] = edit.blocks.filter(function isKeyValue(entry,) {
          return entry.kind === 'keyvalue';
        },);
        if (block?.kind !== 'keyvalue')
          throw new Error('Expected parsed key-value block',);
        expect(block.commentsBefore,).toStrictEqual([' one', ' two',],);
        expect(Object.hasOwn(block, 'commentAfter',),).toBe(false,);
      },
    },),
    it({
      name: 'retains a trailing comment at end of file without a newline',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1 #tail', },);
        const [block,] = edit.blocks;
        if (block?.kind !== 'keyvalue')
          throw new Error('Expected parsed key-value block',);
        expect(block.commentAfter,).toBe('tail',);
      },
    },),
    it({
      name: 'does not attach a comment from the following line',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1\n# after\n', },);
        const [block,] = edit.blocks;
        if (block?.kind !== 'keyvalue')
          throw new Error('Expected parsed key-value block',);
        expect(Object.hasOwn(block, 'commentAfter',),).toBe(false,);
      },
    },),
  ],
},);
