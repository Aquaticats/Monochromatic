/** Exercise the exported size-dismissal detection of the pi cost-dismissal shadow. @module */

import { createHash, } from 'node:crypto';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { findSizeDismissal, } from '../dist/final/node/index.mjs';

/**
 Pattern id the detector records for a sentence that carries cost and a dismissal together.
 */
const SIZE_DISMISSAL_PATTERN = 'size-dismissal';

/**
 Sentences shaped on one cost vocabulary entry each, every one of them waving the cost away.
 */
const DISMISSING_SENTENCES = [
  'The scan is quadratic, but the rows are short.',
  'It is a nested loop, though the input is tiny.',
  'That is exponential, but in practice fine.',
] as const;

/**
 Sentences that name the same costs without a dismissal, so every flagged vocabulary entry has a negative control.
 */
const COST_ONLY_SENTENCES = [
  'The scan is quadratic, so rewrite it as one linear pass.',
  'The accumulator makes it superlinear, so measure before shipping.',
  'A nested scan costs per row, and that must change.',
] as const;

/**
 Whole-unit digest the detector truncates for its record.
 */
function unitDigest(unit: string,): string {
  return createHash('sha256',)
    .update(unit,)
    .digest('hex',);
}

/**
 Recorded block hash of the first hit in one unit, and empty when the unit holds none.
 */
function firstBlockHash(unit: string,): string {
  return findSizeDismissal(unit,)[0]?.blockHash ?? '';
}

await describe({
  name: findSizeDismissal.name,
  children: [
    ...DISMISSING_SENTENCES.map(function flaggedTest(sentence,): ReturnType<typeof it> {
      return it({
        name: `flags ${JSON.stringify(sentence,)}`,
        fn: async (): Promise<void> => {
          /** One cost entry and one dismissal entry in one sentence is the whole pattern. */
          const hits = findSizeDismissal(sentence,);
          expect(hits,).toHaveLength(1,);
          expect(hits[0]?.pattern,).toBe(SIZE_DISMISSAL_PATTERN,);
          expect(hits[0]?.sentenceIndex,).toBe(0,);
        },
      },);
    },),
    ...COST_ONLY_SENTENCES.map(function unflaggedTest(sentence,): ReturnType<typeof it> {
      return it({
        name: `leaves ${JSON.stringify(sentence,)} unflagged`,
        fn: async (): Promise<void> => {
          /** Naming a cost without dismissing it is the behaviour the detector exists to leave alone. */
          expect(findSizeDismissal(sentence,),).toHaveLength(0,);
        },
      },);
    },),
    it({
      name: 'reports one hit per dismissing sentence, in sentence order',
      fn: async (): Promise<void> => {
        /** A unit folds into as many hits as it has dismissing sentences, indexed by position. */
        const [first, second,] = DISMISSING_SENTENCES;
        const unit = `${first} ${second}`;
        const hits = findSizeDismissal(unit,);
        expect(hits.map(function index(hit,): number {
          return hit.sentenceIndex;
        },),).toEqual([0, 1,],);
        expect(new Set(hits.map(function hash(hit,): string {
          return hit.blockHash;
        },)).size,).toBe(1,);
      },
    },),
    it({
      name: 'locates the unit by a truncated digest and records no prose',
      fn: async (): Promise<void> => {
        /** The record is joinable back to a unit without carrying any of its text. */
        const [sentence,] = DISMISSING_SENTENCES;
        const [hit,] = findSizeDismissal(sentence,);
        expect(hit,).toBeDefined();
        const recorded = hit?.blockHash ?? '';
        expect(recorded,).not.toContain('quadratic',);
        expect(unitDigest(sentence,),).toContain(recorded,);
        expect(recorded.length,).toBeLessThan(unitDigest(sentence,).length,);
      },
    },),
    it({
      name: 'keeps equal units together and different units apart',
      fn: async (): Promise<void> => {
        /** The same prose must join across runs, and prose differing by one trailing space must not. */
        const [sentence,] = DISMISSING_SENTENCES;
        expect(firstBlockHash(sentence,),).toBe(firstBlockHash(sentence,),);
        expect(firstBlockHash(sentence,),).not.toBe(firstBlockHash(`${sentence} `,),);
      },
    },),
  ],
},);
