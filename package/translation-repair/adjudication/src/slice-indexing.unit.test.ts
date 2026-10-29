/**
 Tests for the slice-indexing invariant every cache key rests on.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assertSliceIndexing,
  reindexSlicePair,
  SliceIndexingError,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';

/**
 Builds one slice pair carrying the two indices under test.

 @param sourceIndex - index stamped on the original side

 @param targetIndex - index stamped on the translation side

 @returns Slice pair shaped like a prepared one

 @example
 ```ts
 const slice = sliceAt({ sourceIndex: 0, targetIndex: 0, },);
 ```
 */
function sliceAt(
  {
    sourceIndex,
    targetIndex,
  }: {
    readonly sourceIndex: number;
    readonly targetIndex: number;
  },
): {
  readonly source: {
    readonly sliceIndex: number;
    readonly nodes: readonly never[];
    readonly startOffset: number;
    readonly endOffset: number;
    readonly text: string;
  };
  readonly target: {
    readonly sliceIndex: number;
    readonly nodes: readonly never[];
    readonly startOffset: number;
    readonly endOffset: number;
    readonly text: string;
  };
} {
  return {
    source: {
      sliceIndex: sourceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 6,
      text: '猫在睡觉。',
    },
    target: {
      sliceIndex: targetIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 16,
      text: 'The cat sleeps.',
    },
  };
}

await describe({
  name: assertSliceIndexing.name,
  children: [
    it({
      name: 'accepts what a real preparation produces, which is the case that has to keep passing: the '
        + 'invariant is not new, it is simply never checked, and a guard nobody can satisfy is worse than '
        + 'no guard at all',
      fn: async () => {
        /** Two sections on each side, so subdivision runs more than once. */
        const prepared = prepareDocumentPair({
          sourceText: '## 简介\n\n猫猫喜欢晒太阳。\n\n## 习惯\n\n它每天都在窗边睡觉。\n',
          targetText: '## Intro\n\nThe cat likes the sun.\n\n## Habits\n\nIt sleeps by the window daily.\n',
        },);
        expect(prepared.slices
          .length,).toBeGreaterThan(0,);
        expect(function checkRealPreparation() {
          assertSliceIndexing({ slices: prepared.slices, },);
        },).not.toThrow();
        expect(prepared.slices
          .map(function toIndex(slice,): number {
            return slice.target
              .sliceIndex;
          },),).toEqual(prepared.slices
            .map(function toPosition(
              _slice,
              position,
            ): number {
              return position;
            },),);
      },
    },),
    it({
      name: 'REFUSES a slice whose two sides disagree, which section pairing can produce: a forced pair '
        + 'joins section 4 to section 6, and every consumer reads the TARGET side alone, so the source '
        + 'index is unchecked exactly where it would be wrong',
      fn: async () => {
        expect(function checkDisagreeingSides() {
          assertSliceIndexing({
            slices: [
              sliceAt({
                sourceIndex: 0,
                targetIndex: 0,
              },),
              sliceAt({
                sourceIndex: 3,
                targetIndex: 1,
              },),
            ],
          },);
        },).toThrow(SliceIndexingError,);
      },
    },),
    it({
      name: 'REFUSES a repeated index, which would let one cached slice answer for another. The cache key '
        + 'carries the index, so two slices sharing one index share a key, and the second would resume the '
        + 'first slice\'s settled text as its own',
      fn: async () => {
        /**
         What checkRepeatedIndex raised, read for its class as well as its wording.
         */
        const refusalOfCheckRepeatedIndex = caught(function checkRepeatedIndex() {
          assertSliceIndexing({
            slices: [
              sliceAt({
                sourceIndex: 0,
                targetIndex: 0,
              },),
              sliceAt({
                sourceIndex: 0,
                targetIndex: 0,
              },),
            ],
          },);
        },);

        expect(refusalOfCheckRepeatedIndex,).toBeInstanceOf(SliceIndexingError,);
        expect((refusalOfCheckRepeatedIndex as Error).message,).toContain('reads that index as the position',);
      },
    },),
    it({
      name: 'REFUSES a gap, which is what makes a range check pass while naming a slice that does not '
        + 'exist: assembly maps replacements back through these indices, and a hole in them is a '
        + 'replacement with nowhere to land',
      fn: async () => {
        expect(function checkGap() {
          assertSliceIndexing({
            slices: [
              sliceAt({
                sourceIndex: 0,
                targetIndex: 0,
              },),
              sliceAt({
                sourceIndex: 2,
                targetIndex: 2,
              },),
            ],
          },);
        },).toThrow(SliceIndexingError,);
      },
    },),
    it({
      name: 'accepts an empty preparation, since a document pair the aligner refused entirely produces no '
        + 'slices and that is a settled outcome rather than a defect',
      fn: async () => {
        expect(function checkEmpty() {
          assertSliceIndexing({ slices: [], },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'accepts what `reindexSlicePair` produces from sides that disagreed, which is how the '
        + 'preparation stops depending on the base index it handed to subdivision: the final name is '
        + 'stamped once, onto both sides, from where the slice actually landed',
      fn: async () => {
        /** Slice whose two sides were stamped by different producers. */
        const disagreeing = sliceAt({
          sourceIndex: 4,
          targetIndex: 1,
        },);

        /** Same slice, renamed by its position. */
        const stamped = reindexSlicePair({
          slice: disagreeing,
          slicePosition: 0,
        },);
        expect(stamped.source
          .sliceIndex,).toBe(0,);
        expect(stamped.target
          .sliceIndex,).toBe(0,);
        expect(stamped.target
          .text,).toBe(disagreeing.target
            .text,);
        expect(function checkStamped() {
          assertSliceIndexing({ slices: [stamped,], },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'WORDS its refusal from the fault alone, as a marked class the boundary may print',
      fn: async () => {
        const error = new SliceIndexingError({
          fault: {
            kind: 'sides-disagree',
            position: 1,
            sourceIndex: 1,
            targetIndex: 2,
          },
        },);
        expect(error.message,).toBe(
          'slice at position 1 carries source index 1 against target index 2, so which one names it '
            + 'depends on who is asking',
        );
        expect(error.messageNamesOnly,).toBe(true,);
      },
    },),
  ],
},);
