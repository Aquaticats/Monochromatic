/**
 Tests for the one apostrophe fold the correction and uncertainty readers share.
 
 The fold is a policy, not a helper: it decides which input characters count as
 the ASCII apostrophes every phrase list in this package is written in. These
 cases pin the fold itself so a change to it (another apostrophe codepoint, or
 folding to something other than `'`) is answered here first.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  LEFT_SINGLE_QUOTE,
  normaliseApostrophes,
  RIGHT_SINGLE_QUOTE,
} from './normalise-apostrophes.ts';

await describe({
  name: normaliseApostrophes.name,
  children: [
    it({
      name: 'folds the left single quotation mark to an ASCII apostrophe',
      fn: async () => {
        expect(normaliseApostrophes(`${LEFT_SINGLE_QUOTE}quoted${LEFT_SINGLE_QUOTE}`,),)
          .toBe("'quoted'",);
      },
    },),
    it({
      name: 'folds the right single quotation mark to an ASCII apostrophe',
      fn: async () => {
        expect(normaliseApostrophes(`that${RIGHT_SINGLE_QUOTE}s wrong`,),)
          .toBe("that's wrong",);
      },
    },),
    it({
      name: 'folds both marks in one pass and keeps the rest of the text',
      fn: async () => {
        expect(
          normaliseApostrophes(`${LEFT_SINGLE_QUOTE}don${RIGHT_SINGLE_QUOTE}t do that${LEFT_SINGLE_QUOTE} he said`,),
        ).toBe("'don't do that' he said",);
      },
    },),
    it({
      name: 'leaves text that carries no curly apostrophe unchanged',
      fn: async () => {
        expect(normaliseApostrophes("that's already straight",),)
          .toBe("that's already straight",);
      },
    },),
    it({
      name: 'returns the empty string unchanged',
      fn: async () => {
        expect(normaliseApostrophes('',),)
          .toBe('',);
      },
    },),
  ],
},);
