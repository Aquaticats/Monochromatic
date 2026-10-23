/**
 Tests for the refusal a seat reading stops the entry with: the writing-bench error, or the router's every-provider-dry
 refusal when every provider reads dry.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type BudgetView,
  EveryProviderDryError,
} from '@monochromatic-dev/translation-repair-provider';
import { failureWritten, } from '@monochromatic-dev/translation-repair-stage';
import {
  benchesOf,
  judgeSeatsFor,
  RUN_WRITING_PROVIDERS,
  WRITING_BENCHES,
  WritingBenchUnreachableError,
  writingProvidersOf,
  writtenRefusal,
} from '../../dist/final/node/index.mjs';

/**
 Every provider dry.
 */
const ALL_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
};

/**
 Bedrock alone wet.
 */
const BEDROCK_WET: BudgetView = { ...ALL_DRY, bedrock: false, };

/**
 No provider held out.
 */
const NO_HOLDS = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  hyper: 0,
  openrouter: 0,
};

await describe({
  name: writtenRefusal.name,
  children: [
    it({
      name: 'BUILDS THE EVERY-PROVIDER-DRY REFUSAL when every provider reads dry, stating the reading, the holds and '
        + 'the NeuralWatt day start the client reports, and marks it written',
      fn: async () => {
        /**
         The refusal for a lanes reading on a fully dry view.
         */
        const refusal = writtenRefusal({
          phase: 'lanes',
          clauses: ['editors 0 of 3 reachable, floor 2',],
          dry: ALL_DRY,
          client: {
            providerHolds: () => NO_HOLDS,
            neuralwattReturn: () => ({ kind: 'at', msOfDay: 69_308_000, }),
          },
        },);
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
        expect(refusal.message,).toContain('the seat reading for lanes read neuralwatt dry, synthetic dry',);
        expect(refusal.message,).toContain('holds neuralwatt 0ms',);
        expect(refusal.message,).toContain('at 19:15:08 UTC',);
        expect(failureWritten({ error: refusal, },),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES THE NEURALWATT DAY UNDECIDED when the client reports none, rather than inventing one',
      fn: async () => {
        /**
         The refusal for a client with no NeuralWatt accessor.
         */
        const refusal = writtenRefusal({
          phase: 'lanes',
          clauses: [],
          dry: ALL_DRY,
          client: { providerHolds: () => NO_HOLDS, },
        },);
        expect(refusal.message,).toContain('no time of day can be stated',);
      },
    },),

    it({
      name: 'COUNTS A WET PROVIDER ONLY IF IT SERVES A WRITING SEAT: Xiaomi wet and serving only a checker seat leaves '
        + 'every provider dry in the refusal, stating Xiaomi wet beside its reason, and a client that puts a writer '
        + 'seat on Xiaomi gets the bench error instead',
      fn: async () => {
        /**
         Every provider dry but Xiaomi.
         */
        const xiaomiWet: BudgetView = { ...ALL_DRY, xiaomi: false, };
        /**
         The refusal on the roster's own writing providers.
         */
        const onRoster = writtenRefusal({
          phase: 'lanes',
          clauses: ['editors 0 of 3 reachable, floor 2',],
          dry: xiaomiWet,
          client: { providerHolds: () => NO_HOLDS, },
        },);
        expect(onRoster,).toBeInstanceOf(EveryProviderDryError,);
        expect(onRoster.message,).toContain('xiaomi wet (serves no seat that writes)',);
        expect(failureWritten({ error: onRoster, },),).toBe(true,);
        /**
         The refusal when a writer seat sits on Xiaomi.
         */
        const withWriter = writtenRefusal({
          phase: 'lanes',
          clauses: ['editors 0 of 3 reachable, floor 2',],
          dry: xiaomiWet,
          client: { providerHolds: () => NO_HOLDS, writingProviders: ALL_DRY, },
        },);
        expect(withWriter,).toBeInstanceOf(WritingBenchUnreachableError,);
      },
    },),

    it({
      name: 'DERIVES THE WRITING PROVIDERS FROM THE ROSTER\'S WRITING BENCHES, the benches the floor holds to a pair, '
        + 'and not from a list of provider names: every provider but Xiaomi, whose one seat is a checker',
      fn: async () => {
        /**
         The seats of the writing benches with every provider wet.
         */
        const benches = benchesOf({
          seats: judgeSeatsFor({
            dry: {
              neuralwatt: false,
              synthetic: false,
              bedrock: false,
              deepseek: false,
              xiaomi: false,
              hyper: false,
              openrouter: false,
            },
          },),
        },);
        /**
         Models seated in a writing bench.
         */
        const writing = [...WRITING_BENCHES,].flatMap(function seatsOf(name,) {
          return benches[name];
        },);
        expect(writingProvidersOf({ modelIds: writing, },),).toEqual(RUN_WRITING_PROVIDERS,);
        expect(RUN_WRITING_PROVIDERS,).toEqual({
          neuralwatt: true,
          synthetic: true,
          bedrock: true,
          deepseek: true,
          xiaomi: false,
          hyper: true,
          openrouter: true,
        },);
        expect(writingProvidersOf({ modelIds: ['xiaomi/mimo-v2.6-flash',], },),).toEqual({
          neuralwatt: false,
          synthetic: false,
          bedrock: false,
          deepseek: false,
          xiaomi: true,
          hyper: false,
          openrouter: true,
        },);
      },
    },),

    it({
      name: 'BUILDS THE WRITING-BENCH ERROR when any provider reads wet, with the clauses, marked written',
      fn: async () => {
        /**
         The refusal for a lanes reading with Bedrock still wet.
         */
        const refusal = writtenRefusal({
          phase: 'lanes',
          clauses: ['editors 0 of 3 reachable, floor 2',],
          dry: BEDROCK_WET,
          client: { providerHolds: () => NO_HOLDS, },
        },);
        expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
        expect(refusal.message,).toContain('editors 0 of 3 reachable, floor 2',);
        expect(failureWritten({ error: refusal, },),).toBe(true,);
      },
    },),
  ],
},);
