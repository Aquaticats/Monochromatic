/**
 Tests for never pooling results across builds.

 THE WHOLE POINT IS THE SPLIT. `artifact-pool.ts` refuses to pool results
 whose built output differs, and a standing summed across two builds describes
 neither of them. These cases pin that a collection spanning digests is never
 formed in the first place.

 Content is cat-themed invention; a digest here is any opaque string, so the
 fixtures spell them as such.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { groupByDigest, } from '../../dist/final/node/index.mjs';

/**
 One build's identifier.
 */
const TABBY = 'sha256-tree-v1:tabby';

/**
 A second build's identifier.
 */
const CALICO = 'sha256-tree-v1:calico';

await describe({
  name: groupByDigest.name,
  children: [
    it({
      name: 'REFUSES to pool two builds, splitting them instead',
      fn: async () => {
        /**
         Two readings from one build and one from another, interleaved so a
         grouping that only compared neighbours would get it wrong.
         */
        const groups = groupByDigest({
          readings: [
            {
              digest: TABBY,
              entryId: 'one',
            },
            {
              digest: CALICO,
              entryId: 'two',
            },
            {
              digest: TABBY,
              entryId: 'three',
            },
          ],
        },);

        expect(groups.length,).toBe(2,);
        expect(groups[0]?.digest,).toBe(TABBY,);
        expect(groups[0]?.readings.length,).toBe(2,);
        expect(groups[1]?.digest,).toBe(CALICO,);
        expect(groups[1]?.readings.length,).toBe(1,);
      },
    },),

    it({
      name: 'orders the largest group first, so the strongest evidence is read first',
      fn: async () => {
        /**
         A build with one reading arriving before a build with two.
         */
        const groups = groupByDigest({
          readings: [
            {
              digest: CALICO,
              entryId: 'one',
            },
            {
              digest: TABBY,
              entryId: 'two',
            },
            {
              digest: TABBY,
              entryId: 'three',
            },
          ],
        },);

        expect(groups[0]?.digest,).toBe(TABBY,);
      },
    },),

    it({
      name: 'keeps reading order inside a group, so a stable read gives a stable report',
      fn: async () => {
        /**
         Three readings of one build, read in a known order.
         */
        const groups = groupByDigest({
          readings: [
            {
              digest: TABBY,
              entryId: 'whiskers',
            },
            {
              digest: TABBY,
              entryId: 'mittens',
            },
            {
              digest: TABBY,
              entryId: 'socks',
            },
          ],
        },);

        expect(groups[0]?.readings.map(function toId(reading,): string {
          return reading.entryId;
        },),).toEqual([
          'whiskers',
          'mittens',
          'socks',
        ],);
      },
    },),

    it({
      name: 'ACCEPTS an empty archive, returning no groups rather than one empty one',
      fn: async () => {
        expect(groupByDigest({ readings: [], },).length,).toBe(0,);
      },
    },),
  ],
},);
