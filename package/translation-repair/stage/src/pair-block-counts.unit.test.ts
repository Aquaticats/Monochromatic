/**
 Tests for many-to-many block diagnostic counts.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { countPairedBlocks, } from '../dist/final/node/index.mjs';

await describe({
  name: countPairedBlocks.name,
  children: [
    it({
      name: 'COUNTS REPEATED SOURCE ONCE while retaining both relations, matching one-to-many alignment',
      fn: async () => {
        expect(countPairedBlocks({
          pairs: [
            { source: 0, target: 0, },
            { source: 1, target: 1, },
            { source: 1, target: 2, },
          ],
        },),).toEqual({
          source: 2,
          target: 3,
          relations: 3,
        },);
      },
    },),

    it({
      name: 'COUNTS REPEATED TARGET ONCE while retaining both relations, matching many-to-one alignment',
      fn: async () => {
        expect(countPairedBlocks({
          pairs: [
            { source: 0, target: 0, },
            { source: 1, target: 1, },
            { source: 2, target: 1, },
          ],
        },),).toEqual({
          source: 3,
          target: 2,
          relations: 3,
        },);
      },
    },),
  ],
},);
