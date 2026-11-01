/**
 Tests for the stratified draw: how the band quota fills and how ties order.
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
  allocateBandQuota,
  drawStratifiedSample,
  type GradingCandidate,
} from '../dist/final/node/index.mjs';

/**
 One candidate for the draw fixtures.

 @param fields - entry the issue came from, size band of its source and the issue itself

 @returns Candidate shaped like the grading reader builds

 @example
 ```ts
 const candidate = candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-a', },);
 ```
 */
function candidateOf(
  {
    entryId,
    band,
    issueId,
  }: {
    readonly entryId: string;
    readonly band: GradingCandidate['band'];
    readonly issueId: string;
  },
): GradingCandidate {
  return {
    entryId,
    band,
    issueId,
    category: 'accuracy/omission',
    severity: 'minor',
    summary: 'A clause the translation dropped.',
    sourceAnchor: 'unanchored',
    sourceQuotes: [],
    targetQuotes: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: allocateBandQuota.name,
      children: [
        it({
          name: 'FILLS THE BANDS ONE SLOT A PASS up to the size asked, and no further than the bands '
            + 'can collectively hold',
          fn: async () => {
            expect(allocateBandQuota({
              available: {
                small: 2,
                medium: 1,
                large: 1,
              },
              size: 3,
            },),).toEqual({
              small: 1,
              medium: 1,
              large: 1,
            },);
            expect(allocateBandQuota({
              available: {
                small: 3,
                medium: 0,
                large: 0,
              },
              size: 2,
            },),).toEqual({
              small: 2,
              medium: 0,
              large: 0,
            },);
            expect(allocateBandQuota({
              available: {
                small: 1,
                medium: 1,
                large: 1,
              },
              size: 99,
            },),).toEqual({
              small: 1,
              medium: 1,
              large: 1,
            },);
          },
        },),

        it({
          name: 'REFUSES a size that is not a whole number of slots, zero or more, where a fraction '
            + 'once drew a slot from every band on every pass (two and a half slots asked, nine '
            + 'handed out)',
          fn: async () => {
            for (const size of [2.5, -1, Number.NaN, Number.POSITIVE_INFINITY,]) {
              /**
               What the allocation raised, read for its class and its whole
               wording.
               */
              const refusal = caught(function allocate(): unknown {
                return allocateBandQuota({
                  available: {
                    small: 10,
                    medium: 10,
                    large: 10,
                  },
                  size,
                },);
              },);
              expect(refusal,).toBeInstanceOf(RangeError,);
              expect(String(refusal,),).toBe(
                `RangeError: A sample size is a whole number of slots, zero or more; received ${String(size,)}.`,
              );
            }
          },
        },),

        it({
          name: 'HANDS OUT NOTHING for a size of zero',
          fn: async () => {
            expect(allocateBandQuota({
              available: {
                small: 10,
                medium: 10,
                large: 10,
              },
              size: 0,
            },),).toEqual({
              small: 0,
              medium: 0,
              large: 0,
            },);
          },
        },),
      ],
    },),

    describe({
      name: drawStratifiedSample.name,
      children: [
        it({
          name: 'BREAKS A RANK TIE BETWEEN ENTRIES by the entry shuffle, whatever order the candidates '
            + 'arrived in',
          fn: async () => {
            /**
             Two entries holding one candidate each in one band, so both
             sit at rank zero in their own entry.
             */
            const drawn = drawStratifiedSample({
              candidates: [
                candidateOf({ entryId: 'Zeta', band: 'small', issueId: 'issue-a', },),
                candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-b', },),
              ],
              size: 2,
              seed: 'meow',
            },);
            /**
             The same candidates arriving in the other order.
             */
            const reversed = drawStratifiedSample({
              candidates: [
                candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-b', },),
                candidateOf({ entryId: 'Zeta', band: 'small', issueId: 'issue-a', },),
              ],
              size: 2,
              seed: 'meow',
            },);
            expect(drawn.map(function entryOf(candidate,) {
              return candidate.entryId;
            },),).toEqual(['Zeta', 'Alpha'],);
            expect(reversed.map(function entryOf(candidate,) {
              return candidate.entryId;
            },),).toEqual(['Zeta', 'Alpha'],);
          },
        },),

        it({
          name: 'ORDERS ONE ENTRY\'S CANDIDATES by the issue shuffle, whatever order they arrived in',
          fn: async () => {
            /**
             Two candidates of one entry in one band, ranked by their
             shuffled issue keys. This seed puts issue-b first, against
             both arrival order and alphabetical order, so a sort by
             arrival or by id fails it.
             */
            const drawn = drawStratifiedSample({
              candidates: [
                candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-b', },),
                candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-a', },),
              ],
              size: 2,
              seed: 'purr',
            },);
            /**
             The same candidates arriving in the other order.
             */
            const reversed = drawStratifiedSample({
              candidates: [
                candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-a', },),
                candidateOf({ entryId: 'Alpha', band: 'small', issueId: 'issue-b', },),
              ],
              size: 2,
              seed: 'purr',
            },);
            expect(drawn.map(function issueOf(candidate,) {
              return candidate.issueId;
            },),).toEqual(['issue-b', 'issue-a'],);
            expect(reversed.map(function issueOf(candidate,) {
              return candidate.issueId;
            },),).toEqual(['issue-b', 'issue-a'],);
          },
        },),
      ],
    },),
  ],
},);
