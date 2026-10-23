/**
 Tests for the router's slot ledger: only a provider with a per-model limit
 is ever saturated, and a take is paired with a release on scope exit.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createSlotLedger,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Model the cases count slots for.
 */
const KIMI = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 One Synthetic slot per model, no ceiling elsewhere.
 */
const ONE_SYNTHETIC_SLOT = {
  neuralwatt: Number.POSITIVE_INFINITY,
  synthetic: 1,
  hyper: Number.POSITIVE_INFINITY,
  bedrock: Number.POSITIVE_INFINITY,
  deepseek: Number.POSITIVE_INFINITY,
  xiaomi: Number.POSITIVE_INFINITY,
  anthropic: Number.POSITIVE_INFINITY,
  openrouter: Number.POSITIVE_INFINITY,
};

await describe({
  name: createSlotLedger.name,
  children: [
    it({
      name: 'SATURATES a limiting provider at its limit and never a provider without one',
      fn: async () => {
        /**
         Ledger granting Synthetic one slot per model and the others none.
         */
        const ledger = createSlotLedger({ limits: ONE_SYNTHETIC_SLOT, },);
        expect(ledger.saturated({ modelId: KIMI, },),).toEqual({
          neuralwatt: false,
          synthetic: false,
          hyper: false,
          bedrock: false,
          deepseek: false,
          xiaomi: false,
          anthropic: false,
          openrouter: false,
        },);
        ledger.take({
          provider: 'synthetic',
          modelId: KIMI,
        },);
        ledger.take({
          provider: 'hyper',
          modelId: KIMI,
        },);
        ledger.take({
          provider: 'openrouter',
          modelId: KIMI,
        },);
        expect(ledger.saturated({ modelId: KIMI, },),).toEqual({
          neuralwatt: false,
          synthetic: true,
          hyper: false,
          bedrock: false,
          deepseek: false,
          xiaomi: false,
          anthropic: false,
          openrouter: false,
        },);
        expect(ledger.limits({ provider: 'synthetic', },),).toBe(true,);
        expect(ledger.limits({ provider: 'openrouter', },),).toBe(false,);
      },
    },),

    it({
      name: 'RELEASES a held slot on scope exit and counts per model, so one model\'s saturation '
        + 'does not spill onto another',
      fn: async () => {
        /**
         Ledger granting Synthetic one slot per model.
         */
        const ledger = createSlotLedger({ limits: ONE_SYNTHETIC_SLOT, },);
        ledger.take({
          provider: 'synthetic',
          modelId: KIMI,
        },);
        expect(ledger.saturated({ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },).synthetic,).toBe(false,);
        {
          using held = ledger.held({
            provider: 'synthetic',
            modelId: KIMI,
          },);
          void held;
          expect(ledger.saturated({ modelId: KIMI, },).synthetic,).toBe(true,);
        }
        expect(ledger.saturated({ modelId: KIMI, },).synthetic,).toBe(false,);
      },
    },),
  ],
},);
