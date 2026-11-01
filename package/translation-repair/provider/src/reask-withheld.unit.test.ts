/**
 Tests for the one question the backfill of the writing benches and the re-ask of a runaway both ask: is this model
 withheld from this role in this reading of the meters (ledger B1330).

 A MODEL IS WITHHELD BY A ROUTE-TIME HOLD ONLY WHILE HYPER IS THE PROVIDER THAT WOULD SERVE IT: Qwen3.8-27B carries
 `hyper-slow` and Kimi-K3 carries `hyper-slow-select`, and a reading in which Synthetic serves either seats it.

 The router asks a second question of the same card, which of those holds keep one request off the provider it means
 to send it to (`holdsKeepingRequestOff`, ledger B1365): none for a measurement, and for a request that names no seat
 what the caller's rule says.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { readingWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import {
  type ChatTextRequest,
  holdsKeepingRequestOff,
  isWithheldFromRole,
  type ReaskSeat,
  routeTimeHoldsInReading,
} from '../dist/final/node/index.mjs';

//region Withheld from a role tests

/**
 Request for a model, naming the seat when one is given, as a router reads it.

 @param modelId - model asked

 @param reaskSeat - seat the request names, when it names one

 @returns Request carrying nothing the hold reads besides the model and the seat

 @example
 ```ts
 const reaskSeat: ReaskSeat = { role: 'editor', bench: [], reask: 'other-model', };
 const request = requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', reaskSeat, },);
 ```
 */
function requestFor(
  { modelId, reaskSeat, }: { readonly modelId: RosterModelId; readonly reaskSeat?: ReaskSeat; },
): ChatTextRequest {
  return {
    modelId,
    messages: [{ role: 'user', content: '这只猫睡在哪里？', },],
    signal: new AbortController().signal,
    ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
  };
}

await describe({
  name: 'the holds of a card that keep a model out of a role',
  children: [
    it({
      name: 'WITHHOLDS QWEN3.8-27B FROM EVERY ROLE WHILE HYPER WOULD SERVE IT AND FROM NONE WHILE SYNTHETIC DOES',
      fn: async () => {
        for (const role of ['editor', 'refiner', 'roster', 'writer', 'translator',] as const) {
          expect(routeTimeHoldsInReading({
            modelId: 'hf:Qwen/Qwen3.8-27B',
            role,
            dry: readingWith({ wet: ['hyper',], },),
          },),).toEqual(['hyper-slow',],);
          expect(routeTimeHoldsInReading({
            modelId: 'hf:Qwen/Qwen3.8-27B',
            role,
            dry: readingWith({ wet: ['synthetic',], },),
          },),).toEqual([],);
        }
      },
    },),

    it({
      name: 'WITHHOLDS KIMI-K3 FROM THE SELECT AND SLATE SEATS ALONE, and only while Hyper would serve it',
      fn: async () => {
        for (const role of ['select-judge', 'slate-judge',] as const) {
          expect(isWithheldFromRole({
            modelId: 'hf:moonshotai/Kimi-K3',
            role,
            dry: readingWith({ wet: ['hyper',], },),
          },),).toBe(true,);
          expect(isWithheldFromRole({
            modelId: 'hf:moonshotai/Kimi-K3',
            role,
            dry: readingWith({ wet: ['synthetic', 'hyper',], },),
          },),).toBe(false,);
        }
        for (const role of ['editor', 'refiner', 'wide-judge', 'late-judge',] as const) {
          expect(isWithheldFromRole({
            modelId: 'hf:moonshotai/Kimi-K3',
            role,
            dry: readingWith({ wet: ['hyper',], },),
          },),).toBe(false,);
        }
      },
    },),

    it({
      name: 'WITHHOLDS NO MODEL WHOSE CARD CARRIES NO ROUTE-TIME HOLD, in any reading',
      fn: async () => {
        for (const wet of [['hyper',], ['synthetic',], ['hyper', 'synthetic',], [],] as const) {
          expect(isWithheldFromRole({
            modelId: 'minimax-m3',
            role: 'editor',
            dry: readingWith({ wet, },),
          },),).toBe(false,);
        }
      },
    },),

    it({
      name: 'HOLDS A SEAT OF A ROLE THAT HONOURS THE HOLD OFF HYPER and off no other provider, whatever the rule for a '
        + 'request that names no seat (ledger B1365)',
      fn: async () => {
        /**
         The seat of an editor of a run.
         */
        const reaskSeat: ReaskSeat = { role: 'editor', bench: [], reask: 'other-model', };
        for (const whenNoSeat of ['hold', 'serve',] as const) {
          expect(holdsKeepingRequestOff({
            request: requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', reaskSeat, },),
            provider: 'hyper',
            whenNoSeat,
          },),).toEqual(['hyper-slow',],);
          expect(holdsKeepingRequestOff({
            request: requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', reaskSeat, },),
            provider: 'synthetic',
            whenNoSeat,
          },),).toEqual([],);
        }
      },
    },),

    it({
      name: 'HOLDS A REQUEST THAT NAMES NO SEAT TO THE HOLDS EVERY ROLE HONOURS WHERE THE RULE SAYS HOLD, and serves it '
        + 'where the rule says serve (ledger B1365, B1350)',
      fn: async () => {
        /**
         Request for Qwen3.8-27B that names no seat.
         */
        const request = requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
        expect(holdsKeepingRequestOff({ request, provider: 'hyper', whenNoSeat: 'hold', },),).toEqual(['hyper-slow',],);
        expect(holdsKeepingRequestOff({ request, provider: 'hyper', whenNoSeat: 'serve', },),).toEqual([],);
      },
    },),

    it({
      name: 'HOLDS NO MEASUREMENT whatever the rule and the provider (ledger B1141, B1365)',
      fn: async () => {
        for (const whenNoSeat of ['hold', 'serve',] as const) {
          expect(holdsKeepingRequestOff({
            request: requestFor({
              modelId: 'hf:Qwen/Qwen3.8-27B',
              reaskSeat: { role: 'editor', bench: [], reask: 'measured-model', },
            },),
            provider: 'hyper',
            whenNoSeat,
          },),).toEqual([],);
        }
      },
    },),

    it({
      name: 'HOLDS NOTHING OFF HYPER FOR A ROLE THE HOLD DOES NOT REACH: Kimi-K3 is held in the select seats alone',
      fn: async () => {
        expect(holdsKeepingRequestOff({
          request: requestFor({
            modelId: 'hf:moonshotai/Kimi-K3',
            reaskSeat: { role: 'select-judge', bench: [], reask: 'other-model', },
          },),
          provider: 'hyper',
          whenNoSeat: 'serve',
        },),).toEqual(['hyper-slow-select',],);
        expect(holdsKeepingRequestOff({
          request: requestFor({
            modelId: 'hf:moonshotai/Kimi-K3',
            reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
          },),
          provider: 'hyper',
          whenNoSeat: 'serve',
        },),).toEqual([],);
      },
    },),
  ],
},);

//endregion Withheld from a role tests
