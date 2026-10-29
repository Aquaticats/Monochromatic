/**
 Tests for the two roster readings a selection round makes before it asks a
 judge: a judge listed twice is refused, and each candidate position is read
 for the models with a stake in it.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ProducerRosterError, } from '@monochromatic-dev/translation-repair-assembly';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  assertDistinctJudges,
  stakesByIndexOf,
} from '../dist/final/node/index.mjs';

/**
 A proposal by one model beside the archive wording another model reproduced
 exactly, which is the stake shape a consolidation slate carries.
 */
const CANDIDATES: readonly Candidate<string>[] = [
  {
    producer: {
      kind: 'model',
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    },
    value: 'proposal',
    rendered: 'The cat chases butterflies.',
  },
  {
    producer: {
      kind: 'incumbent',
      matched: [SEAT_SYNTHETIC_VISION_WITHHELD,],
    },
    value: 'archive',
    rendered: 'The cat loves chasing butterflies.',
  },
];

await describe({
  name: 'selection roster checks',
  children: [
    describe({
      name: assertDistinctJudges.name,
      children: [
        it({
          name: 'PASSES a roster of distinct judges and REFUSES a judge listed twice before any model is asked',
          fn: async () => {
            // Returning at all is the pass: the check has no result to read.
            assertDistinctJudges({
              judges: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
              candidates: CANDIDATES,
            },);
            const refusal = caught(function repeatsAJudge() {
              assertDistinctJudges({
                judges: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                candidates: CANDIDATES,
              },);
            },);
            expect(refusal,).toBeInstanceOf(ProducerRosterError,);
          },
        },),
      ],
    },),
    describe({
      name: stakesByIndexOf.name,
      children: [
        it({
          name: 'KEYS EACH ONE-BASED POSITION to the models with a stake in it, a reproducing model included',
          fn: async () => {
            const stakes = stakesByIndexOf({ candidates: CANDIDATES, },);
            expect([...(stakes.get(1,) ?? []),],).toEqual([SEAT_HYPER_OPENROUTER_VISION_EDITOR,],);
            expect([...(stakes.get(2,) ?? []),],).toEqual([SEAT_SYNTHETIC_VISION_WITHHELD,],);
            expect(stakes.has(0,),).toBe(false,);
            expect(stakes.has(3,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
