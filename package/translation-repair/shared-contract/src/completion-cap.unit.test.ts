/**
 Tests for the completion cap every client sends as `max_tokens`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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
      name: 'RESOLVES every card naming the pooled 90th to that one floor, and finds no card on the pooled 99th, '
        + 'since a seat the cap census reads calls for carries its own measured cap (ledger B990)',
      fn: async () => {
        /**
         Caps of the cards naming each pool.
         */
        const p90 = capsNaming({ pool: 'pooled-p90', },);
        const p99 = capsNaming({ pool: 'pooled-p99', },);
        expect(p90,).toHaveLength(1,);
        expect(p99,).toEqual([],);
      },
    },),
    it({
      name: 'CARRIES THE MEASURED CAP of the Xiaomi checker seat, the census rule\'s reading of 2026-10-29 over 293 '
        + 'Xiaomi calls and 1,362 OpenRouter calls, and not the pooled 99th it rode before',
      fn: async () => {
        expect(MODEL_CARDS['xiaomi/mimo-v2.6-flash'].completionCap,).toBe(5_098,);
        expect(COMPLETION_CAP['xiaomi/mimo-v2.6-flash'],).toBe(5_098,);
      },
    },),
  ],
},);
