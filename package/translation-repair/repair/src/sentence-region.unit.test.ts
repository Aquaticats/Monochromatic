/**
 Tests for the sentence region around a span: the whole sentences a defect sits in, which is the region an accepted
 issue's critic would quote and an editor may rewrite.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { sentenceRegionAround, } from '../dist/final/node/index.mjs';

/**
 Three sentences in one paragraph.
 */
const THREE = 'One cat naps. Two cats play here. Three cats eat.';

/**
 Reads the region a span widens to, as the text it covers.

 @param text - passage the span is in

 @param startOffset - first character of the span

 @param endOffset - end of the span, exclusive

 @returns The text of the region

 @example
 ```ts
 const region = regionText({ text: THREE, startOffset: 20, endOffset: 24, },);
 ```
 */
function regionText(
  {
    text,
    startOffset,
    endOffset,
  }: {
    readonly text: string;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): string {
  /**
   Offsets of the region.
   */
  const region = sentenceRegionAround({
    text,
    startOffset,
    endOffset,
  },);
  return text.slice(
    region.startOffset,
    region.endOffset,
  );
}

await describe({
  name: sentenceRegionAround.name,
  children: [
    it({
      name: 'WIDENS a span inside the middle sentence to that whole sentence, terminator in and the separators out',
      fn: async () => {
        expect(regionText({
          text: THREE,
          startOffset: THREE.indexOf('play',),
          endOffset: THREE.indexOf('play',) + 'play'.length,
        },),).toBe('Two cats play here.',);
      },
    },),

    it({
      name: 'KEEPS a span that is already a whole sentence, whichever side of it carries the separator, so a borrowed '
        + 'sentence with its leading space is the sentence and nothing more',
      fn: async () => {
        /**
         Where the middle sentence begins.
         */
        const at = THREE.indexOf('Two',);
        /**
         The middle sentence.
         */
        const sentence = 'Two cats play here.';
        expect(regionText({
          text: THREE,
          startOffset: at,
          endOffset: at + sentence.length,
        },),).toBe(sentence,);
        expect(regionText({
          text: THREE,
          startOffset: at - 1,
          endOffset: at + sentence.length,
        },),).toBe(sentence,);
        expect(regionText({
          text: THREE,
          startOffset: at,
          endOffset: at + sentence.length + 1,
        },),).toBe(sentence,);
      },
    },),

    it({
      name: 'STARTS at the head of the text for the first sentence and ENDS at the end of the text for a last '
        + 'sentence that has no terminator',
      fn: async () => {
        expect(regionText({
          text: THREE,
          startOffset: 4,
          endOffset: 7,
        },),).toBe('One cat naps.',);
        /**
         Text whose last sentence is never closed.
         */
        const open = 'One cat naps. The second cat waits';
        expect(regionText({
          text: open,
          startOffset: open.indexOf('waits',),
          endOffset: open.length,
        },),).toBe('The second cat waits',);
      },
    },),

    it({
      name: 'COVERS EVERY SENTENCE a span reaches across',
      fn: async () => {
        expect(regionText({
          text: THREE,
          startOffset: THREE.indexOf('naps',),
          endOffset: THREE.indexOf('play',),
        },),).toBe('One cat naps. Two cats play here.',);
      },
    },),

    it({
      name: 'STOPS at a line break as at a terminator, so a line with no terminator is its own region',
      fn: async () => {
        /**
         Heading, a line without a terminator, then another.
         */
        const lines = 'Heading\n\nA line without a stop\nNext line.';
        expect(regionText({
          text: lines,
          startOffset: lines.indexOf('without',),
          endOffset: lines.indexOf('without',) + 'without'.length,
        },),).toBe('A line without a stop',);
      },
    },),

    it({
      name: 'READS the terminators of the original\'s language too',
      fn: async () => {
        /**
         Two Chinese sentences.
         */
        const original = '猫猫在窗台上睡觉。它醒了！';
        expect(regionText({
          text: original,
          startOffset: original.indexOf('醒',),
          endOffset: original.indexOf('醒',) + 1,
        },),).toBe('它醒了！',);
      },
    },),

    it({
      name: 'RETURNS a zero-width span as it came, since a point between sentences belongs to none of them',
      fn: async () => {
        expect(sentenceRegionAround({
          text: THREE,
          startOffset: 14,
          endOffset: 14,
        },),).toStrictEqual({
          startOffset: 14,
          endOffset: 14,
        },);
      },
    },),
  ],
},);
