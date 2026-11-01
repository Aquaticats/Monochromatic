import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseTomlEdit, } from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'parsed document blocks',
  children: [
    it({
      name: 'tiles adjacent key-values without empty filler blocks',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a = 1\nb = 2\n', },);
        expect(edit.blocks.map(function kindOf(block,) {
          return block.kind;
        },),)
          .toStrictEqual(['keyvalue', 'keyvalue',],);
        const [first, second,] = edit.blocks;
        if ((first?.kind !== 'keyvalue') || (second?.kind !== 'keyvalue'))
          throw new Error('Expected adjacent parsed key-values',);
        expect(Object.hasOwn(first, 'commentAfter',),).toBe(false,);
        expect(Object.hasOwn(second, 'commentAfter',),).toBe(false,);
      },
    },),
    it({
      name: 'retains a trailing comment on a parsed table header',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[tbl] # note\nx = 1\n', },);
        const [table,] = edit.blocks;
        if (table?.kind !== 'table')
          throw new Error('Expected parsed table',);
        expect(table.commentAfter,).toBe(' note',);
        expect(Object.hasOwn(table, 'commentAfter',),).toBe(true,);
        expect(table.body.map(function kindOf(block,) {
          return block.kind;
        },),)
          .toStrictEqual(['keyvalue',],);
      },
    },),
    it({
      name: 'keeps distinct parsed indices for every array-of-tables header',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[foo]]\na = 1\n[[foo]]\na = 2\n', },);
        const [first, second,] = edit.blocks;
        if ((first?.kind !== 'table') || (second?.kind !== 'table'))
          throw new Error('Expected array-of-tables blocks',);
        expect(first.aotIndex,).toBe(0,);
        expect(second.aotIndex,).toBe(1,);
        expect(Object.hasOwn(first, 'commentAfter',),).toBe(false,);
        expect(Object.hasOwn(second, 'commentAfter',),).toBe(false,);
      },
    },),
    it({
      name: 'tracks nested array-of-tables indices after every header segment',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[a.b]]\nx = 1\n[[a.b]]\nx = 2\n', },);
        const [first, second,] = edit.blocks;
        if ((first?.kind !== 'table') || (second?.kind !== 'table'))
          throw new Error('Expected nested array-of-tables blocks',);
        expect(first.headerSegments,).toStrictEqual(['a', 'b',],);
        expect(first.aotIndex,).toBe(0,);
        expect(second.aotIndex,).toBe(1,);
      },
    },),
  ],
},);
