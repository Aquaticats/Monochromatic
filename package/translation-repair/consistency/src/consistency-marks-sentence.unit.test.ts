/**
 Checks the sentence that names the marks at the head of a consistency user turn (R4): it carries the marks of the
 call, whatever length a stray run in a shown text made them, and names the word its sheet calls the Chinese text.

 @module
 */

import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';
import { marksSentenceOf, } from '../dist/final/node/index.mjs';

await describe({
  name: marksSentenceOf.name,
  children: [
    it({
      name: 'NAMES BOTH MARKS as the call chose them and says they are not part of the text',
      fn: async () => {
        expect(marksSentenceOf({ marks: { open: '⟦', close: '⟧', }, shown: 'paragraph', },),).toBe(
          'In each paragraph the repeated wording is enclosed between ⟦ and ⟧; the marks are not part of the text.',
        );
        expect(marksSentenceOf({ marks: { open: '⟦⟦⟦', close: '⟧⟧⟧', }, shown: 'ORIGINAL', },),).toBe(
          'In each ORIGINAL the repeated wording is enclosed between ⟦⟦⟦ and ⟧⟧⟧; the marks are not part of the text.',
        );
      },
    },),
  ],
},);
