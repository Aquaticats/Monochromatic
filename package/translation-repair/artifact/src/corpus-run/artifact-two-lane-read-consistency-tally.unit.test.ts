/**
 Tests for what the recount of a group and the recount of a carry both ask of
 their ballots: that no seat spoke twice, and which place a group's recorded
 winner names.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyWinner, } from '@monochromatic-dev/translation-repair-consistency';
import {
  type ArtifactConsistencyGroup,
  assertSeatsHeardOnce,
  winningPositionOf,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 A group the bench was asked about, with the winner a case gives.

 @param winner - winner recorded

 @returns Group

 @example
 ```ts
 const group = groupWith({ winner: { kind: 'chosen', label: 'OCCURRENCE 2', }, },);
 ```
 */
function groupWith(
  { winner, }: { readonly winner: ConsistencyWinner; },
):ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; } {
  return {
    repeat: { kind: 'maximal', wording: '小猫打盹', },
    occurrences: ['OCCURRENCE 1', 'OCCURRENCE 2',].map(function placeOf(label, at,) {
      return {
        label,
        sliceIndex: at,
        startOffset: at * 10,
        endOffset: (at * 10) + 4,
        line: at + 1,
        rendering: { kind: 'unlocated', reason: 'no-agreement', } as const,
        outcome: 'untouched' as const,
      };
    },),
    unplaced: [],
    verdict: { kind: 'asked', ballots: [], usable: 0, quorum: 1, winner, },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertSeatsHeardOnce.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS SEATS THAT ARE ALL DIFFERENT AND NONE AT ALL',
          fn: async () => {
            expect([
              assertSeatsHeardOnce({ seats: ['hyper-cat', 'bedrock-cat',], path: 'cat.ballots', },),
              assertSeatsHeardOnce({ seats: [], path: 'cat.ballots', },),
            ],).toEqual([undefined, undefined,],);
          },
        },),
        it({
          name: 'REFUSES A SEAT AT ITS SECOND POSITION, naming the path of the ballot that repeats it',
          fn: async () => {
            expect(refusalTextOf({
              read: function readsSeats(): unknown {
                return assertSeatsHeardOnce({ seats: ['hyper-cat', 'bedrock-cat', 'hyper-cat',], path: 'cat.ballots', },);
              },
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.ballots[2].seat: '
                + 'expected a seat heard once, since a gather hears each seat once.',
            );
          },
        },),
      ],
    },),
    describe({
      name: winningPositionOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE POSITION OF THE PLACE THE WINNER NAMES, and minus one when no place won or the label names no place',
          fn: async () => {
            expect([
              winningPositionOf({ group: groupWith({ winner: { kind: 'chosen', label: 'OCCURRENCE 2', }, },), },),
              winningPositionOf({ group: groupWith({ winner: { kind: 'chosen', label: 'OCCURRENCE 1', }, },), },),
              winningPositionOf({ group: groupWith({ winner: { kind: 'none', reason: 'no-majority', }, },), },),
              winningPositionOf({ group: groupWith({ winner: { kind: 'chosen', label: 'OCCURRENCE 9', }, },), },),
            ],).toEqual([1, 0, -1, -1,],);
          },
        },),
      ],
    },),
  ],
},);
