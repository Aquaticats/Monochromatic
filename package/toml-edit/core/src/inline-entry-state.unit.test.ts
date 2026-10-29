import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emptyTomlEdit,
  parseTomlEdit,
  tomlSet,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'parsed inline-table state',
  children: [
    it({
      name: 'does not attach phantom comments to inline entries',
      fn: async () => {
        const edit = parseTomlEdit({ source: 't = { x = 1 }\n', },);
        const block = edit.blocks.find(function isKeyValue(entry,) {
          return entry.kind === 'keyvalue';
        },);
        if ((block === undefined) || (block.kind !== 'keyvalue'))
          throw new Error('Expected parsed key-value block',);
        if (block.value.kind !== 'inline-table')
          throw new Error('Expected parsed inline-table value',);
        expect(block.value.entries[0]?.commentsBefore,).toStrictEqual([],);
      },
    },),
    it({
      name: 'does not add comments to synthetic inline-table entries',
      fn: async () => {
        const edit = tomlSet({
          edit: emptyTomlEdit(),
          path: ['t',],
          value: { x: 1, },
        },);
        const block = edit.blocks.find(function isKeyValue(entry,) {
          return entry.kind === 'keyvalue';
        },);
        if ((block === undefined) || (block.kind !== 'keyvalue'))
          throw new Error('Expected synthetic key-value block',);
        if (block.value.kind !== 'inline-table')
          throw new Error('Expected synthetic inline-table value',);
        expect(block.value.entries[0]?.commentsBefore,).toStrictEqual([],);
      },
    },),
    it({
      name: 'does not attach comments to an appended inline-table entry',
      fn: async () => {
        const edit = tomlSet({
          edit: parseTomlEdit({ source: 'foo = { a = 1 }\n', },),
          path: ['foo', 'b',],
          value: 2,
        },);
        const [block,] = edit.blocks;
        if (block?.kind !== 'keyvalue')
          throw new Error('Expected edited key-value block',);
        if (block.value.kind !== 'inline-table')
          throw new Error('Expected edited inline-table value',);
        const entry = block.value.entries.find(function isAdded(candidate,) {
          return candidate.keySegments[0] === 'b';
        },);
        expect(entry?.commentsBefore,).toStrictEqual([],);
      },
    },),
  ],
},);
