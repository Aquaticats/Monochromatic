/**
 Tests for the rule that refuses a disputed wording on a slice whose archive
 rendering the repair lane's adjudicators disputed: a candidate is compared
 with each wording in all but whitespace, and a candidate with nothing but
 whitespace in it is no wording at all.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { disputedWordingFindings, } from '../dist/final/node/index.mjs';

/**
 Archive wording the adjudicators disputed.
 */
const ARCHIVE_WORDING = 'The cat is doing the sleeping on the windowsill.';

await describe({
  name: disputedWordingFindings.name,
  children: [
    it({
      name: 'REFUSES a candidate that is a disputed wording in all but its whitespace, one finding per wording it is',
      fn: async () => {
        expect(disputedWordingFindings({
          candidateText: 'The cat is doing\nthe sleeping on  the windowsill.',
          disputedWordings: [
            {
              text: ARCHIVE_WORDING,
              reason: 'the archive rendering the adjudicators disputed',
            },
            {
              text: 'The cat naps.',
              reason: 'the repair lane\'s text',
            },
          ],
        },),).toEqual([
          'Your translation is the archive rendering the adjudicators disputed, which cannot ship on this slice; '
            + 'render the ORIGINAL afresh.',
        ],);
      },
    },),
    it({
      name: 'PASSES a candidate that holds nothing but whitespace, though a disputed wording holds nothing either',
      fn: async () => {
        expect(disputedWordingFindings({
          candidateText: ' \n\t ',
          disputedWordings: [
            {
              text: '',
              reason: 'the repair lane\'s empty text',
            },
          ],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'PASSES any candidate on a slice with no disputed wording',
      fn: async () => {
        expect(disputedWordingFindings({
          candidateText: ARCHIVE_WORDING,
          disputedWordings: [],
        },),).toEqual([],);
      },
    },),
  ],
},);
