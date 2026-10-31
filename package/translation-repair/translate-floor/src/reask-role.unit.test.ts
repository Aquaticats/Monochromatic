/**
 Test for the seat the repair of an invalid candidate names (ledger B1079).

 A RUNAWAY IS RE-ASKED OF A MODEL ITS CARD DOES NOT HOLD OUT OF THE ROLE ("Honour holds (Recommended)", 2026-10-30), and
 a request that names no seat is re-asked of its own model alone. The author of an invalid candidate is asked about its
 findings in the translator role, with the authors of the slate as the bench, so a runaway author is re-asked of a model
 that is none of them.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Translate floor role tests

await describe({
  name: 'the seat the repair of an invalid candidate names',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE TRANSLATOR ROLE where an author is asked about the findings on its candidate',
      fn: async () => {
        const source = await readFile(new URL('translate-repair.ts', import.meta.url,), 'utf8',);
        expect(source.includes('reaskSeat: {',),).toBe(true,);
        expect(source.includes('role: \'translator\',',),).toBe(true,);
      },
    },),
  ],
},);

//endregion Translate floor role tests
