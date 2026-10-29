/**
 Tests for the seeded draw of shipped regions and the case each draw makes.

 THE DRAW MUST BE REPRODUCIBLE FROM ITS SEED ALONE, whatever order artifacts
 sit in on disk, and must cap at the sheet size. The order itself is pinned
 against digests read off a run, so a change to the domain or the field
 separator that would let two draws collide shows here.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildCase,
  drawRegions,
  type ShippedRegion,
} from '../../dist/final/node/index.mjs';

/**
 Builds one shipped region of the repair lane.

 @param entryId - corpus entry

 @param sliceIndex - slice the lane replaced

 @returns Region as the damage reader builds one

 @example
 ```ts
 const region = regionAt({ entryId: 'mittens', sliceIndex: 0, },);
 ```
 */
function regionAt(
  {
    entryId,
    sliceIndex,
  }: {
    readonly entryId: string;
    readonly sliceIndex: number;
  },
): ShippedRegion {
  return {
    entryId,
    lane: 'repair',
    sliceIndex,
    regionId: `repair#${String(sliceIndex,)}`,
    sourceText: '猫猫在窗台上睡觉。',
    incumbentText: 'The cat sleeps on the sill.',
    shippedText: 'The cat naps on the windowsill.',
    pageRelation: { kind: 'survives', },
  };
}

/**
 Names a drawn region for comparison.

 @param region - drawn region

 @returns Entry and region identity

 @example
 ```ts
 const name = nameOf({ region, },);
 ```
 */
function nameOf({ region, }: { readonly region: ShippedRegion; },): string {
  return `${region.entryId} ${region.regionId}`;
}

/**
 Regions of four entries, in the order a directory happened to list them.
 */
const POOL: readonly ShippedRegion[] = [
  regionAt({
    entryId: 'tabby',
    sliceIndex: 0,
  },),
  regionAt({
    entryId: 'mittens',
    sliceIndex: 0,
  },),
  regionAt({
    entryId: 'mittens',
    sliceIndex: 1,
  },),
  regionAt({
    entryId: 'biscuit',
    sliceIndex: 2,
  },),
];

await describe({
  name: 'damage-sample-draw',
  children: [
    describe({
      name: drawRegions.name,
      children: [
        it({
          name: 'ORDERS the pool by the digest of the seed and the region identity, not by the order given',
          fn: async () => {
            expect(drawRegions({
              regions: POOL,
              seed: 'damage-round-one',
            },).map(function named(region,): string {
              return nameOf({ region, },);
            },),).toEqual([
              'mittens repair#1',
              'mittens repair#0',
              'biscuit repair#2',
              'tabby repair#0',
            ],);
          },
        },),
        it({
          name: 'DRAWS the same regions in the same order whichever order the pool lists them in',
          fn: async () => {
            expect(drawRegions({
              regions: POOL.toReversed(),
              seed: 'damage-round-one',
            },).map(function named(region,): string {
              return nameOf({ region, },);
            },),).toEqual([
              'mittens repair#1',
              'mittens repair#0',
              'biscuit repair#2',
              'tabby repair#0',
            ],);
          },
        },),
        it({
          name: 'DRAWS another order for another seed',
          fn: async () => {
            expect(drawRegions({
              regions: POOL,
              seed: 'damage-round-two',
            },).map(function named(region,): string {
              return nameOf({ region, },);
            },),).toEqual([
              'tabby repair#0',
              'mittens repair#1',
              'mittens repair#0',
              'biscuit repair#2',
            ],);
          },
        },),
        it({
          name: 'RETURNS nothing for an empty pool',
          fn: async () => {
            expect(drawRegions({
              regions: [],
              seed: 'damage-round-one',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'CAPS the draw at twenty regions from a pool of twenty-five',
          fn: async () => {
            /**
             Regions of one entry, one per slice.
             */
            const many = Array.from(
              { length: 25, },
              function regionOfSlice(
                _unused,
                sliceIndex,
              ): ShippedRegion {
                return regionAt({
                  entryId: 'mittens',
                  sliceIndex,
                },);
              },
            );

            expect(drawRegions({
              regions: many,
              seed: 'damage-round-one',
            },).map(function sliceOf(region,): number {
              return region.sliceIndex;
            },),).toEqual([
              1,
              0,
              22,
              7,
              12,
              17,
              14,
              16,
              19,
              8,
              2,
              23,
              13,
              9,
              10,
              24,
              5,
              15,
              4,
              6,
            ],);
          },
        },),
      ],
    },),
    describe({
      name: buildCase.name,
      children: [
        it({
          name: 'MAKES the whole slice the region, with the archive wording as both the before text and the baseline, '
            + 'and no issue withheld or recorded',
          fn: async () => {
            expect(buildCase({
              ref: regionAt({
                entryId: 'mittens',
                sliceIndex: 0,
              },),
            },),).toEqual({
              entryId: 'mittens',
              positions: [],
              region: {
                envelopeId: 'repair#0',
                issueIds: [],
                before: 'The cat sleeps on the sill.',
                editorAfter: 'The cat naps on the windowsill.',
              },
              issues: [],
              sourceText: '猫猫在窗台上睡觉。',
              baselineText: 'The cat sleeps on the sill.',
              recorded: '',
            },);
          },
        },),
      ],
    },),
  ],
},);
