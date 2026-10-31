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
      name: 'ASKS THE CONSISTENCY GATHER IN NO ROLE OF ITS OWN: the role it asks in is its caller\'s',
      fn: async () => {
        const source = await readFile(new URL('pass-consistency-gather.ts', import.meta.url,), 'utf8',);
        expect(source.includes('role: \'late-judge\'',),).toBe(false,);
      },
    },),

    it({
      name: 'TAKES THE ROLE OF THE CONSISTENCY GATHER FROM ITS CALLER, and each caller names the role of the bench it '
        + 'asks: the choice and the run-off of a page are asked of the late judges, the writers of a carry of the '
        + 'translators',
      fn: async () => {
        const gather = await readFile(new URL('pass-consistency-gather.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source: gather, callee: 'gatherStageVoices', },),).toEqual(['role',],);
        const named = async (file: string,): Promise<readonly string[]> => rolesNamedAtCalls({
          source: await readFile(new URL(file, import.meta.url,), 'utf8',),
          callee: 'gatherThroughCache',
        },);
        expect(await named('pass-consistency-ask.ts',),).toEqual(['late-judge',],);
        expect(await named('pass-consistency-run-off.ts',),).toEqual(['late-judge',],);
        expect(await named('pass-consistency-carry-ask.ts',),).toEqual(['translator', 'late-judge',],);
      },
    },),
  ],
},);

//endregion Pass role tests
