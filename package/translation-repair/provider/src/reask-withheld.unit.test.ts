/**
 Tests for the one question the backfill of the writing benches and the re-ask of a runaway both ask: is this model
 withheld from this role in this reading of the meters (ledger B1330).

 A MODEL IS WITHHELD BY A ROUTE-TIME HOLD ONLY WHILE HYPER IS THE PROVIDER THAT WOULD SERVE IT: Qwen3.8-27B carries
 `hyper-slow` and Kimi-K3 carries `hyper-slow-select`, and a reading in which Synthetic serves either seats it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readingWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import {
  isWithheldFromRole,
  routeTimeHoldsInReading,
} from '../dist/final/node/index.mjs';

//region Withheld from a role tests

await describe({
  name: routeTimeHoldsInReading.name,
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
  ],
},);

//endregion Withheld from a role tests
