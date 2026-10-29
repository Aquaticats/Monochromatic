/**
 Guards class one hundred six and its predicate. A strict majority of the
 usable contest ballots calling the archive flawed (owner answers 2026-10-11,
 "Majority flawed yes" and "Majority, like the run-off") decides both whether a
 tied slate over an eligible standing is run off and whether that standing is
 forfeit past an undecided gate: zheermao8 slice 9's contest split 2 to 2 with
 every ballot calling the archive flawed, and the archive's wording shipped
 because an eligible standing keeps its single round. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeContestSlice,
  type LaneContestBallot,
  withTenseCandidates,
} from '@monochromatic-dev/translation-repair-lane-contest';
import { archiveFlawedByMajority, } from '../dist/final/node/index.mjs';
import { catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';

/**
 One usable ballot with the archive verdict a case needs.

 @param archive - what this judge made of the archive, absent when it did
 not say

 @returns Ballot naming the repair lane

 @example
 ```ts
 const ballot = ballotOf({ archive: 'flawed', },);
 ```
 */
function ballotOf(
  { archive, }: { readonly archive?: 'publishable' | 'flawed'; },
): LaneContestBallot {
  return {
    choice: 'repair',
    unsupported: [],
    unsupportedRaw: [],
    dropped: [],
    droppedRaw: [],
    reason: 'the cat naps',
    ...((archive === undefined) ? {} : { archive, }),
  };
}

await describe({
  name: 'archive flawed predicates',
  children: [
    it({
      name: 'READS a contest settled on neither with every ballot calling the archive flawed',
      fn: async () => {
        expect(archiveFlawedByMajority({
          verdict: { kind: 'settled-neither', archive: 'declined', },
          ballots: [
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'flawed', },),
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'READS a contest settled on neither where three of its five usable ballots call the archive flawed and two call it publishable, a strict majority (owner answer 2026-10-11, "Majority flawed yes")',
      fn: async () => {
        expect(archiveFlawedByMajority({
          verdict: { kind: 'settled-neither', archive: 'declined', },
          ballots: [
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'publishable', },),
            ballotOf({ archive: 'publishable', },),
          ],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'STANDS ASIDE from the run-off where two of four usable ballots call the archive flawed, since two of four is not a strict majority (owner answer 2026-10-11, "Majority flawed yes")',
      fn: async () => {
        expect(archiveFlawedByMajority({
          verdict: { kind: 'settled-neither', archive: 'declined', },
          ballots: [
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'publishable', },),
            ballotOf({ archive: 'publishable', },),
          ],
        },),).toBe(false,);
      },
    },),
    it({
      name: 'COUNTS THE SAME for a contest slice that records tense candidates, since the finding is no ballot and no verdict: the run-off follows the ballots alone',
      fn: async () => {
        /**
         Ballots every judge cast calling the archive flawed.
         */
        const ballots = [
          ballotOf({ archive: 'flawed', },),
          ballotOf({ archive: 'flawed', },),
        ];
        /**
         The slice as the contest records it, before any finding.
         */
        const settled = describeContestSlice({
          sliceIndex: 0,
          outcome: {
            choice: 'neither',
            ballots,
            usable: 2,
            findings: [],
          },
        },);
        /**
         The same slice with a tense candidate in the archive wording.
         */
        const [recorded,] = withTenseCandidates({
          slices: [settled,],
          comparison: [
            catRow({
              sliceIndex: 0,
              repairText: 'The cat napped by the window.',
              translateText: 'The cat slept by the window.',
            },),
          ],
          lifeEnded: true,
          subjectNames: ['The cat',],
        },);
        expect(recorded?.tenseCandidates?.length,).toBe(1,);
        expect(archiveFlawedByMajority({
          verdict: recorded?.verdict ?? settled.verdict,
          ballots: recorded?.ballots ?? [],
        },),).toBe(true,);
      },
    },),
    it({
      name: 'STANDS ASIDE where one ballot would publish the archive, where one ballot did not say, where the contest chose a lane, and where no ballot was usable',
      fn: async () => {
        expect(archiveFlawedByMajority({
          verdict: { kind: 'settled-neither', },
          ballots: [
            ballotOf({ archive: 'flawed', },),
            ballotOf({ archive: 'publishable', },),
          ],
        },),).toBe(false,);
        expect(archiveFlawedByMajority({
          verdict: { kind: 'settled-neither', archive: 'declined', },
          ballots: [
            ballotOf({ archive: 'flawed', },),
            ballotOf({},),
          ],
        },),).toBe(false,);
        expect(archiveFlawedByMajority({
          verdict: { kind: 'lane-won', lane: 'repair', },
          ballots: [ballotOf({ archive: 'flawed', },),],
        },),).toBe(false,);
        expect(archiveFlawedByMajority({
          verdict: { kind: 'quorum-not-met', },
          ballots: [ballotOf({ archive: 'flawed', },),],
        },),).toBe(false,);
        expect(archiveFlawedByMajority({
          verdict: { kind: 'settled-neither', },
          ballots: [],
        },),).toBe(false,);
      },
    },),
  ],
},);
