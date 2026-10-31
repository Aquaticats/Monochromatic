/**
 Tests for the role each asking of a bench names in this package (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench: the panel is asked of the wide judges, and the probe of
 introduced defects of the checkers.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Adjudication role tests

await describe({
  name: 'the role the adjudication names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE WIDE-JUDGE ROLE where the panel is asked',
      fn: async () => {
        const source = await readFile(new URL('./panel-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['wide-judge',],);
      },
    },),

    it({
      name: 'NAMES THE CHECKER ROLE where the probe of introduced defects is asked',
      fn: async () => {
        const source = await readFile(new URL('./introduced-defect-probe.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['checker',],);
      },
    },),
  ],
},);

//endregion Adjudication role tests
