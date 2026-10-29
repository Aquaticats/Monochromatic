/**
 Tests for the completion cap every client sends as `max_tokens`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMPLETION_CAP,
  completionCapFor,
  MODEL_CARDS,
  ROSTER_MODEL_IDS,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Caps the roster's cards resolve to, for the cards naming one pooled percentile.

 @param pool - pooled percentile's name as a card writes it

 @returns Every distinct cap those cards carry

 @example
 ```ts
 const caps = capsNaming({ pool: 'pooled-p99', },);
 ```
 */
function capsNaming({ pool, }: { readonly pool: 'pooled-p90' | 'pooled-p99'; },): readonly number[] {
  return [
    ...new Set(
      ROSTER_MODEL_IDS
        .filter(function names(modelId,): boolean {
          return MODEL_CARDS[modelId].completionCap === pool;
        },)
        .map(function capOf(modelId,): number {
          return COMPLETION_CAP[modelId];
        },),
    ),
  ];
}

await describe({
  name: completionCapFor.name,
  children: [
    it({
      name: 'CARRIES a measured cap for every roster seat, so no client can send a call without one',
      fn: async () => {
        expect(Object.keys(COMPLETION_CAP,).toSorted(),).toEqual([...ROSTER_MODEL_IDS,].toSorted(),);
        for (const modelId of ROSTER_MODEL_IDS)
          expect(completionCapFor({ modelId, },),).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'LOWERS to a caller\'s own ceiling and never raises to one, since the cap is the bound that '
        + 'holds where a cancel does not',
      fn: async () => {
        /**
         Measured cap for one seat.
         */
        const cap = COMPLETION_CAP[SEAT_HYPER_OPENROUTER_UNMEASURED];
        expect(completionCapFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, requested: cap - 1, },),).toBe(cap - 1,);
        expect(completionCapFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, requested: cap + 1, },),).toBe(cap,);
      },
    },),
    it({
      name: 'RESOLVES a card naming a pooled percentile to that one pooled figure, the 99th above the 90th, so a '
        + 'model with no calls of its own is capped like every other such model',
      fn: async () => {
        /**
         Caps of the cards naming each pool.
         */
        const p90 = capsNaming({ pool: 'pooled-p90', },);
        const p99 = capsNaming({ pool: 'pooled-p99', },);
        expect(p90,).toHaveLength(1,);
        expect(p99,).toHaveLength(1,);
        expect(p99[0],).toBeGreaterThan(p90[0] ?? Number.POSITIVE_INFINITY,);
      },
    },),
  ],
},);
