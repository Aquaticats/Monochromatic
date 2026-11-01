/**
 Tests the two readings of which providers may take a call (`provider-router-reach.ts`), over the holds a router keeps.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { SEAT_BEDROCK_ONLY_TEXT, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  createCredentialHolds,
  createReachReaders,
} from '../dist/final/node/index.mjs';

/**
 A call with no picture, addressed to a model only Bedrock serves.
 */
const REQUEST = {
  modelId: SEAT_BEDROCK_ONLY_TEXT,
  messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
  signal: new AbortController().signal,
  perCallTimeoutMs: 1_000,
} as const;

/**
 Stream bound holds that hold nothing out.
 */
const NO_BOUND_HOLDS = {
  hold: function holdNothing(): void {
    return undefined;
  },
  remainingMs: function remainsNothing(): number {
    return 0;
  },
};

await describe({
  name: createReachReaders.name,
  children: [
    it({
      name: 'READS THE PROVIDERS THAT SERVE THE CALL, and takes out the one whose credential was refused',
      fn: async () => {
        const credentialHolds = createCredentialHolds();
        const { reachFor, } = createReachReaders({
          credentialHolds,
          boundHolds: NO_BOUND_HOLDS,
        },);
        expect(reachFor({ request: REQUEST, },).bedrock,).toBe(true,);
        credentialHolds.hold({ provider: 'bedrock', },);
        expect(reachFor({ request: REQUEST, },).bedrock,).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES WITHOUT A CALL, unmarked as raised after an exchange, when the holds leave nobody',
      fn: async () => {
        const credentialHolds = createCredentialHolds();
        credentialHolds.hold({ provider: 'bedrock', },);
        const { reachPastBoundHoldsOrRefusal, } = createReachReaders({
          credentialHolds,
          boundHolds: NO_BOUND_HOLDS,
        },);
        try {
          reachPastBoundHoldsOrRefusal({ request: REQUEST, },);
        }
        catch (error) {
          expect(error instanceof NoProviderForModelError ? error.afterExchange : 'not a refusal',).toBe(false,);
          return;
        }
        throw new Error('the holds leave nobody and the reading must refuse',);
      },
    },),
    it({
      name: 'MARKS THE REFUSAL AS RAISED AFTER AN EXCHANGE when the router asks it after a call was cut (B1230)',
      fn: async () => {
        const credentialHolds = createCredentialHolds();
        credentialHolds.hold({ provider: 'bedrock', },);
        const { reachPastBoundHoldsOrRefusal, } = createReachReaders({
          credentialHolds,
          boundHolds: NO_BOUND_HOLDS,
        },);
        try {
          reachPastBoundHoldsOrRefusal({
            request: REQUEST,
            afterExchange: true,
          },);
        }
        catch (error) {
          expect(error instanceof NoProviderForModelError ? error.afterExchange : 'not a refusal',).toBe(true,);
          return;
        }
        throw new Error('the holds leave nobody and the reading must refuse',);
      },
    },),
  ],
},);
