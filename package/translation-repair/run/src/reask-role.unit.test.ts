/**
 Test for the role the judge fidelity probe names (ledger B1079, B1100).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS ROLE MAY TAKE ("Honour holds (Recommended)", 2026-10-30), so a call that asks a
 bench names the role of that bench. The probe seats roster models before any role holds them.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolesNamedAtCalls, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reask-role-call.test-fixture.ts';

//region Run role tests

await describe({
  name: 'the role the run package names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE ROSTER ROLE where the judge fidelity probe asks its judges',
      fn: async () => {
        const source = await readFile(new URL('judge-fidelity.ts', import.meta.url,), 'utf8',);
        expect(rolesNamedAtCalls({ source, callee: 'selectBestCandidate', },),).toEqual(['roster',],);
      },
    },),
  ],
},);

//endregion Run role tests
