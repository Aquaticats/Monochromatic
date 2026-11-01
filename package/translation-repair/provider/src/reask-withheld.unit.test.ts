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

import { readingWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import {
  holdsKeepingRequestOff,
  isWithheldFromRole,
  isWithheldUnderRule,
  type ReaskSeat,
  routeTimeHoldsInReading,
  routeTimeHoldsOnProvider,
} from '../dist/final/node/index.mjs';
import { requestFor, } from './seated-request.test-fixture.ts';

//region Withheld from a role tests

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
      name: 'NAMES THE HOLDS OF A CARD ON THE PROVIDER THEY NAME ALONE (ledger B1377): Hyper for Qwen3.8-27B in any role '
        + 'and for Kimi-K3 in the select seats, no other provider, and the roster-wide role where none is given',
      fn: async () => {
        expect(routeTimeHoldsOnProvider({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'editor', provider: 'hyper', },),)
          .toEqual(['hyper-slow',],);
        expect(routeTimeHoldsOnProvider({ modelId: 'hf:Qwen/Qwen3.8-27B', provider: 'hyper', },),).toEqual(['hyper-slow',],);
        expect(routeTimeHoldsOnProvider({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'editor', provider: 'synthetic', },),)
          .toEqual([],);
        expect(routeTimeHoldsOnProvider({ modelId: 'hf:moonshotai/Kimi-K3', role: 'select-judge', provider: 'hyper', },),)
          .toEqual(['hyper-slow-select',],);
        expect(routeTimeHoldsOnProvider({ modelId: 'hf:moonshotai/Kimi-K3', role: 'editor', provider: 'hyper', },),)
          .toEqual([],);
        expect(routeTimeHoldsOnProvider({ modelId: 'minimax-m3', role: 'editor', provider: 'hyper', },),).toEqual([],);
      },
    },),

    it({
      name: 'WITHHOLDS UNDER A RULE THE WAY THE RE-ASK READS ONE (ledger B1372): the rule that withholds every held model '
        + 'does so whatever the meters say, the rule of a reading only while Hyper would serve the model, and the rule '
        + 'of a caller that applies the reading itself withholds none',
      fn: async () => {
        /**
         Models and roles under question: Qwen3.8-27B in an editor seat, which the hold reaches, and a model whose card
         carries no route-time hold.
         */
        const held = { modelId: 'hf:Qwen/Qwen3.8-27B', role: 'editor', } as const;
        const unheld = { modelId: 'minimax-m3', role: 'editor', } as const;
        expect(isWithheldUnderRule({ ...held, rule: { applied: 'always', }, },),).toBe(true,);
        expect(isWithheldUnderRule({ ...unheld, rule: { applied: 'always', }, },),).toBe(false,);
        expect(isWithheldUnderRule({
          ...held,
          rule: { applied: 'in-reading', dry: readingWith({ wet: ['hyper',], },), },
        },),).toBe(true,);
        expect(isWithheldUnderRule({
          ...held,
          rule: { applied: 'in-reading', dry: readingWith({ wet: ['synthetic',], },), },
        },),).toBe(false,);
        expect(isWithheldUnderRule({ ...held, rule: { applied: 'by-the-caller', }, },),).toBe(false,);
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
      name: 'HOLDS A SEAT OF A ROLE THAT HONOURS THE HOLD OFF HYPER and off no other provider (ledger B1365)',
      fn: async () => {
        /**
         The seat of an editor of a run.
         */
        const reaskSeat: ReaskSeat = { role: 'editor', bench: [], reask: 'other-model', };
        expect(holdsKeepingRequestOff({
          request: requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', reaskSeat, },),
          provider: 'hyper',
        },),).toEqual(['hyper-slow',],);
        expect(holdsKeepingRequestOff({
          request: requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', reaskSeat, },),
          provider: 'synthetic',
        },),).toEqual([],);
      },
    },),

    it({
      name: 'HOLDS A REQUEST THAT NAMES NO SEAT TO THE HOLDS EVERY ROLE HONOURS (ledger B1373, B1350): every production '
        + 'caller names its seat or its measurement, so a request that names none is held on the first ask as on the re-ask',
      fn: async () => {
        /**
         Request for Qwen3.8-27B that names no seat.
         */
        const request = requestFor({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
        expect(holdsKeepingRequestOff({ request, provider: 'hyper', },),).toEqual(['hyper-slow',],);
        expect(holdsKeepingRequestOff({ request, provider: 'synthetic', },),).toEqual([],);
      },
    },),

    it({
      name: 'HOLDS NO MEASUREMENT whatever the provider (ledger B1141, B1365)',
      fn: async () => {
        expect(holdsKeepingRequestOff({
          request: requestFor({
            modelId: 'hf:Qwen/Qwen3.8-27B',
            reaskSeat: { role: 'editor', bench: [], reask: 'measured-model', },
          },),
          provider: 'hyper',
        },),).toEqual([],);
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
        },),).toEqual(['hyper-slow-select',],);
        expect(holdsKeepingRequestOff({
          request: requestFor({
            modelId: 'hf:moonshotai/Kimi-K3',
            reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
          },),
          provider: 'hyper',
        },),).toEqual([],);
      },
    },),
  ],
},);

//endregion Withheld from a role tests
