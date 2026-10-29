/**
 Tests for the tense candidates of a lane contest: the list the judges read and
 the finding the artifact keeps are one list under one set of labels, and the
 finding feeds no verdict.

 Fixtures are cat-themed invention; no corpus text appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactContestSlice,
  describeContestSlice,
  describeSoleLaneSlice,
  laneContestTenseGroups,
  type LaneContestBallot,
  withTenseCandidates,
} from '../dist/final/node/index.mjs';
import { insertionRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/sole-lane-comparison-row.test-fixture.ts';
import { catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';

/**
 Ballot backing the repair lane.
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
 Contest record of one slice, won by the repair lane.

 @param sliceIndex - slice the record answers

 @returns Record carrying three ballots for the repair lane

 @example
 ```ts
 const slice = repairWon({ sliceIndex: 0, },);
 ```
 */
function repairWon({ sliceIndex, }: { readonly sliceIndex: number; },): ArtifactContestSlice {
  return describeContestSlice({
    sliceIndex,
    outcome: {
      choice: 'repair',
      ballots: [
        FOR_REPAIR,
        FOR_REPAIR,
        FOR_REPAIR,
      ],
      usable: 3,
      findings: [],
    },
  },);
}

/**
 One candidate the detector finds in the repair wording of the first row.
 */
const REPAIR_FOUND = {
  sentence: 'Mittens naps in the bookshop attic.',
  matched: 'Mittens naps',
  rule: 's-ending-verb',
};

//region Lane contest tense tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: laneContestTenseGroups.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GROUPS the archive wording and the two lane wordings under the labels the contest sheet gives them, only the ones with a candidate, where the entry records a death',
          fn: async () => {
            expect(laneContestTenseGroups({
              lifeEnded: true,
              subjectNames: ['Mittens',],
              incumbentText: 'Mittens is loud.',
              repairText: 'The cat slept.',
              translateText: 'She naps by the window.',
            },),).toEqual([
              {
                label: 'ARCHIVE RENDERING',
                found: [
                  {
                    sentence: 'Mittens is loud.',
                    matched: 'Mittens is',
                    rule: 'copula-or-auxiliary',
                  },
                ],
              },
              {
                label: 'CANDIDATE "translate"',
                found: [
                  {
                    sentence: 'She naps by the window.',
                    matched: 'She naps',
                    rule: 's-ending-verb',
                  },
                ],
              },
            ],);
          },
        },),

        it({
          name: 'GROUPS nothing where the entry records no death, whatever the texts say',
          fn: async () => {
            expect(laneContestTenseGroups({
              lifeEnded: false,
              subjectNames: ['Mittens',],
              incumbentText: 'Mittens is loud.',
              repairText: 'She naps by the window.',
              translateText: 'She is here.',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: withTenseCandidates.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS the very records it was given where the entry records no death',
          fn: async () => {
            /**
             Records the driver returned.
             */
            const slices = [repairWon({ sliceIndex: 0, },),];
            expect(withTenseCandidates({
              slices,
              comparison: [
                catRow({
                  sliceIndex: 0,
                  repairText: 'Mittens naps in the bookshop attic.',
                  translateText: 'The cat dozed in the attic.',
                },),
              ],
              lifeEnded: false,
            },) === slices,).toBe(true,);
          },
        },),

        it({
          name: 'ADDS the finding to the slices whose texts have a candidate and leaves the others as they stand, changing no verdict, ballot or count',
          fn: async () => {
            /**
             Records the driver returned for two slices.
             */
            const slices = [
              repairWon({ sliceIndex: 0, },),
              repairWon({ sliceIndex: 1, },),
            ];
            /**
             The same records with the finding added.
             */
            const recorded = withTenseCandidates({
              slices,
              comparison: [
                catRow({
                  sliceIndex: 0,
                  repairText: 'Mittens naps in the bookshop attic.',
                  translateText: 'The cat dozed in the attic.',
                },),
                catRow({
                  sliceIndex: 1,
                  repairText: 'The cat dozed in the bookshop attic.',
                  translateText: 'The cat napped in the attic.',
                },),
              ],
              lifeEnded: true,
              subjectNames: ['Mittens',],
            },);
            expect(recorded.map(function findingOf(slice,) {
              return slice.tenseCandidates;
            },),).toEqual([
              [
                {
                  label: 'CANDIDATE "repair"',
                  found: [REPAIR_FOUND,],
                },
              ],
              undefined,
            ],);
            expect(recorded.map(function withoutFinding(slice,) {
              const { tenseCandidates: _finding, ...rest } = slice;
              return rest;
            },),).toEqual(slices,);
            expect(recorded[1],).toBe(slices[1],);
          },
        },),

        it({
          name: 'LEAVES a sole-lane record as it stands, since no judge was shown the sheet whose candidates it would record',
          fn: async () => {
            /**
             The record the driver makes for an archive-absent insertion.
             */
            const sole = describeSoleLaneSlice({
              sliceIndex: 0,
              lane: 'translate',
            },);
            /**
             Records returned beside a sole-lane one.
             */
            const slices = [
              sole,
              repairWon({ sliceIndex: 1, },),
            ];
            /**
             The same records with the finding added.
             */
            const recorded = withTenseCandidates({
              slices,
              comparison: [
                insertionRow({
                  sliceIndex: 0,
                  translateText: 'She naps in the attic of the bookshop.',
                },),
                catRow({
                  sliceIndex: 1,
                  repairText: 'She naps in the bookshop attic.',
                  translateText: 'The cat dozed in the attic.',
                },),
              ],
              lifeEnded: true,
            },);
            expect(recorded[0],).toBe(sole,);
            expect(recorded[1]?.tenseCandidates?.length,).toBe(1,);
          },
        },),

        it({
          name: 'FINDS pronoun candidates in the records of an entry that declares no subject name',
          fn: async () => {
            /**
             Records the driver returned for one slice.
             */
            const slices = [repairWon({ sliceIndex: 0, },),];
            expect(withTenseCandidates({
              slices,
              comparison: [
                catRow({
                  sliceIndex: 0,
                  repairText: 'She naps in the bookshop attic.',
                  translateText: 'The cat dozed in the attic.',
                },),
              ],
              lifeEnded: true,
            },).map(function findingOf(slice,) {
              return slice.tenseCandidates;
            },),).toEqual([
              [
                {
                  label: 'CANDIDATE "repair"',
                  found: [
                    {
                      sentence: 'She naps in the bookshop attic.',
                      matched: 'She naps',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
            ],);
          },
        },),

        it({
          name: 'REFUSES a contest record the comparison does not hold, which the contest never produces',
          fn: async () => {
            const refusal = caught(function strayRecord() {
              return withTenseCandidates({
                slices: [repairWon({ sliceIndex: 5, },),],
                comparison: [
                  catRow({
                    sliceIndex: 0,
                    repairText: 'Mittens naps in the bookshop attic.',
                    translateText: 'The cat dozed in the attic.',
                  },),
                ],
                lifeEnded: true,
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              'Error: unreachable: a contest answers only slices the comparison holds, so every contest record has a '
                + 'comparison row',
            );
          },
        },),
      ],
    },),
  ],
},);

//endregion Lane contest tense tests
