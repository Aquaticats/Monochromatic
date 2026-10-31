/**
 Test for the role the block-section pairing names (ledger B1079, B1100).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS ROLE MAY TAKE ("Honour holds (Recommended)", 2026-10-30), so a call that asks a
 bench names the role of that bench. The pairing of sections is asked of the run roster and nothing narrower.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Stage role tests

await describe({
  name: 'the role the stage package names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE ROSTER ROLE where the sections of a page are paired',
      fn: async () => {
        const source = await readFile(new URL('pair-sections-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['roster',],);
      },
    },),
  ],
},);

//endregion Stage role tests
