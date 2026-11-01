/**
 Tests for the check that says target spans can be written back.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assertPlacementLayout,
  PlacementLayoutError,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';
import { anchorAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/insertion-anchor-pair.test-fixture.ts';

/**
 Document the spans index into.
 */
const TARGET_TEXT = 'The cat sleeps.\n\nShe purrs.';

/**
 Builds one pair covering a span of {@link TARGET_TEXT}.

 @param sliceIndex - position of this slice

 @param startOffset - absolute start

 @param endOffset - absolute exclusive end

 @returns Pair whose target side carries that span

 @example
 ```ts
 const pair = spanAt({ sliceIndex: 0, startOffset: 0, endOffset: 15, },);
 ```
 */
function spanAt(
  {
    sliceIndex,
    startOffset,
    endOffset,
  }: {
    readonly sliceIndex: number;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): ChunkPair {
  /**
   Both sides, since only the target one is read here.
   */
  const side = {
    sliceIndex,
    nodes: [],
    startOffset,
    endOffset,
    text: TARGET_TEXT.slice(
      startOffset,
      endOffset,
    ),
  };
  return {
    source: side,
    target: side,
  };
}

/**
 Where the second paragraph starts.
 */
const SECOND_START = TARGET_TEXT.indexOf('She purrs.',);

await describe({
  name: assertPlacementLayout.name,
  children: [
    it({
      name: 'accepts what a real preparation produces, which is the case that has to keep passing: every '
        + 'span there comes from a disjoint run of nodes, so the rule holds by construction and a guard '
        + 'nobody can satisfy is worse than no guard',
      fn: async () => {
        /** Two sections on each side. */
        const prepared = prepareDocumentPair({
          sourceText: '## 简介\n\n猫猫喜欢晒太阳。\n\n## 习惯\n\n它每天都在窗边睡觉。\n',
          targetText: '## Intro\n\nThe cat likes the sun.\n\n## Habits\n\nIt sleeps by the window daily.\n',
        },);
        expect(function checkRealPreparation() {
          assertPlacementLayout({
            slices: prepared.slices,
            targetText: prepared.targetText,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'accepts every legal shape an insertion makes: several at one boundary, one at a span`s start '
        + 'meaning before it, and one at a span`s end. Each is a placement the translate lane produces, '
        + 'and refusing any of them would refuse the work rather than a defect',
      fn: async () => {
        expect(function checkLegalShapes() {
          assertPlacementLayout({
            slices: [
              spanAt({
                sliceIndex: 0,
                startOffset: 0,
                endOffset: 15,
              },),
              anchorAt({
                sliceIndex: 1,
                offset: 15,
              },),
              anchorAt({
                sliceIndex: 2,
                offset: SECOND_START,
              },),
              anchorAt({
                sliceIndex: 3,
                offset: SECOND_START,
              },),
              spanAt({
                sliceIndex: 4,
                startOffset: SECOND_START,
                endOffset: TARGET_TEXT.length,
              },),
            ],
            targetText: TARGET_TEXT,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'REFUSES a span running past the document, which is the failure that produces plausible text '
        + 'rather than a diagnostic: slicing a string CLAMPS, so an assembly reading past the end returns '
        + 'a document nobody can tell is wrong from the output',
      fn: async () => {
        expect(function checkPastEnd() {
          assertPlacementLayout({
            slices: [
              {
                source: spanAt({
                  sliceIndex: 0,
                  startOffset: 0,
                  endOffset: 15,
                },).source,
                target: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: TARGET_TEXT.length + 40,
                  text: TARGET_TEXT,
                },
              },
            ],
            targetText: TARGET_TEXT,
          },);
        },).toThrow(PlacementLayoutError,);
      },
    },),
    it({
      name: 'REFUSES a span that ends before it starts, naming both offsets, since slicing reads it as an empty '
        + 'span and an assembly would take the mistake for an insertion',
      fn: async () => {
        /**
         What the check raised on the one span running from ten back to five.
         */
        const refusal = caught(function checkEndsBeforeStart() {
          assertPlacementLayout({
            slices: [
              spanAt({
                sliceIndex: 0,
                startOffset: 10,
                endOffset: 5,
              },),
            ],
            targetText: TARGET_TEXT,
          },);
        },);

        expect(refusal,).toBeInstanceOf(PlacementLayoutError,);
        expect(String(refusal,),).toBe(
          'PlacementLayoutError: slice at position 0 ends at 5 before it starts at 10, which slicing reads as an '
            + 'empty span rather than as the mistake it is',
        );
        expect((refusal as PlacementLayoutError).fault,).toEqual({
          kind: 'backwards',
          startOffset: 10,
          endOffset: 5,
        },);
      },
    },),
    it({
      name: 'REFUSES a placement that moves BACKWARDS, which is one statement covering overlap too: two '
        + 'spans that overlap, two starting at one offset, and a list sorted differently from its slices '
        + 'are all the same defect seen from different sides',
      fn: async () => {
        /**
         What checkBackwards raised, read for its class as well as its wording.
         */
        const refusalOfCheckBackwards = caught(function checkBackwards() {
          assertPlacementLayout({
            slices: [
              spanAt({
                sliceIndex: 0,
                startOffset: SECOND_START,
                endOffset: TARGET_TEXT.length,
              },),
              spanAt({
                sliceIndex: 1,
                startOffset: 0,
                endOffset: 15,
              },),
            ],
            targetText: TARGET_TEXT,
          },);
        },);

        expect(refusalOfCheckBackwards,).toBeInstanceOf(PlacementLayoutError,);
        expect((refusalOfCheckBackwards as Error).message,).toContain('would move or overwrite',);
      },
    },),
    it({
      name: 'REFUSES an anchor strictly INSIDE a span, and an anchor that follows a span it shares a '
        + 'boundary with. Writing either one lands text in the middle of a passage the same run is '
        + 'replacing, and which of them wins depends on sort order',
      fn: async () => {
        expect(function checkAnchorInside() {
          assertPlacementLayout({
            slices: [
              spanAt({
                sliceIndex: 0,
                startOffset: 0,
                endOffset: 15,
              },),
              anchorAt({
                sliceIndex: 1,
                offset: 4,
              },),
            ],
            targetText: TARGET_TEXT,
          },);
        },).toThrow(PlacementLayoutError,);
        expect(function checkAnchorAfterItsSpan() {
          assertPlacementLayout({
            slices: [
              spanAt({
                sliceIndex: 0,
                startOffset: 0,
                endOffset: 15,
              },),
              anchorAt({
                sliceIndex: 1,
                offset: 0,
              },),
            ],
            targetText: TARGET_TEXT,
          },);
        },).toThrow(PlacementLayoutError,);
      },
    },),
    it({
      name: 'REFUSES content covering nothing, since an empty span and a place where text is missing are '
        + 'indistinguishable by their offsets, and only one of them may be written INTO',
      fn: async () => {
        /**
         What checkHollowContent raised, read for its class as well as its wording.
         */
        const refusalOfCheckHollowContent = caught(function checkHollowContent() {
          assertPlacementLayout({
            slices: [
              spanAt({
                sliceIndex: 0,
                startOffset: 15,
                endOffset: 15,
              },),
            ],
            targetText: TARGET_TEXT,
          },);
        },);

        expect(refusalOfCheckHollowContent,).toBeInstanceOf(PlacementLayoutError,);
        expect((refusalOfCheckHollowContent as Error).message,).toContain('content covering nothing',);
      },
    },),
    it({
      name: 'REFUSES text the document does not hold at those offsets, which is how slices cut from ANOTHER '
        + 'document reach assembly: the offsets are all in range, every span is ordered, and the result '
        + 'is a confident splice into a passage nobody read',
      fn: async () => {
        /**
         What checkForeignSlices raised, read for its class as well as its wording.
         */
        const refusalOfCheckForeignSlices = caught(function checkForeignSlices() {
          assertPlacementLayout({
            slices: [
              {
                source: anchorAt({
                  sliceIndex: 0,
                  offset: 0,
                },).source,
                target: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 15,
                  text: 'The dog barks.!',
                },
              },
            ],
            targetText: TARGET_TEXT,
          },);
        },);

        expect(refusalOfCheckForeignSlices,).toBeInstanceOf(PlacementLayoutError,);
        expect((refusalOfCheckForeignSlices as Error).message,).toContain('cut from another document',);
      },
    },),
    it({
      name: 'REFUSES an anchor that claims to cover text, and an offset that is not a whole number. Both '
        + 'are values the constructor cannot produce and the structural type still permits',
      fn: async () => {
        /**
         What checkFatAnchor raised, read for its class as well as its wording.
         */
        const refusalOfCheckFatAnchor = caught(function checkFatAnchor() {
          assertPlacementLayout({
            slices: [
              {
                source: anchorAt({
                  sliceIndex: 0,
                  offset: 0,
                },).source,
                target: {
                  kind: 'insertion',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 15,
                  text: 'The cat sleeps.',
                },
              },
            ],
            targetText: TARGET_TEXT,
          },);
        },);

        expect(refusalOfCheckFatAnchor,).toBeInstanceOf(PlacementLayoutError,);
        expect((refusalOfCheckFatAnchor as Error).message,).toContain('a place covers none of the three',);
        /**
         What checkFractionalOffset raised, read for its class as well as its wording.
         */
        const refusalOfCheckFractionalOffset = caught(function checkFractionalOffset() {
          assertPlacementLayout({
            slices: [
              {
                source: anchorAt({
                  sliceIndex: 0,
                  offset: 0,
                },).source,
                target: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 1 / 2,
                  text: '',
                },
              },
            ],
            targetText: TARGET_TEXT,
          },);
        },);

        expect(refusalOfCheckFractionalOffset,).toBeInstanceOf(PlacementLayoutError,);
        expect((refusalOfCheckFractionalOffset as Error).message,).toContain('whole numbers',);
      },
    },),
  ],
},);
