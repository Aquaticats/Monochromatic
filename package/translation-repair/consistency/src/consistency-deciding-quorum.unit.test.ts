/**
 Tests for the quorum a consistency count applies: the gather's, never below the two voices a decision needs.

 Fixtures are plain numbers; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { decidingQuorumOf, } from '../dist/final/node/index.mjs';

await describe({
  name: decidingQuorumOf.name,
  children: [
    it({
      name: 'RAISES A QUORUM OF ONE (a seated bench of one or two seats) to two, since one judge alone never decides '
        + '(the lead\'s ruling of 2026-10-25, on the owner\'s rule), and a quorum of nothing too',
      fn: async () => {
        expect([
          decidingQuorumOf({ quorum: 1, },),
          decidingQuorumOf({ quorum: 0, },),
        ],).toEqual([2, 2,],);
      },
    },),
    it({
      name: 'KEEPS A QUORUM OF TWO OR MORE as the gather closed on it, the positive control: the floor raises nothing '
        + 'that already stands above it',
      fn: async () => {
        expect([
          decidingQuorumOf({ quorum: 2, },),
          decidingQuorumOf({ quorum: 3, },),
          decidingQuorumOf({ quorum: 5, },),
        ],).toEqual([2, 3, 5,],);
      },
    },),
  ],
},);
