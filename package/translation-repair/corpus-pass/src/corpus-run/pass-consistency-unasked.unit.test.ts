/**
 Tests for settling a repeat group the bench is not asked about, or whose
 handling failed: every place stays as it stands, and the group says why.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RepeatGroup, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ArtifactConsistencyRepeat, } from '@monochromatic-dev/translation-repair-artifact';
import {
  type ConsistencyPlace,
  settleFailed,
  settleUnasked,
  unplacedOf,
} from '../../dist/final/node/index.mjs';
import { placeIn, } from './consistency-settle.test-fixture.ts';

/**
 The repeated sentence, as the artifact records it.
 */
const SENTENCE: ArtifactConsistencyRepeat = {
  kind: 'maximal',
  wording: '小猫打盹',
};

/**
 The refrain, as the artifact records it.
 */
const FRAME: ArtifactConsistencyRepeat = {
  kind: 'refrain',
  prefix: '所以这只猫',
  suffix: '的小宝贝',
};

/**
 Three places of the repeated sentence.
 */
const PLACES: readonly ConsistencyPlace[] = [
  placeIn({ position: 0, english: 'The cat naps. It is warm.', },),
  placeIn({ position: 1, english: 'Later the kitten naps in the sun.', },),
  placeIn({ position: 2, english: 'Evening: The cat naps in the dark.', },),
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: settleUnasked.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES EVERY PLACE OF A GROUP THAT READ ALIKE CONSISTENT, the bench not asked and no replacement made',
          fn: async () => {
            expect(settleUnasked({
              repeat: SENTENCE,
              places: PLACES,
              unplaced: [],
              reason: 'consistent',
            },),).toEqual({
              repeat: SENTENCE,
              places: PLACES,
              unplaced: [],
              verdict: { kind: 'not-asked', reason: 'consistent', },
              renderings: [
                { kind: 'unlocated', reason: 'not-asked', },
                { kind: 'unlocated', reason: 'not-asked', },
                { kind: 'unlocated', reason: 'not-asked', },
              ],
              outcomes: ['consistent', 'consistent', 'consistent',],
              edits: [],
              carries: [],
              findings: [],
            },);
          },
        },),
        it({
          name: 'LEAVES EVERY PLACE OF A GROUP WITH TOO FEW PLACES OR NO ENGLISH UNTOUCHED, naming the reason',
          fn: async () => {
            expect((['too-few-placed', 'no-english',] as const).map(function settle(reason,): unknown {
              const settled = settleUnasked({
                repeat: SENTENCE,
                places: PLACES.slice(0, 2,),
                unplaced: [],
                reason,
              },);
              return [settled.verdict, settled.outcomes, settled.carries,];
            },),).toEqual([
              [{ kind: 'not-asked', reason: 'too-few-placed', }, ['untouched', 'untouched',], [],],
              [{ kind: 'not-asked', reason: 'no-english', }, ['untouched', 'untouched',], [],],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: settleFailed.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES EVERY PLACE OF A GROUP WHOSE HANDLING FAILED UNTOUCHED, recording the detail, no replacement and no carry',
          fn: async () => {
            const unplaced = [{ startOffset: 40, endOffset: 44, line: 5, reason: 'in-no-slice', },] as const;
            expect(settleFailed({
              repeat: FRAME,
              places: PLACES,
              unplaced,
              detail: 'refused by ConsistencyInvariantError',
            },),).toEqual({
              repeat: FRAME,
              places: PLACES,
              unplaced,
              verdict: { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
              renderings: [
                { kind: 'unlocated', reason: 'not-asked', },
                { kind: 'unlocated', reason: 'not-asked', },
                { kind: 'unlocated', reason: 'not-asked', },
              ],
              outcomes: ['untouched', 'untouched', 'untouched',],
              edits: [],
              carries: [],
              findings: [],
            },);
          },
        },),
      ],
    },),
    describe({
      name: unplacedOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS EACH PLACE NO SLICE HOLDS WHOLE WITH ITS OFFSETS, LINE AND REASON, in the detector\'s order',
          fn: async () => {
            const group: RepeatGroup = {
              kind: 'maximal',
              length: 4,
              occurrences: [],
              unplaced: [
                { startOffset: 40, endOffset: 44, line: 5, reason: 'crosses-slice-end', },
                { startOffset: 60, endOffset: 64, line: 7, reason: 'in-no-slice', },
              ],
            };
            expect(unplacedOf({ group, },),).toEqual([
              { startOffset: 40, endOffset: 44, line: 5, reason: 'crosses-slice-end', },
              { startOffset: 60, endOffset: 64, line: 7, reason: 'in-no-slice', },
            ],);
            expect(unplacedOf({ group: { ...group, unplaced: [], }, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
