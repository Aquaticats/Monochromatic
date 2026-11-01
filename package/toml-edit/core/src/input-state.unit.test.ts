import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emptyTomlEdit,
  tomlFloat,
  tomlInteger,
  tomlLocalDate,
  tomlSet,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'synthetic scalar state',
  children: [
    it({
      name: 'records the TOML kind of each accepted JS scalar',
      fn: async () => {
        const cases = [
          { value: 'text', kind: 'string', },
          { value: true, kind: 'boolean', },
          { value: 42n, kind: 'integer', },
          { value: new Date('2026-05-14T10:00:00.000Z',), kind: 'offset-date-time', },
          { value: 42, kind: 'integer', },
          { value: 1.5, kind: 'float', },
          { value: tomlFloat(1,), kind: 'float', },
          { value: tomlInteger(42n,), kind: 'integer', },
          { value: tomlLocalDate('2026-05-14',), kind: 'local-date', },
        ] as const;
        for (const { value, kind, } of cases) {
          const edit = tomlSet({ edit: emptyTomlEdit(), path: ['value',], value, },);
          const [block,] = edit.blocks;
          if (block?.kind !== 'keyvalue')
            throw new Error('Expected synthetic key-value block',);
          if (block.value.kind !== 'scalar')
            throw new Error('Expected synthetic scalar value',);
          expect(block.value.tomlKind,).toBe(kind,);
        }
      },
    },),
  ],
},);
