/**
 Tests for the per-slice wording both lanes report.

 This is the join key of every two-lane comparison, so its failure mode is a
 comparison that reads clean while lining up two different passages. The
 coverage checks are the whole point: a lane that skipped a slice, or reported
 one its preparation never produced, must say so rather than produce a shorter
 list nobody counts.

 Fixtures are invented. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildLaneSliceTexts,
  LaneSliceCoverageError,
} from '../dist/final/node/index.mjs';

/**
 Builds one prepared slice pair.

 @param index - global slice index both sides carry

 @param source - original text of this slice

 @param target - archive translation of it

 @returns Pair shaped as preparation produces

 @example
 ```ts
 const pair = pairOf({ index: 0, source: '猫', target: 'The cat.', },);
 ```
 */
function pairOf(
  {
    index,
    source,
    target,
  }: {
    readonly index: number;
    readonly source: string;
    readonly target: string;
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
      sliceIndex: index,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: source,
    },
    target: {
      sliceIndex: index,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: target,
    },
  };
}

/**
 Two prepared slices about a cat.
 */
const CAT_SLICES = [
  pairOf({
    index: 0,
    source: 'source of the nap',
    target: 'The cat sleeps on the sill.',
  },),
  pairOf({
    index: 1,
    source: 'source of the bowl',
    target: 'The bowl is full.',
  },),
];

/**
 The same pair with the second slice a place rather than existing text, which
 is the only kind of slice a lane may report as unfilled.
 */
const ANCHORED_SLICES = [
  CAT_SLICES[0] ?? pairOf({
    index: 0,
    source: 'source of the nap',
    target: 'The cat sleeps on the sill.',
  },),
  {
    source: {
      sliceIndex: 1,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: 'source of the bowl',
    },
    target: makeInsertionChunk({
      sliceIndex: 1,
      offset: 0,
    },),
  },
];

await describe({
  name: buildLaneSliceTexts.name,
  children: [
    it({
      name:
        'pairs every prepared slice with the archive wording it was judged against, '
          + 'including a slice the lane left alone, because a rate needs its denominator '
          + 'and leaving a slice alone is a decision rather than an absence',
      fn: async () => {
        /**
         Wordings for a lane that changed the first slice only.
         */
        const wordings = buildLaneSliceTexts({
          slices: CAT_SLICES,
          undecided: 'refuse',
          decided: [
            { sliceIndex: 0, text: 'The cat is asleep on the windowsill.', },
            { sliceIndex: 1, text: 'The bowl is full.', },
          ],
        },);
        expect(wordings,).toHaveLength(2,);
        expect(wordings[0]?.incumbentText,).toBe('The cat sleeps on the sill.',);
        expect(wordings[0]?.outcome,).toEqual({
          kind: 'decided',
          acceptedText: 'The cat is asleep on the windowsill.',
        },);

        // The untouched slice is still reported, and its accepted wording is the
        // archive's own rather than an empty string standing for "nothing".
        expect(wordings[1]?.outcome,).toEqual({
          kind: 'decided',
          acceptedText: 'The bowl is full.',
        },);
        expect(wordings[1]?.incumbentText,).toBe('The bowl is full.',);
      },
    },),
    it({
      name:
        'keeps document order and the global index, which is what a comparison joins two lanes on, '
          + 'rather than the position within this list',
      fn: async () => {
        /**
         Wordings built from decisions given out of order.
         */
        const wordings = buildLaneSliceTexts({
          slices: CAT_SLICES,
          undecided: 'refuse',
          decided: [
            { sliceIndex: 1, text: 'The bowl is full.', },
            { sliceIndex: 0, text: 'The cat sleeps on the sill.', },
          ],
        },);
        expect(wordings.map(function toIndex(one,): number {
          return one.sliceIndex;
        },),).toEqual([0, 1,],);
      },
    },),
    it({
      name:
        'REFUSES a lane that left a prepared slice undecided, because the alternative is a shorter list '
          + 'that every later count silently reads as a smaller document',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'refuse',
            decided: [{ sliceIndex: 0, text: 'The cat is asleep.', },],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('1',);
      },
    },),
    it({
      name:
        'REFUSES a decision naming a slice this preparation never produced, which is how two different '
          + 'preparations of one entry would otherwise be joined without anything noticing',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'refuse',
            decided: [
              { sliceIndex: 0, text: 'The cat is asleep.', },
              { sliceIndex: 1, text: 'The bowl is full.', },
              { sliceIndex: 7, text: 'A slice from another slicing entirely.', },
            ],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('7',);
      },
    },),
    it({
      name:
        'records a slice the lane never reached as UNEXAMINED rather than as agreeing with the archive, '
          + 'which is what the repair lane`s whole-document block needs: it returns from inside the slice loop, '
          + 'so the slices after the crossing were never looked at and reporting the archive wording as their '
          + 'decision would state a choice nobody took',
      fn: async () => {
        /**
         Wordings for a lane that stopped after the first slice.
         */
        const wordings = buildLaneSliceTexts({
          slices: CAT_SLICES,
          undecided: 'not-evaluated',
          decided: [{ sliceIndex: 0, text: 'The cat is asleep on the windowsill.', },],
        },);
        expect(wordings,).toHaveLength(2,);
        expect(wordings[0]?.outcome,).toEqual({
          kind: 'decided',
          acceptedText: 'The cat is asleep on the windowsill.',
        },);
        // NAMED rather than absent. An optional wording had to carry this
        // meaning by not being there, which every consumer then had to guess
        // at, and two different gaps guessed the same way.
        expect(wordings[1]?.outcome,).toEqual({ kind: 'not-evaluated', },);

        // The archive wording is still reported for the unexamined slice, since
        // that is what the returned document carries there.
        expect(wordings[1]?.incumbentText,).toBe('The bowl is full.',);
      },
    },),
    it({
      name:
        'REFUSES a decision that comes AFTER an unexamined slice, since `not-evaluated` describes '
          + 'one shape and no other: a lane that stopped, so an evaluated prefix and an unevaluated '
          + 'suffix. Decisions for slices 0 and 2 with 1 unexamined is a slice that was DROPPED, and '
          + 'accepting it would let that pass as an early stop',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'not-evaluated',
            decided: [{ sliceIndex: 1, text: 'The bowl is empty.', },],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('after leaving an earlier one unexamined',);
      },
    },),
    it({
      name:
        'still REFUSES a decision naming an unknown slice under the not-evaluated policy, because that policy '
          + 'forgives a lane that stopped early and not one that describes another slicing',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'not-evaluated',
            decided: [{ sliceIndex: 9, text: 'A slice from another slicing entirely.', },],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('9',);
      },
    },),
    it({
      name:
        'accepts a NAMED unfilled slice under `refuse` and still refuses every other gap, which is the '
          + 'difference between a lane that examined a passage and could not fill it and a lane that lost '
          + 'one: the first can say which slices, the second cannot',
      fn: async () => {
        /**
         Wordings where the lane decided the first slice and could not fill
         the second.
         */
        const wordings = buildLaneSliceTexts({
          slices: ANCHORED_SLICES,
          undecided: 'refuse',
          decided: [{ sliceIndex: 0, text: 'The cat naps on the sill.', },],
          unfilledChunkIndices: [1,],
        },);
        expect(wordings,).toHaveLength(2,);
        expect(wordings[1]?.outcome,).toEqual({ kind: 'unfilled', },);
        // And the anchor says the archive holds nothing here, which is the
        // fact separating a gap that remains from wording that stands.
        expect(wordings[1]?.incumbentKind,).toBe('absent',);
        expect(wordings[1]?.incumbentText,).toBe('',);

        /**
         Failure the same gap raises when nothing names it.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: ANCHORED_SLICES,
            undecided: 'refuse',
            decided: [{ sliceIndex: 0, text: 'The cat naps on the sill.', },],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
      },
    },),
    it({
      name:
        'REFUSES a slice reported as unfilled AND decided, since what the lane accepted there would be '
          + 'unstated, and refuses one this preparation never produced, the same way it refuses a '
          + 'decision naming a foreign slice',
      fn: async () => {
        /**
         Failure raised by a slice claimed twice.
         */
        let both: unknown;
        try {
          buildLaneSliceTexts({
            slices: ANCHORED_SLICES,
            undecided: 'refuse',
            decided: [
              { sliceIndex: 0, text: 'The cat naps on the sill.', },
              { sliceIndex: 1, text: '', },
            ],
            unfilledChunkIndices: [1,],
          },);
        }
        catch (error) {
          both = error;
        }
        expect(both,).toBeInstanceOf(LaneSliceCoverageError,);

        /**
         Failure raised by an unfilled index from another slicing.
         */
        let foreign: unknown;
        try {
          buildLaneSliceTexts({
            slices: ANCHORED_SLICES,
            undecided: 'refuse',
            decided: [
              { sliceIndex: 0, text: 'The cat naps on the sill.', },
              { sliceIndex: 1, text: '', },
            ],
            unfilledChunkIndices: [7,],
          },);
        }
        catch (error) {
          foreign = error;
        }
        expect(foreign,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(foreign,),).toContain('7',);
      },
    },),
    it({
      name:
        'names a slice the lane reached with NO VOICE HEARD as the archive standing by default, which is a '
          + 'different fact from the archive being chosen: nobody answered, so nobody chose anything, and the '
          + 'outcome carries no wording for a reader to mistake for one',
      fn: async () => {
        /**
         Wordings where the second slice met silence.
         */
        const wordings = buildLaneSliceTexts({
          slices: CAT_SLICES,
          undecided: 'refuse',
          decided: [{ sliceIndex: 0, text: 'The cat naps on the sill.', },],
          unheardChunkIndices: [1,],
        },);
        expect(wordings,).toHaveLength(2,);
        expect(wordings[1]?.outcome,).toEqual({ kind: 'incumbent-fallback', },);
        // And the archive DOES hold wording here, which is what separates this
        // from an unfilled passage and what makes falling back possible at all.
        expect(wordings[1]?.incumbentKind,).toBe('present',);
        expect(wordings[1]?.incumbentText,).toBe('The bowl is full.',);
      },
    },),
    it({
      name:
        'REFUSES a slice reported as unheard AND decided, since whether anyone answered for it would be '
          + 'unstated, and refuses an unheard index this preparation never produced',
      fn: async () => {
        /**
         Failure raised by a slice claimed both ways.
         */
        let both: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'refuse',
            decided: [
              { sliceIndex: 0, text: 'The cat naps on the sill.', },
              { sliceIndex: 1, text: 'The bowl is full.', },
            ],
            unheardChunkIndices: [1,],
          },);
        }
        catch (error) {
          both = error;
        }
        expect(both,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(both,),).toContain('unheard and decided at once',);

        /**
         Failure raised by an unheard index from another slicing.
         */
        let foreign: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'refuse',
            decided: [
              { sliceIndex: 0, text: 'The cat naps on the sill.', },
              { sliceIndex: 1, text: 'The bowl is full.', },
            ],
            unheardChunkIndices: [7,],
          },);
        }
        catch (error) {
          foreign = error;
        }
        expect(foreign,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(foreign,),).toContain('7',);
      },
    },),
    it({
      name:
        'REFUSES a slice reported as unheard AND unfilled, because only one of those has an incumbent to '
          + 'stand on and the two disagree about whether the archive translates the passage at all',
      fn: async () => {
        /**
         Failure raised by the contradiction.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: ANCHORED_SLICES,
            undecided: 'refuse',
            decided: [{ sliceIndex: 0, text: 'The cat naps on the sill.', },],
            unfilledChunkIndices: [1,],
            unheardChunkIndices: [1,],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('unfilled and unheard at once',);
        expect(String(caught,),).toContain('stated twice and differently',);
      },
    },),
    it({
      name:
        'REFUSES an unheard slice the archive never translated, the MIRROR of the unfilled rule: '
          + '`incumbent-fallback` says the archive`s wording stands here, so reporting it at a place with no '
          + 'archive wording would record a passage as covered by a translation that does not exist',
      fn: async () => {
        /**
         Failure raised by a fallback with nothing to fall back on.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: ANCHORED_SLICES,
            undecided: 'refuse',
            decided: [{ sliceIndex: 0, text: 'The cat naps on the sill.', },],
            unheardChunkIndices: [1,],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('no wording for it to fall back on',);
      },
    },),
    it({
      name:
        'REFUSES an UNFILLED slice that comes after an unexamined one, not only a decision, because a lane '
          + 'that stopped cannot report reaching a later slice by any route: an unfilled slice sitting after '
          + 'an unexamined one asserts the lane resumed after stopping',
      fn: async () => {
        /**
         Failure raised by reaching a slice after the stop.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: ANCHORED_SLICES,
            undecided: 'not-evaluated',
            decided: [],
            unfilledChunkIndices: [1,],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('after leaving an earlier one unexamined',);
      },
    },),
    it({
      name:
        'REFUSES an UNHEARD slice after an unexamined one for the same reason, since hearing nobody at a '
          + 'slice still means the lane got there',
      fn: async () => {
        /**
         Failure raised by reaching a slice after the stop.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'not-evaluated',
            decided: [],
            unheardChunkIndices: [1,],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('after leaving an earlier one unexamined',);
      },
    },),
    it({
      name: 'WORDS its refusal from the set label and fault alone, as a marked class the boundary may print',
      fn: async () => {
        const error = new LaneSliceCoverageError({
          fault: {
            kind: 'set-and-decided',
            set: 'unfilled',
            sliceIndex: 2,
          },
        },);
        expect(error.message,).toBe(
          'lane reports slice 2 as unfilled and decided at once, so what it accepted there is unstated',
        );
        expect(error.messageNamesOnly,).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES A PREPARATION REPEATING AN INDEX, which would let one decision answer for two slices, in '
        + 'words that agree with each count (ledger B66)',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: [
              pairOf({
                index: 0,
                source: 'source of the nap',
                target: 'The cat sleeps on the sill.',
              },),
              pairOf({
                index: 0,
                source: 'source of the bowl',
                target: 'The bowl is full.',
              },),
            ],
            undecided: 'refuse',
            decided: [],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('preparation produced 2 slices under 1 distinct index',);
      },
    },),
    it({
      name: 'REFUSES A LANE DECIDING ONE SLICE TWICE, where the later wording would silently win, in words '
        + 'that agree with each count (ledger B66)',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          buildLaneSliceTexts({
            slices: CAT_SLICES,
            undecided: 'refuse',
            decided: [
              { sliceIndex: 0, text: 'The cat is asleep.', },
              { sliceIndex: 0, text: 'The cat naps.', },
            ],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('lane decided 2 times over 1 distinct slice',);
        expect(String(caught,),).not.toContain('1 distinct slices',);
      },
    },),
  ],
},);
