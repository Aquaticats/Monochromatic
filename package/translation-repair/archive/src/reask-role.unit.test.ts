/**
 Tests for the role each asking of a bench names in this package (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. The archive is reviewed and its coverage and naturalness
 read with the run roster, the preparation round's bench.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Archive role tests

await describe({
  name: 'the role the archive stages name',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE ROSTER ROLE where the coverage of a translation is asked',
      fn: async () => {
        const source = await readFile(new URL('coverage-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['roster',],);
      },
    },),

    it({
      name: 'NAMES THE ROSTER ROLE where a block is reviewed, and where its reviewers are judged',
      fn: async () => {
        const source = await readFile(new URL('archive-block-review-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['roster',],);
        expect(rolesNamedAtCalls({ source, callee: 'decideBestCandidate', },),).toEqual(['roster',],);
      },
    },),

    it({
      name: 'NAMES THE ROSTER ROLE where the naturalness of a block is reviewed',
      fn: async () => {
        const source = await readFile(new URL('absolute-naturalness-review-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'runWindowedRounds', },),).toEqual(['roster',],);
      },
    },),
  ],
},);

//endregion Archive role tests
