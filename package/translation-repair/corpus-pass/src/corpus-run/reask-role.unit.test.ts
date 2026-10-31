/**
 Tests for the role the pass names when it gathers the consistency of a page (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. The consistency pass is asked of the late judges.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Pass role tests

await describe({
  name: 'the role the pass names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE LATE-JUDGE ROLE where the consistency of a page is gathered',
      fn: async () => {
        const source = await readFile(new URL('./pass-consistency-gather.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['late-judge',],);
      },
    },),
  ],
},);

//endregion Pass role tests
