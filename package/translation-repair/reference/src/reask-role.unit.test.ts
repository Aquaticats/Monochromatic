/**
 Tests for the role the reference attestation names when it asks a bench (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), so
 every call that asks a bench names the role of that bench. The attestation and its confirmation are asked of the run
 roster.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Reference role tests

await describe({
  name: 'the role the reference attestation names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE ROSTER ROLE where cited references are attested, and where the attestation is confirmed',
      fn: async () => {
        const source = await readFile(new URL('reference-attest-stage.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'gatherStageVoices', },),).toEqual(['roster', 'roster',],);
      },
    },),
  ],
},);

//endregion Reference role tests
