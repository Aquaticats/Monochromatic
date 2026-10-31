/**
 Tests for the role the rendering audit names when it asks a bench (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. The audit is asked of the checkers.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Rendering audit role tests

await describe({
  name: 'the role the rendering audit names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE CHECKER ROLE where a rendering is audited',
      fn: async () => {
        const source = await readFile(new URL('rendering-audit.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['checker',],);
      },
    },),
  ],
},);

//endregion Rendering audit role tests
