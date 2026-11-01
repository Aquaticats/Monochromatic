/**
 Tests for writing disjoint edits into a text in one pass (ledger B70).

 THE REFERENCE IS THE OLD WAY: every edit spliced in one at a time, last first
 so earlier offsets held. The one-pass writer must give the same text for any
 disjoint edits, in any order they are given.

 Fixtures are cat-themed invention.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  OverlappingEditsError,
  type SpliceEdit,
  spliceDisjointEdits,
} from '../dist/final/node/index.mjs';

/**
 Text every case edits.
 */
const TEXT = 'the cat sat on the warm mat';

/**
 Writes edits one at a time, last first, as the callers did before.

 @param text - original text

 @param edits - disjoint edits

 @returns Text with every edit written

 @example
 ```ts
 const expected = lastFirst({ text: TEXT, edits, },);
 ```
 */
function lastFirst(
  {
    text,
    edits,
  }: {
    readonly text: string;
    readonly edits: readonly SpliceEdit[];
  },
): string {
  return edits
    .toSorted(function byStartDescending(left, right,): number {
      return right.start - left.start;
    },)
    .reduce(function writeOne(current, edit,): string {
      return current.slice(0, edit.start,) + edit.text + current.slice(edit.end,);
    }, text,);
}

await describe({
  name: spliceDisjointEdits.name,
  children: [
    it({
      name: 'WRITES replacements, an insertion and a deletion given in any order as splicing them last first did, '
        + 'with two edits meeting at a boundary',
      fn: async () => {
        /**
         Edits out of order: a replacement, an insertion, a deletion, and two
         replacements that meet where one ends.
         */
        const edits: readonly SpliceEdit[] = [
          {
            start: 24,
            end: 27,
            text: 'rug',
          },
          {
            start: 0,
            end: 0,
            text: 'Look: ',
          },
          {
            start: 19,
            end: 24,
            text: '',
          },
          {
            start: 4,
            end: 7,
            text: 'kitten',
          },
          {
            start: 7,
            end: 11,
            text: ' napped',
          },
        ];
        expect(spliceDisjointEdits({
          text: TEXT,
          edits,
        },),).toBe('Look: the kitten napped on the rug',);
        expect(spliceDisjointEdits({
          text: TEXT,
          edits,
        },),).toBe(lastFirst({
          text: TEXT,
          edits,
        },),);
      },
    },),
    it({
      name: 'RETURNS the text unchanged when there is nothing to write',
      fn: async () => {
        expect(spliceDisjointEdits({
          text: TEXT,
          edits: [],
        },),).toBe(TEXT,);
      },
    },),
    it({
      name: 'REFUSES two edits that share part of the text, naming both ranges',
      fn: async () => {
        /**
         The refusal for two replacements that overlap.
         */
        const refusal = caught(function overlapping(): void {
          spliceDisjointEdits({
            text: TEXT,
            edits: [
              {
                start: 8,
                end: 14,
                text: 'sprawled',
              },
              {
                start: 4,
                end: 11,
                text: 'kitten sat',
              },
            ],
          },);
        },);
        expect(refusal,).toBeInstanceOf(OverlappingEditsError,);
        expect(caughtValueText(refusal,),).toBe('edits 4-11 and 8-14 overlap or start at one offset',);
      },
    },),
    it({
      name: 'REFUSES two insertions at one offset, which the offsets give no order',
      fn: async () => {
        /**
         The refusal for two insertions before the same word.
         */
        const refusal = caught(function sameStart(): void {
          spliceDisjointEdits({
            text: TEXT,
            edits: [
              {
                start: 4,
                end: 4,
                text: 'grey ',
              },
              {
                start: 4,
                end: 4,
                text: 'old ',
              },
            ],
          },);
        },);
        expect(refusal,).toBeInstanceOf(OverlappingEditsError,);
        expect(caughtValueText(refusal,),).toBe('edits 4-4 and 4-4 overlap or start at one offset',);
      },
    },),
  ],
},);
