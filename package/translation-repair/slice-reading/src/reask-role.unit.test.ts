/**
 Tests for the role the block pairing names when it asks a bench (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. Pairing is asked of the run roster, the preparation
 round's bench.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Slice reading role tests

await describe({
  name: 'the role the block pairing names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE ROSTER ROLE where the blocks of an archive are paired',
      fn: async () => {
        const source = await readFile(new URL('pair-blocks-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runWindowedRounds', },),).toEqual(['roster',],);
      },
    },),
  ],
},);

//endregion Slice reading role tests
