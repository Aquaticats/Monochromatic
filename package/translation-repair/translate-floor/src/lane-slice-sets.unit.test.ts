/**
 Tests for the five checks every list of named slices has to pass.

 WHY THESE ARE WORTH ASKING DIRECTLY. Three lists exist and they arrived one
 at a time, each written as its own loop against the preparation, until by the
 third they were the same five checks with the wording changed. That is how
 the disjointness between them came to be checked in one direction only. The
 checks now live in one place and the lists differ by data, so a case here
 asks a rule once and every list inherits the answer.

 THE ORDER IS PART OF THE CONTRACT, not an implementation detail, and two
 cases pin it. A slice named by two lists disagrees with itself before it
 disagrees with the archive, so it must report the contradiction between the
 lists rather than whichever archive rule the earlier list happens to break.
 A slice that is also decided reports that first of all.

 WHAT THE ARCHIVE RULE IS FOR. Every list is legal at one kind of slice only,
 and getting that wrong is how an exemption list turns into a way around the
 coverage rule: name a translated slice `unfilled` and the lane is excused
 from saying anything about text the archive already holds.

 `lane-slice-text.ts` is the only caller, and it hands in whatever three lists
 a finished lane produced. A valid lane reaches none of these refusals.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  LaneSliceCoverageError,
  type NamedSliceSet,
  validateNamedSets,
} from '../dist/final/node/index.mjs';
import { listed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/sorted-number-sets.test-fixture.ts';

/**
 Archive wording standing at a slice the corpus translated.
 */
const ARCHIVE_SILL = 'The cat sleeps on the sill.';

/**
 Builds one pair whose translation side holds wording.

 @param sliceIndex - position of this pair in the preparation

 @returns Pair the archive translates

 @example
 ```ts
 const pair = translatedPair({ sliceIndex: 0, },);
 ```
 */
function translatedPair(
  { sliceIndex, }: { readonly sliceIndex: number; },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      startOffset: 0,
      endOffset: 9,
      nodes: [],
      text: '猫猫在窗台上睡觉。',
    },
    target: {
      sliceIndex,
      startOffset: 0,
      endOffset: ARCHIVE_SILL.length,
      nodes: [],
      text: ARCHIVE_SILL,
    },
  };
}

/**
 Builds one pair whose translation side is a place a rendering belongs and
 none exists.

 @param sliceIndex - position of this pair in the preparation

 @returns Pair the archive never translated

 @example
 ```ts
 const pair = untranslatedPair({ sliceIndex: 1, },);
 ```
 */
function untranslatedPair(
  { sliceIndex, }: { readonly sliceIndex: number; },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      startOffset: 0,
      endOffset: 8,
      nodes: [],
      text: '窗台上有一只鸟。',
    },
    target: makeInsertionChunk({
      sliceIndex,
      offset: 0,
    },),
  };
}

/**
 Preparation these cases name slices out of: slices 0 and 2 translated,
 slices 1 and 3 not.

 BOTH KINDS TWICE, so a case can move a named slice from one side of the
 archive to the other without also changing its index.

 @returns Four pairs in document order

 @example
 ```ts
 const slices = preparation();
 ```
 */
function preparation(): readonly ChunkPair[] {
  return [
    translatedPair({ sliceIndex: 0, },),
    untranslatedPair({ sliceIndex: 1, },),
    translatedPair({ sliceIndex: 2, },),
    untranslatedPair({ sliceIndex: 3, },),
  ];
}

/**
 List of slices a lane reached and could not fill, which is legal only where
 the archive holds nothing.

 @param indices - slices this list names

 @returns List as the lane builder spells it

 @example
 ```ts
 const set = unfilledSet({ indices: [1,], },);
 ```
 */
function unfilledSet(
  { indices, }: { readonly indices: readonly number[]; },
): NamedSliceSet {
  return {
    label: 'unfilled',
    indices,
    incumbent: 'absent',
  };
}

/**
 List of slices no judge answered for, which is legal only where the archive
 holds wording to fall back on.

 @param indices - slices this list names

 @returns List as the lane builder spells it

 @example
 ```ts
 const set = unheardSet({ indices: [0,], },);
 ```
 */
function unheardSet(
  { indices, }: { readonly indices: readonly number[]; },
): NamedSliceSet {
  return {
    label: 'unheard',
    indices,
    incumbent: 'present',
  };
}

/**
 List of slices this lane had nothing to do at, which is legal only where the
 archive holds nothing.

 @param indices - slices this list names

 @returns List as the lane builder spells it

 @example
 ```ts
 const set = notApplicableSet({ indices: [3,], },);
 ```
 */
function notApplicableSet(
  { indices, }: { readonly indices: readonly number[]; },
): NamedSliceSet {
  return {
    label: 'not-applicable',
    indices,
    incumbent: 'absent',
  };
}

await describe({
  name: validateNamedSets.name,
  children: [
    it({
      name: 'ACCEPTS three lists that name legal, distinct and disjoint '
        + 'slices, and hands each back at the position its list was given in',
      fn: async () => {
        expect(listed({
          sets: validateNamedSets({
            sets: [
              unfilledSet({ indices: [1,], },),
              unheardSet({ indices: [0,], },),
              notApplicableSet({ indices: [3,], },),
            ],
            slices: preparation(),
            decidedIndices: new Set([2,],),
          },),
        },),)
          .toEqual([
            [1,],
            [0,],
            [3,],
          ],);
      },
    },),

    it({
      name: 'ACCEPTS a list that names nothing, and still returns a set for '
        + 'it, so a caller destructuring by position reads its own list rather '
        + 'than the next one',
      fn: async () => {
        expect(listed({
          sets: validateNamedSets({
            sets: [
              unfilledSet({ indices: [], },),
              unheardSet({ indices: [0,], },),
            ],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },),
        },),)
          .toEqual([
            [],
            [0,],
          ],);
      },
    },),

    it({
      name: 'ACCEPTS no lists at all, since a lane that named nothing is an '
        + 'ordinary lane and not a lane with something missing',
      fn: async () => {
        expect(listed({
          sets: validateNamedSets({
            sets: [],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },),
        },),)
          .toEqual([],);
      },
    },),

    it({
      name: 'REFUSES a list naming one slice twice, naming both counts in words that agree with each (ledger '
        + 'B66): the set would still be the right shape afterwards and one slice would be named once',
      fn: async () => {
        const refusalOfRepeat = caught(function repeat() {
          validateNamedSets({
            sets: [
              unfilledSet({
                indices: [
                  1,
                  1,
                ],
              },),
            ],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },);
        },);

        expect(refusalOfRepeat,).toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfRepeat as Error).message,)
          .toContain('lane reports 2 unfilled slices under 1 distinct index',);
        expect((refusalOfRepeat as Error).message,).not.toContain('1 distinct indices',);
      },
    },),

    it({
      name: 'REFUSES a slice this preparation never produced, which is what a '
        + 'lane built against a different slicing looks like from here',
      fn: async () => {
        const refusalOfStranger = caught(function stranger() {
          validateNamedSets({
            sets: [unfilledSet({ indices: [9,], },),],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },);
        },);

        expect(refusalOfStranger,).toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfStranger as Error).message,)
          .toContain('lane reports slice 9 unfilled, which this preparation never produced',);
      },
    },),

    it({
      name: 'REFUSES a slice the lane also reported a wording for, since a '
        + 'slice cannot both carry a decision and be excused from carrying one',
      fn: async () => {
        const refusalOfDecided = caught(function decided() {
          validateNamedSets({
            sets: [unfilledSet({ indices: [1,], },),],
            slices: preparation(),
            decidedIndices: new Set([1,],),
          },);
        },);

        expect(refusalOfDecided,).toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfDecided as Error).message,)
          .toContain('lane reports slice 1 as unfilled and decided at once, '
            + 'so what it accepted there is unstated',);
      },
    },),

    it({
      name: 'REFUSES a slice two lists both name, and names both lists, which '
        + 'is the check that was one-directional while each list carried its '
        + 'own loop',
      fn: async () => {
        const refusalOfBothLists = caught(function bothLists() {
          validateNamedSets({
            sets: [
              unfilledSet({ indices: [1,], },),
              notApplicableSet({ indices: [1,], },),
            ],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },);
        },);

        expect(refusalOfBothLists,).toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfBothLists as Error).message,)
          .toContain('lane reports slice 1 as unfilled and not-applicable at once, '
            + 'so what it did there is stated twice and differently',);
      },
    },),

    it({
      name: 'REFUSES a slice two lists name EVEN WHERE the archive rule would '
        + 'also refuse it, reporting the contradiction between the lists: a '
        + 'slice that disagrees with itself has not earned the right to be '
        + 'asked which archive rule it breaks',
      fn: async () => {
        const refusalOfBothAndWrongSide = caught(function bothAndWrongSide() {
          validateNamedSets({
            sets: [
              unfilledSet({ indices: [0,], },),
              notApplicableSet({ indices: [0,], },),
            ],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },);
        },);

        expect(refusalOfBothAndWrongSide,)
          .toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfBothAndWrongSide as Error).message,)
          .toContain('as unfilled and not-applicable at once',);
      },
    },),

    it({
      name: 'REPORTS THE DECISION FIRST for a slice that is decided AND named '
        + 'by two lists, since the wording it carries settles the question '
        + 'both lists are arguing about',
      fn: async () => {
        const refusalOfDecidedAndBoth = caught(function decidedAndBoth() {
          validateNamedSets({
            sets: [
              unfilledSet({ indices: [1,], },),
              notApplicableSet({ indices: [1,], },),
            ],
            slices: preparation(),
            decidedIndices: new Set([1,],),
          },);
        },);

        expect(refusalOfDecidedAndBoth,)
          .toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfDecidedAndBoth as Error).message,)
          .toContain('as unfilled and decided at once',);
      },
    },),

    it({
      name: 'REFUSES a list about untranslated passages naming one the '
        + 'archive translates, which is how an exemption list turns into a way '
        + 'around the coverage rule',
      fn: async () => {
        const refusalOfTranslatedSlice = caught(function translatedSlice() {
          validateNamedSets({
            sets: [unfilledSet({ indices: [0,], },),],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },);
        },);

        expect(refusalOfTranslatedSlice,)
          .toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfTranslatedSlice as Error).message,)
          .toContain('lane reports slice 0 unfilled, and the archive holds wording for it',);
      },
    },),

    it({
      name: 'REFUSES a list about wording the archive holds naming a slice it '
        + 'never translated, which is the same rule read from the other side',
      fn: async () => {
        const refusalOfUntranslatedSlice = caught(function untranslatedSlice() {
          validateNamedSets({
            sets: [unheardSet({ indices: [1,], },),],
            slices: preparation(),
            decidedIndices: new Set<number>(),
          },);
        },);

        expect(refusalOfUntranslatedSlice,)
          .toBeInstanceOf(LaneSliceCoverageError,);
        expect((refusalOfUntranslatedSlice as Error).message,)
          .toContain('lane reports slice 1 unheard, '
            + 'and the archive holds no wording for it to fall back on',);
      },
    },),
  ],
},);
