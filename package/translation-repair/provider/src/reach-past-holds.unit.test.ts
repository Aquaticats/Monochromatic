/**
 Tests the reach a call is routed over once every hold the router keeps is
 read: a model whose every serving provider had its credential refused, or is
 held out on its stream bound, is refused without a call, and a model with a
 provider left keeps it. Cat-themed invention throughout; no corpus content
 appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_TEXT_BEDROCK,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  createCredentialHolds,
  reachPastHoldsOrRefusal,
} from '../dist/final/node/index.mjs';

/**
 Reach of a model Bedrock, Hyper and OpenRouter serve.
 */
const BEDROCK_HYPER_OPENROUTER = {
  neuralwatt: false,
  synthetic: false,
  hyper: true,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: true,
};

/**
 Reach of a model only Bedrock serves.
 */
const BEDROCK_ONLY = {
  neuralwatt: false,
  synthetic: false,
  hyper: false,
  bedrock: true,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
};

/**
 Reach of a model nobody serves.
 */
const NOBODY = {
  neuralwatt: false,
  synthetic: false,
  hyper: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  openrouter: false,
};

/**
 Stream bound holds that hold the named providers out for one model.

 @param held - providers held out for every model asked

 @returns Holds of the shape the router keeps

 @example
 ```ts
 const boundHolds = boundHoldsOn({ held: ['hyper',], },);
 ```
 */
function boundHoldsOn({ held, }: { readonly held: readonly ProviderName[]; },): {
  readonly hold: () => void;
  readonly remainingMs: (args: { readonly provider: ProviderName; readonly modelId: string; },) => number;
} {
  return {
    hold: function holdNothing(): void {
      return undefined;
    },
    remainingMs: function remainingOn({ provider, }: { readonly provider: ProviderName; },): number {
      return held.includes(provider,) ? 1_000 : 0;
    },
  };
}

await describe({
  name: reachPastHoldsOrRefusal.name,
  children: [
    it({
      name: 'KEEPS EVERY SERVING PROVIDER where nothing is held, and a model nobody serves is left for the policy to refuse',
      fn: async () => {
        expect(reachPastHoldsOrRefusal({
          reach: BEDROCK_HYPER_OPENROUTER,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          credentialHolds: createCredentialHolds(),
          boundHolds: boundHoldsOn({ held: [], },),
        },),).toEqual(BEDROCK_HYPER_OPENROUTER,);
        expect(reachPastHoldsOrRefusal({
          reach: NOBODY,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          credentialHolds: createCredentialHolds(),
          boundHolds: boundHoldsOn({ held: [], },),
        },),).toEqual(NOBODY,);
      },
    },),
    it({
      name: 'TAKES OUT A PROVIDER WHOSE CREDENTIAL WAS REFUSED and one held on the model\'s stream bound, keeping the rest',
      fn: async () => {
        const credentialHolds = createCredentialHolds();
        credentialHolds.hold({ provider: 'bedrock', },);

        expect(reachPastHoldsOrRefusal({
          reach: BEDROCK_HYPER_OPENROUTER,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
          credentialHolds,
          boundHolds: boundHoldsOn({ held: ['hyper',], },),
        },),).toEqual({
          ...NOBODY,
          openrouter: true,
        },);
      },
    },),
    it({
      name: 'REFUSES A MODEL ONLY THE REFUSED PROVIDER SERVES, naming the credential and the provider',
      fn: async () => {
        const credentialHolds = createCredentialHolds();
        credentialHolds.hold({ provider: 'bedrock', },);

        expect(function ask(): unknown {
          return reachPastHoldsOrRefusal({
            reach: BEDROCK_ONLY,
            modelId: SEAT_BEDROCK_ONLY_TEXT,
            credentialHolds,
            boundHolds: boundHoldsOn({ held: [], },),
          },);
        },).toThrow(
          `no provider can take ${SEAT_BEDROCK_ONLY_TEXT}: `
            + 'every provider serving this model had its credential refused this run (bedrock)',
        );
      },
    },),
    it({
      name: 'REFUSES A MODEL EVERY SERVING PROVIDER OF WHICH IS HELD ON ITS STREAM BOUND, naming the bound and the wait',
      fn: async () => {
        expect(function ask(): unknown {
          return reachPastHoldsOrRefusal({
            reach: BEDROCK_ONLY,
            modelId: SEAT_BEDROCK_ONLY_TEXT,
            credentialHolds: createCredentialHolds(),
            boundHolds: boundHoldsOn({ held: ['bedrock',], },),
          },);
        },).toThrow(
          `no provider can take ${SEAT_BEDROCK_ONLY_TEXT}: `
            + 'every provider serving this model ran past its stream bound; held out for another 1000ms',
        );
      },
    },),
  ],
},);
