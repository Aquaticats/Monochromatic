/**
 Tests for the contest record, at the level it is derived rather than through
 a whole artifact.

 WHAT IS UNDER TEST is the split the stage does not make for itself: `neither`
 means two unrelated things depending on how many voices were heard, and this
 module is where they stop being one answer.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  contestEligibleIndexes,
  describeContestSlice,
  describeSoleLaneSlice,
  type LaneContestBallot,
  type LaneContestOutcome,
  soleLaneOf,
} from '../../dist/final/node/index.mjs';
import { ARCHIVE_NAP, catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';
import { insertionRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/sole-lane-comparison-row.test-fixture.ts';

/**
 Wording the repair lane left.
 */
const REPAIR_NAP = 'The cat naps in the bookshop attic.';

/**
 Wording the translate lane left, differing from both.
 */
const TRANSLATE_NAP = 'The cat dozes in the attic of the bookshop.';

/**
 Ballot backing the repair lane, with nothing found against either candidate.
 */
const FOR_REPAIR: LaneContestBallot = {
  choice: 'repair',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'keeps the bookshop, which the original names',
};

/**
 Ballot backing the translate lane.
 */
const FOR_TRANSLATE: LaneContestBallot = {
  choice: 'translate',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'reads more naturally and says the same thing',
};

/**
 Ballot backing neither, which is a reading rather than a silence.
 */
const FOR_NEITHER: LaneContestBallot = {
  choice: 'neither',
  unsupported: [
    'repair',
    'translate',
  ],
  unsupportedRaw: ['both add an attic the original does not mention',],
  dropped: [],
  droppedRaw: [],
  reason: 'both candidates invent a detail',
};

/**
 Builds an outcome the way the stage returns one.

 @param ballots - usable ballots

 @param choice - candidate the stage settled on

 @returns Outcome carrying those ballots

 @example
 ```ts
 const outcome = catOutcome({ ballots: [FOR_REPAIR,], choice: 'neither', },);
 ```
 */
function catOutcome(
  {
    ballots,
    choice,
  }: {
    readonly ballots: readonly LaneContestBallot[];
    readonly choice: LaneContestOutcome['choice'];
  },
): LaneContestOutcome {
  return {
    choice,
    ballots,
    usable: ballots.length,
    findings: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: describeContestSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'RECORDS the lane enough voices backed, naming it rather than leaving a reader to count the '
              + 'ballots back up',
          fn: async () => {
            expect(describeContestSlice({
              sliceIndex: 3,
              outcome: catOutcome({
                ballots: [
                  FOR_REPAIR,
                  FOR_REPAIR,
                  FOR_TRANSLATE,
                ],
                choice: 'repair',
              },),
            },),).toEqual({
              sliceIndex: 3,
              verdict: {
                kind: 'lane-won',
                lane: 'repair',
              },
              ballots: [
                FOR_REPAIR,
                FOR_REPAIR,
                FOR_TRANSLATE,
              ],
              usable: 3,
            },);
          },
        },),
        it({
          name:
            'SEPARATES a roster that heard enough voices and backed neither lane from one that went '
              + 'unheard, since the stage answers `neither` for both and a reader counting refusals would '
              + 'otherwise be counting silence',
          fn: async () => {
            /**
             Enough voices, none of them carrying a lane.
             */
            const settled = describeContestSlice({
              sliceIndex: 0,
              outcome: catOutcome({
                ballots: [
                  FOR_NEITHER,
                  FOR_NEITHER,
                ],
                choice: 'neither',
              },),
            },);

            /**
             One voice, which cannot settle anything under the same rule.
             */
            const unheard = describeContestSlice({
              sliceIndex: 1,
              outcome: catOutcome({
                ballots: [FOR_REPAIR,],
                choice: 'neither',
              },),
            },);
            expect(settled.verdict,).toEqual({ kind: 'settled-neither', },);
            expect(unheard.verdict,).toEqual({ kind: 'quorum-not-met', },);
          },
        },),
        it({
          name:
            'KEEPS THE BALLOTS on a verdict that ships nothing, which is exactly where a reader asking '
              + 'why is looking',
          fn: async () => {
            expect(describeContestSlice({
              sliceIndex: 0,
              outcome: catOutcome({
                ballots: [
                  FOR_NEITHER,
                  FOR_NEITHER,
                ],
                choice: 'neither',
              },),
            },).ballots,).toEqual([
              FOR_NEITHER,
              FOR_NEITHER,
            ],);
          },
        },),
        it({
          name:
            'RECORDS A SLICE THE FLOOR KEPT FROM THE ROSTER with the mark, and a roster that heard nobody without '
              + 'it, so a reader of the record alone tells a round never opened from one that failed',
          fn: async () => {
            /**
             Records of the same empty outcome, one marked by the stage and one not.
             */
            const records = {
              skipped: describeContestSlice({
                sliceIndex: 0,
                outcome: {
                  ...catOutcome({
                    ballots: [],
                    choice: 'neither',
                  },),
                  benchBelowFloor: true,
                },
              },),
              unheard: describeContestSlice({
                sliceIndex: 0,
                outcome: catOutcome({
                  ballots: [],
                  choice: 'neither',
                },),
              },),
            };
            expect({
              skippedVerdict: records.skipped.verdict,
              skipped: records.skipped.benchBelowFloor,
              unheardHasKey: 'benchBelowFloor' in records.unheard,
            },).toEqual({
              skippedVerdict: { kind: 'quorum-not-met', },
              skipped: true,
              unheardHasKey: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: contestEligibleIndexes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'NAMES ONLY the slices whose two lane wordings differ, since a contest between two identical '
              + 'candidates has nothing to ask and no answer worth buying',
          fn: async () => {
            expect(contestEligibleIndexes({
              comparison: [
                catRow({
                  sliceIndex: 0,
                  repairText: REPAIR_NAP,
                  translateText: TRANSLATE_NAP,
                },),
                catRow({
                  sliceIndex: 1,
                  repairText: ARCHIVE_NAP,
                  translateText: ARCHIVE_NAP,
                },),
                catRow({
                  sliceIndex: 2,
                  repairText: REPAIR_NAP,
                  translateText: ARCHIVE_NAP,
                },),
              ],
            },),).toEqual([
              0,
              2,
            ],);
          },
        },),
        it({
          name: 'NAMES NOTHING when the two documents agree everywhere, rather than naming every slice',
          fn: async () => {
            expect(contestEligibleIndexes({
              comparison: [
                catRow({
                  sliceIndex: 0,
                  repairText: ARCHIVE_NAP,
                  translateText: ARCHIVE_NAP,
                },),
              ],
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: soleLaneOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE TRANSLATE LANE when only the repair lane recorded not-applicable, which is an insertion',
          fn: async () => {
            expect(soleLaneOf({
              row: insertionRow({
                sliceIndex: 0,
                translateText: TRANSLATE_NAP,
              },),
            },),).toEqual({
              kind: 'one-lane',
              lane: 'translate',
            },);
          },
        },),
        it({
          name: 'NAMES THE REPAIR LANE when only the translate lane recorded not-applicable',
          fn: async () => {
            expect(soleLaneOf({
              row: {
                ...catRow({
                  sliceIndex: 0,
                  repairText: REPAIR_NAP,
                  translateText: TRANSLATE_NAP,
                },),
                translateOutcome: { kind: 'not-applicable', },
              },
            },),).toEqual({
              kind: 'one-lane',
              lane: 'repair',
            },);
          },
        },),
        it({
          name: 'ANSWERS BOTH LANES when both lanes had a passage to work on, which leaves the question to the roster',
          fn: async () => {
            expect(soleLaneOf({
              row: catRow({
                sliceIndex: 0,
                repairText: REPAIR_NAP,
                translateText: TRANSLATE_NAP,
              },),
            },),).toEqual({ kind: 'both-lanes', },);
          },
        },),
        it({
          name: 'ANSWERS BOTH LANES when neither lane had a passage, since no such row is ever contested',
          fn: async () => {
            expect(soleLaneOf({
              row: {
                ...catRow({
                  sliceIndex: 0,
                  repairText: REPAIR_NAP,
                  translateText: TRANSLATE_NAP,
                },),
                repairOutcome: { kind: 'not-applicable', },
                translateOutcome: { kind: 'not-applicable', },
              },
            },),).toEqual({ kind: 'both-lanes', },);
          },
        },),
      ],
    },),

    describe({
      name: describeSoleLaneSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS THE SOLE LANE with no ballots and no voices behind it, so no roster reads as having been asked',
          fn: async () => {
            expect(describeSoleLaneSlice({
              sliceIndex: 4,
              lane: 'translate',
            },),).toEqual({
              sliceIndex: 4,
              verdict: {
                kind: 'sole-lane',
                lane: 'translate',
              },
              ballots: [],
              usable: 0,
            },);
          },
        },),
      ],
    },),
  ],
},);
