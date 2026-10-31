/**
 Tests for the role the lane contest names when it asks a bench (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. The contest is asked of the late judges.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Lane contest role tests

await describe({
  name: 'the role the lane contest names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE LATE-JUDGE ROLE where the lanes of a slice are contested',
      fn: async () => {
        const source = await readFile(new URL('./lane-contest-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runWindowedRounds', },),).toEqual(['late-judge',],);
      },
    },),
  ],
},);

//endregion Lane contest role tests
