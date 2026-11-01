/**
 Tests for the refusal a window trial makes of a drawn slice it cannot try.

 The class declares its sentence safe to repeat, so the pick's warning can
 print it whole; the sentence is built from an entry id and a slice index.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { namesWithoutQuoting, } from '@monochromatic-dev/translation-repair-document-structure';
import { TrialSliceRefusalError, } from '../../dist/final/node/index.mjs';

await describe({
  name: TrialSliceRefusalError.name,
  children: [
    it({
      name: 'STATES A SLICE THE PREPARED LIST DOES NOT CARRY by entry and index, and declares the sentence safe',
      fn: async () => {
        /**
         Refusal for an index the draw named and preparation lacks.
         */
        const refusal = new TrialSliceRefusalError({
          entryId: 'Mittens',
          sliceIndex: 99,
          kind: 'slice-missing',
        },);

        expect(String(refusal,),).toBe(
          'TrialSliceRefusalError: Mittens has no slice 99; the draw and the preparation disagree, which means '
            + 'they were made from different text',
        );
        expect(namesWithoutQuoting(refusal,),).toBe(true,);
      },
    },),

    it({
      name: 'STATES A SLICE WHOSE NEIGHBOURING ORIGINAL HOLDS NO TEXT by entry and index',
      fn: async () => {
        /**
         Refusal for a slice with no window to widen to.
         */
        const refusal = new TrialSliceRefusalError({
          entryId: 'Mittens',
          sliceIndex: 0,
          kind: 'window-empty',
        },);

        expect(String(refusal,),).toBe(
          'TrialSliceRefusalError: Mittens/0 has no neighbouring section carrying text, so its wide arm would be '
            + 'its narrow arm and the pair would report a false null',
        );
      },
    },),
  ],
},);
