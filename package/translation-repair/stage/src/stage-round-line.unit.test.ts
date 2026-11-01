/**
 Tests for the line a gather round logs about itself (ledger B82).

 WHY BOTH KINDS. A round whose quorum stood measured time either side of it;
 one whose quorum never stood measured neither, and writing zeros there would
 report a measurement the round never made (ledger P12). Each kind is pinned
 whole, since the run-timing reader parses this exact text.

 Fixtures are invented counts and durations; the stage labels are the
 package's own.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { roundLine, } from '../dist/final/node/index.mjs';

await describe({
  name: roundLine.name,
  children: [
    it({
      name: 'STATES THE TIME EITHER SIDE OF QUORUM where it stood, after the ratio heard and the total',
      fn: async () => {
        expect(roundLine({
          stage: 'editor',
          heard: 6,
          asked: 7,
          totalMs: 91_402,
          quorum: {
            kind: 'stood',
            toQuorumMs: 61_401,
            inGraceMs: 30_001,
          },
        },),).toBe('editor round: 6/7 heard, 91402ms total, 61401ms to quorum, 30001ms in grace',);
      },
    },),
    it({
      name: 'STATES THE COUNT NEEDED where quorum never stood, and no time to quorum or in grace, since the '
        + 'round measured neither',
      fn: async () => {
        expect(roundLine({
          stage: 'translate',
          heard: 1,
          asked: 4,
          totalMs: 12_000,
          quorum: {
            kind: 'never',
            needed: 3,
          },
        },),).toBe('translate round: 1/4 heard, 12000ms total, no quorum (1 of 3 needed), every ask settled',);
      },
    },),
  ],
},);
