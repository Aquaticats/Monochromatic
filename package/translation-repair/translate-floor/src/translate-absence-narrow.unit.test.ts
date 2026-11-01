/**
 Tests the narrowing every catch that acts on an absence alone shares: an
 absence comes back as itself, and anything else leaves as it came, an error
 or a value that is not one.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BlankSelectionError,
  requireTranslateAbsence,
  TranslateAbsenceError,
} from '../dist/final/node/index.mjs';

await describe({
  name: requireTranslateAbsence.name,
  children: [
    it({
      name: 'RETURNS AN ABSENCE as itself, reason and findings with it',
      fn: async () => {
        /**
         Absence a round raised.
         */
        const absence = new TranslateAbsenceError({
          reason: 'declined-rejection',
          findings: ['every candidate adds a nap the original never takes',],
        },);
        expect(requireTranslateAbsence({ error: absence, },),).toBe(absence,);
      },
    },),
    it({
      name: 'RETHROWS ANYTHING ELSE by identity: another error, and a value that is not an error at all',
      fn: async () => {
        /**
         A fault of judging that is not an absence.
         */
        const blank = new BlankSelectionError({ findings: [], },);

        /**
         A thrown value that is no error.
         */
        const purr = 'purr';
        expect(caught(function narrowBlank() {
          requireTranslateAbsence({ error: blank, },);
        },),).toBe(blank,);
        expect(caught(function narrowPurr() {
          requireTranslateAbsence({ error: purr, },);
        },),).toBe(purr,);
      },
    },),
  ],
},);
