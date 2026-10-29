/**
 Tests for where the words of each replacement land once assembly has written every edit: the text a content span is
 written over with, and the final place of each edit's words given the growth of the edits written after it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contentWrite,
  placementsOf,
  type Write,
} from '../dist/final/node/index.mjs';

/**
 A content write over a span, as assembly records it.

 @param startOffset - where the span starts
 @param replacedLength - how long the text it held was
 @param orderIndex - slice written
 @param text - text written
 @returns Write, whose words fill all the text it wrote and start at its start

 @example
 ```ts
 const write = contentOver({ startOffset: 20, replacedLength: 5, orderIndex: 2, text: 'LONGER', },);
 ```
 */
function contentOver(
  {
    startOffset,
    replacedLength,
    orderIndex,
    text,
  }: {
    readonly startOffset: number;
    readonly replacedLength: number;
    readonly orderIndex: number;
    readonly text: string;
  },
): Write {
  return {
    edit: { kind: 'content', startOffset, endOffset: startOffset + replacedLength, orderIndex, text, },
    written: { text, placed: [{ fragment: 0, englishAt: 0, from: 0, to: text.length, },], },
    replacedLength,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contentWrite.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE SPAN\'S LINE-ENDING EDGES AND DROPS THE LANE\'S, and places the lane\'s first character where it would '
            + 'stand: before the written text when the lane carried more line endings at its start than the span, after it when fewer',
          fn: async () => {
            expect([
              contentWrite({ replaced: 'The cat sleeps.', text: '\nThe cat naps.\n', },),
              contentWrite({ replaced: '\n\nTwo.', text: 'Three.', },),
              contentWrite({ replaced: 'Two.', text: 'Three.', },),
            ],).toEqual([
              { text: 'The cat naps.', placed: [{ fragment: 0, englishAt: -1, from: 0, to: 13, },], },
              { text: '\n\nThree.', placed: [{ fragment: 0, englishAt: 2, from: 0, to: 8, },], },
              { text: 'Three.', placed: [{ fragment: 0, englishAt: 0, from: 0, to: 6, },], },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: placementsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MOVES THE WORDS OF AN EDIT BY THE NET GROWTH OF EVERY EDIT WRITTEN AFTER IT, which stands before it, and lists '
            + 'them in the order of the translation',
          fn: async () => {
            /**
             The writes in the order they were made: from the end of the document to its start.
             */
            const writes = [
              contentOver({ startOffset: 20, replacedLength: 5, orderIndex: 2, text: 'LONGER', },),
              contentOver({ startOffset: 0, replacedLength: 4, orderIndex: 0, text: 'ab', },),
            ];
            expect(placementsOf({ writes, },),).toEqual([
              { sliceIndex: 0, start: 0, end: 2, englishAt: 0, },
              { sliceIndex: 2, start: 18, end: 24, englishAt: 18, },
            ],);
          },
        },),
        it({
          name: 'PLACES EACH FRAGMENT OF AN INSERTION UNDER THE SLICE IT WAS WRITTEN FOR, and nothing for a fragment that wrote nothing',
          fn: async () => {
            /**
             An insertion of two fragments, the second of which shows a reader nothing, and a content write before it.
             */
            const writes: readonly Write[] = [
              {
                edit: {
                  kind: 'insertion',
                  startOffset: 10,
                  endOffset: 10,
                  orderIndex: 1,
                  fragments: ['\u{200B}', '\n\nShe naps.',],
                  sliceIndexes: [1, 3,],
                },
                written: {
                  text: '\n\nShe naps.\n\n',
                  placed: [{ fragment: 1, englishAt: 0, from: 2, to: 11, },],
                },
                replacedLength: 0,
              },
              contentOver({ startOffset: 0, replacedLength: 4, orderIndex: 0, text: 'ab', },),
            ];
            expect(placementsOf({ writes, },),).toEqual([
              { sliceIndex: 0, start: 0, end: 2, englishAt: 0, },
              { sliceIndex: 3, start: 10, end: 19, englishAt: 8, },
            ],);
          },
        },),
        it({
          name: 'GIVES NOTHING FOR NO WRITE',
          fn: async () => {
            expect(placementsOf({ writes: [], },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
