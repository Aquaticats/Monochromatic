/**
 Tests for the second signature at block scale.

 `doc/decision/translation-repair-absence-verdict.md` requires two independent
 readings before anything is written into a page. Subdivision supplies the
 first, a pairing that left an original unplaced; this supplies the second, a
 page measurably shorter than its source predicts. Both are required, and
 these cases pin that the second one can actually refuse.

 WHAT WOULD GO WRONG WITHOUT THEM. A gate admitting everything is
 indistinguishable from no gate, and a gate admitting nothing silently
 disables the whole insertion path; neither raises anything, so both look
 identical from outside. The cases in this file exercise both directions on one
 fixture.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { admitInsertions, } from '../dist/final/node/index.mjs';

/**
 Original standing in for a page.
 */
const SOURCE = '橘猫在窗台上睡了整个下午，阳光把它的毛烤得暖烘烘的。';

/**
 Builds a slice whose translation side is a place rather than text.

 @param sliceIndex - position this slice holds

 @param sourceText - original with no translation beside it

 @returns Slice the admission can weigh

 @example
 ```ts
 const slice = anchoredSlice({ sliceIndex: 1, sourceText: '第一只猫。', },);
 ```
 */
function anchoredSlice(
  {
    sliceIndex,
    sourceText,
  }: {
    readonly sliceIndex: number;
    readonly sourceText: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: sourceText.length,
      text: sourceText,
    },
    target: makeInsertionChunk({
      sliceIndex,
      offset: 0,
    },),
  };
}

/**
 Builds a slice both sides of which carry text, which the gate never weighs.

 @param sliceIndex - position this slice holds

 @returns Ordinary paired slice

 @example
 ```ts
 const slice = pairedSlice({ sliceIndex: 0, },);
 ```
 */
function pairedSlice({ sliceIndex, }: { readonly sliceIndex: number; },): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 1,
      text: '猫',
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 3,
      text: 'cat',
    },
  };
}

await describe({
  name: admitInsertions.name,
  children: [
    it({
      name: 'ADMITS an anchored slice into a page that is measurably short, which is the case '
        + 'the whole insertion path exists for',
      fn: async () => {
        expect([...admitInsertions({
          slices: [
            pairedSlice({ sliceIndex: 0, },),
            anchoredSlice({
              sliceIndex: 1,
              sourceText: '第一只猫。',
            },),
          ],
          sourceText: SOURCE,
          targetText: 'The cat slept.',
        },),],)
          .toStrictEqual([ 1, ],);
      },
    },),

    it({
      name: 'REFUSES the same slice when the page carries what its source predicts, since a page '
        + 'of ordinary length more likely merged that passage than dropped it',
      fn: async () => {
        expect([...admitInsertions({
          slices: [
            pairedSlice({ sliceIndex: 0, },),
            anchoredSlice({
              sliceIndex: 1,
              sourceText: '第一只猫。',
            },),
          ],
          sourceText: SOURCE,
          targetText: 'x'.repeat(SOURCE.length * 3,),
        },),],)
          .toStrictEqual([],);
      },
    },),

    it({
      name: 'SPENDS ONE BUDGET ACROSS THE WHOLE PAGE rather than one per slice, so a page short '
        + 'by room for one passage never admits two. Letting each section spend the page shortfall '
        + 'separately would write in several times what the page is missing',
      fn: async () => {
        /**
         A page short by roughly one of these passages.
         */
        const targetText = 'x'.repeat(Math.floor(SOURCE.length * 2,),);

        expect([...admitInsertions({
          slices: [
            anchoredSlice({
              sliceIndex: 0,
              sourceText: '第一只猫在这里。',
            },),
            anchoredSlice({
              sliceIndex: 1,
              sourceText: '第二只猫在那里。',
            },),
            anchoredSlice({
              sliceIndex: 2,
              sourceText: '第三只猫在门口。',
            },),
          ],
          sourceText: SOURCE,
          targetText,
        },),].length,)
          .toBeLessThan(3,);
      },
    },),

    it({
      name: 'NAMES POSITIONS RATHER THAN CHUNK INDICES, since a stamped index means different '
        + 'things depending on who stamped it and the caller looks slices up by where they sit',
      fn: async () => {
        expect([...admitInsertions({
          slices: [
            pairedSlice({ sliceIndex: 40, },),
            anchoredSlice({
              sliceIndex: 41,
              sourceText: '第一只猫。',
            },),
          ],
          sourceText: SOURCE,
          targetText: '',
        },),],)
          .toStrictEqual([ 1, ],);
      },
    },),

    it({
      name: 'ADMITS NOTHING from a document with no anchored slice at all, rather than reading an '
        + 'empty proposal list as licence',
      fn: async () => {
        expect([...admitInsertions({
          slices: [
            pairedSlice({ sliceIndex: 0, },),
            pairedSlice({ sliceIndex: 1, },),
          ],
          sourceText: SOURCE,
          targetText: '',
        },),],)
          .toStrictEqual([],);
      },
    },),
  ],
},);
