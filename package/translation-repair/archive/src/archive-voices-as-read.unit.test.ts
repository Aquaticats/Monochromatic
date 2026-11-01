/**
 Tests that a replacement showing a reader nothing is read as the empty
 removal, once, before any later step asks what it is (ledger B40).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { voicesAsRead, } from '../dist/final/node/index.mjs';

/** Seat the invented replies come from. */
const SEAT = 'hf:cat/Cat-A' as unknown as RosterModelId;

/**
 One invented reply.

 @param replacementText - what the reviewer wrote as the replacement

 @returns Heard voice
 */
function voiceWith({ replacementText, }: { readonly replacementText: string; },): ReturnType<typeof voicesAsRead>[number] {
  return {
    modelId: SEAT,
    value: {
      disposition: 'revise',
      sourceQuote: '',
      replacementText,
      assessment: 'The block carries nothing the original says.',
    },
  };
}

await describe({
  name: voicesAsRead.name,
  children: [
    it({
      name: 'READS A REPLACEMENT THAT SHOWS NOTHING AS EMPTY: spaces, a zero-width space and a Hangul filler',
      fn: async () => {
        for (const replacementText of [
          '   ',
          '\u200B',
          '\u3164',
        ]) {
          const [read,] = voicesAsRead({ voices: [voiceWith({ replacementText, },),], },);
          expect(read?.value.replacementText,).toBe('',);
        }
      },
    },),
    it({
      name: 'LEAVES A REPLACEMENT THAT SHOWS TEXT AS THE REVIEWER WROTE IT, and the rest of the reply with it',
      fn: async () => {
        const voice = voiceWith({ replacementText: 'The cat sleeps.', },);
        expect(voicesAsRead({ voices: [voice,], },),).toEqual([voice,],);
      },
    },),
    it({
      name: 'KEEPS THE REST OF A REPLY WHOSE REPLACEMENT IS READ AS EMPTY',
      fn: async () => {
        const [read,] = voicesAsRead({ voices: [voiceWith({ replacementText: ' ', },),], },);
        expect(read?.modelId,).toBe(SEAT,);
        expect(read?.value.disposition,).toBe('revise',);
        expect(read?.value.assessment,).toBe('The block carries nothing the original says.',);
      },
    },),
  ],
},);
