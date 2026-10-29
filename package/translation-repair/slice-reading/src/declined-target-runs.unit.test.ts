/**
 Tests for which translation blocks a pairing declined.

 WHY THIS FILE EXISTS. Grouping and the coverage assertion both ask this
 question, and they must derive the same answer from the same inputs, since a
 disagreement between them reads as a coverage fault at a place neither one
 caused. The answer is built by turning the pairing into alignment steps,
 which needs to be told HOW MANY blocks each side has.

 WHAT WAS MEASURED. On 2026-08-25, swapping those two counts failed no test
 in this package. A swap is silent in the common case, because most pairs
 carry equal counts, and it only shows itself where the two sides differ:
 exactly the entries the pairing exists for.

 THE FIXTURE THEREFORE MAKES THEM DIFFER, two originals against three
 translation blocks. Under the counts as passed, the third translation block
 is declined; under the swap, the walk stops one block early and reports
 nothing declined at all, which would hand grouping a block no slice covers
 while telling the coverage assertion everything was accounted for.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type BlockPair,
  BlockPairingError,
} from '@monochromatic-dev/translation-repair-stage';
import { declinedTargetIdsOfPairing, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Original side, two paragraphs.
 */
const SOURCE_PAGE = `Mittens slept on the sill until noon.

Whiskers counted the birds outside.
`;

/**
 Translation side, three paragraphs, of which the last answers to no original.
 */
const TARGET_PAGE = `Mittens slept on the sill until noon.

Whiskers counted the birds outside.

Her brother brought her a feather.
`;

/**
 Original side of three paragraphs, for a pairing that leaves the middle one
 unplaced.
 */
const LONGER_SOURCE_PAGE = `Mittens slept on the sill until noon.

Whiskers counted the birds outside.

Tabby chased a moth along the fence.
`;

/**
 Original side of one paragraph, for a pairing that renders it more than once.
 */
const ONE_BLOCK_SOURCE_PAGE = `Mittens slept on the sill until noon.
`;

/**
 Translation side of two paragraphs, the first rendering both paragraphs of
 the two-paragraph original and the second answering to neither.
 */
const MERGED_TARGET_PAGE = `Mittens slept on the sill while Whiskers counted the birds.

Her brother brought her a feather.
`;

/**
 Correspondences the roster returned, leaving the third block unclaimed.
 */
const PAIRS = [
  {
    source: 0,
    target: 0,
  },
  {
    source: 1,
    target: 1,
  },
] as const satisfies readonly BlockPair[];

//endregion Fixtures

await describe({
  name: declinedTargetIdsOfPairing.name,
  children: [
    it({
      name: 'NAMES the translation block no original claimed, reading each side by its own count, so a '
        + 'page with more blocks than its original does not report everything accounted for',
      fn: async () => {
        /**
         Blocks the pairing left for no slice to cover.
         */
        const declined = declinedTargetIdsOfPairing({
          pairs: [...PAIRS,],
          sourceNodes: parseDocument({ text: SOURCE_PAGE, },).nodes,
          targetNodes: parseDocument({ text: TARGET_PAGE, },).nodes,
        },);

        expect([...declined,],).toEqual(['block/2',],);
      },
    },),

    it({
      name: 'DECLINES NOTHING for an empty pairing over a page whose original holds blocks, the pairing a '
        + 'pass hands on when no voice was usable',
      fn: async () => {
        expect([...declinedTargetIdsOfPairing({
          pairs: [],
          sourceNodes: parseDocument({ text: SOURCE_PAGE, },).nodes,
          targetNodes: parseDocument({ text: TARGET_PAGE, },).nodes,
        },),],).toEqual([],);
      },
    },),

    it({
      name: 'DECLINES NOTHING where one original stays unplaced between two pairings, though the last '
        + 'translation block answers to no original',
      fn: async () => {
        expect([...declinedTargetIdsOfPairing({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 2,
              target: 1,
            },
          ],
          sourceNodes: parseDocument({ text: LONGER_SOURCE_PAGE, },).nodes,
          targetNodes: parseDocument({ text: TARGET_PAGE, },).nodes,
        },),],).toEqual([],);
      },
    },),

    it({
      name: 'DECLINES NOTHING where the original holds no block at all, since a pairing that placed nothing '
        + 'declines nothing',
      fn: async () => {
        expect([...declinedTargetIdsOfPairing({
          pairs: [],
          sourceNodes: [],
          targetNodes: parseDocument({ text: 'A cat naps on the mat.', },).nodes,
        },),],).toEqual([],);
      },
    },),

    it({
      name: 'KEEPS a translation block out of the declines where it sits between two renderings of one '
        + 'original, and DECLINES it with the block after it where the pairing names the first rendering alone',
      fn: async () => {
        /**
         One original the translation renders as its first block and its third.
         */
        const sourceNodes = parseDocument({ text: ONE_BLOCK_SOURCE_PAGE, },).nodes;
        /**
         The three translation blocks.
         */
        const targetNodes = parseDocument({ text: TARGET_PAGE, },).nodes;

        expect([...declinedTargetIdsOfPairing({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 0,
              target: 2,
            },
          ],
          sourceNodes,
          targetNodes,
        },),],).toEqual([],);
        expect([...declinedTargetIdsOfPairing({
          pairs: [{
            source: 0,
            target: 0,
          },],
          sourceNodes,
          targetNodes,
        },),],).toEqual([
          'block/1',
          'block/2',
        ],);
      },
    },),

    it({
      name: 'DECLINES a block beside a merge, since an original riding along with its neighbour\'s rendering '
        + 'is placed',
      fn: async () => {
        expect([...declinedTargetIdsOfPairing({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 1,
              target: 0,
            },
          ],
          sourceNodes: parseDocument({ text: SOURCE_PAGE, },).nodes,
          targetNodes: parseDocument({ text: MERGED_TARGET_PAGE, },).nodes,
        },),],).toEqual(['block/1',],);
      },
    },),

    it({
      name: 'REFUSES a pairing whose one pair names the block after the last, which read alone would '
        + 'decline every block the page holds, naming that block and the count (ledger B138)',
      fn: async () => {
        /**
         What the read threw.
         */
        const refusal = caught(function declines(): unknown {
          return declinedTargetIdsOfPairing({
            pairs: [{
              source: 0,
              target: 3,
            },],
            sourceNodes: parseDocument({ text: ONE_BLOCK_SOURCE_PAGE, },).nodes,
            targetNodes: parseDocument({ text: TARGET_PAGE, },).nodes,
          },);
        },);
        expect(refusal,).toBeInstanceOf(BlockPairingError,);
        expect(String(refusal,),).toBe('BlockPairingError: pairing names translation block 3, and there are 3',);
      },
    },),

    it({
      name: 'REFUSES a pairing naming a block further past the page, naming the block the pair names rather '
        + 'than an unclaimed one before it (ledger B138)',
      fn: async () => {
        /**
         What the read threw.
         */
        const refusal = caught(function declines(): unknown {
          return declinedTargetIdsOfPairing({
            pairs: [{
              source: 0,
              target: 4,
            },],
            sourceNodes: parseDocument({ text: ONE_BLOCK_SOURCE_PAGE, },).nodes,
            targetNodes: parseDocument({ text: TARGET_PAGE, },).nodes,
          },);
        },);
        expect(refusal,).toBeInstanceOf(BlockPairingError,);
        expect(String(refusal,),).toBe('BlockPairingError: pairing names translation block 4, and there are 3',);
      },
    },),
  ],
},);
