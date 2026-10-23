/**
 Tests for reading the mark of a contest slice whose round was never opened for want of a bench.

 The mark says no roster was asked, so it is read as true or absent, and held to a record with no ballot.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseContestFloorMark, } from '../../dist/final/node/index.mjs';

await describe({
  name: parseContestFloorMark.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS AN ABSENT MARK AS NO MARK and a true mark on a slice with no ballot as the mark',
      fn: async () => {
        expect({
          absent: parseContestFloorMark({ value: undefined, ballotCount: 0, path: 'x.benchBelowFloor', },),
          marked: parseContestFloorMark({ value: true, ballotCount: 0, path: 'x.benchBelowFloor', },),
        },).toEqual({
          absent: {},
          marked: { benchBelowFloor: true, },
        },);
      },
    },),
    it({
      name: 'REFUSES A MARK THAT IS NOT TRUE, and a true mark on a slice that carries ballots, naming the path',
      fn: async () => {
        /**
         Refusal text for one value and ballot count.

         @param value - recorded mark
         @param ballotCount - ballots the record carries

         @returns The refusal
         */
        function refusalOf(
          {
            value,
            ballotCount,
          }: {
            readonly value: unknown;
            readonly ballotCount: number;
          },
        ): string {
          const refusal = caught(function readMark() {
            return parseContestFloorMark({ value, ballotCount, path: 'x.benchBelowFloor', },);
          },);
          expect(refusal,).toBeInstanceOf(ArtifactParseError,);
          return String(refusal,);
        }
        expect({
          falsy: refusalOf({ value: false, ballotCount: 0, },),
          answered: refusalOf({ value: true, ballotCount: 2, },),
        },).toEqual({
          falsy: 'ArtifactParseError: artifact parse failed at x.benchBelowFloor: expected true or absent.',
          answered: 'ArtifactParseError: artifact parse failed at x.benchBelowFloor: expected no ballots, since a '
            + 'bench read below its floor was not asked, rather than 2.',
        },);
      },
    },),
  ],
},);
