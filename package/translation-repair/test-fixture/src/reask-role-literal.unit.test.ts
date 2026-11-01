/**
 Tests for reading what an object literal names under a key wherever it is written (ledger B1163).

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  keysNamedInLiteralsOfType,
  keysNamedInLiteralsUnder,
} from './reask-role-literal.test-fixture.ts';

//region Re-ask role literal tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: keysNamedInLiteralsUnder.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS WHAT EVERY LITERAL WRITTEN UNDER A PROPERTY NAME NAMES UNDER A KEY (ledger B1163): the key, none '
            + 'where it is absent, a spread alone, and no entry for a property that holds a variable',
          fn: async () => {
            const source = [
              'const a = { reaskSeat: { role, bench, reask: \'other-model\', }, };',
              'const b = { reaskSeat: { role, bench, }, };',
              'const c = { reaskSeat: { ...reaskSeat, otherModelMessages, }, };',
              'const d = { reaskSeat, };',
              'const e = { reaskSeat: seat, };',
              '// reaskSeat: { reask: \'measured-model\', } in a comment is no literal',
            ].join('\n',);
            expect(keysNamedInLiteralsUnder({ source, holder: 'reaskSeat', key: 'reask', },),).toEqual([
              'other-model',
              'none',
              'only a spread',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: keysNamedInLiteralsOfType.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS WHAT A LITERAL ASSIGNED TO A VARIABLE OF A TYPE NAMES UNDER A KEY (ledger B1163), and no other '
            + 'variable',
          fn: async () => {
            const source = [
              'const a: ReaskSeat = { role: \'reader\', bench, };',
              'const b: ReaskSeat = { role, bench, reask, };',
              'const c: Other = { role, };',
              'const d: ReaskSeat = build();',
            ].join('\n',);
            expect(keysNamedInLiteralsOfType({ source, type: 'ReaskSeat', key: 'reask', },),).toEqual([
              'none',
              'reask',
            ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Re-ask role literal tests
