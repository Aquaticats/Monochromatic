/**
 Tests for marking the router's no-provider refusal as raised after an exchange (B1230).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SEAT_BEDROCK_ONLY_TEXT, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { markingAfterExchange, } from '../dist/final/node/index.mjs';

/**
 A call with no picture, addressed to a model only Bedrock serves.
 */
const REQUEST = {
  modelId: SEAT_BEDROCK_ONLY_TEXT,
  messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
  signal: new AbortController().signal,
  perCallTimeoutMs: HANG_STOP_MS,
} as const;

/**
 A refusal raised before any call.
 */
function beforeAnyCall(): NoProviderForModelError {
  return new NoProviderForModelError({ modelId: 'glm-5.3', reason: 'its credential was refused this run', },);
}

/**
 A decision that finds no provider for the call.

 @returns Nothing; it always raises

 @throws {@link NoProviderForModelError} always
 */
async function refusing(): Promise<'hyper'> {
  throw beforeAnyCall();
}

/**
 A decision that picks Hyper.

 @returns Hyper
 */
async function answering(): Promise<'hyper'> {
  return await Promise.resolve('hyper',);
}

/**
 What a decision raised when run through the mark.

 @param exchanged - whether a call of the request was made first

 @param choose - decision to run

 @returns Error it raised, or what it answered

 @example
 ```ts
 const raised = await raisedBy({ exchanged: true, choose: refusing, },);
 ```
 */
async function raisedBy(
  {
    exchanged,
    choose,
  }: {
    readonly exchanged: boolean;
    readonly choose: typeof refusing;
  },
): Promise<unknown> {
  try {
    return await markingAfterExchange({
      exchanged,
      decision: choose,
      request: REQUEST,
      refused: 'bedrock',
    },);
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: 'the exchange mark',
  children: [
    it({
      name: 'MARKS A NO-PROVIDER REFUSAL RAISED AFTER A CALL, keeping its model and its reason, and returns an answer '
        + 'as it came',
      fn: async () => {
        const refusal = await raisedBy({ exchanged: true, choose: refusing, },);
        expect(refusal instanceof NoProviderForModelError ? refusal.afterExchange : 'not a refusal',).toBe(true,);
        expect(String(refusal,),).toBe(
          'NoProviderForModelError: no provider can take glm-5.3: its credential was refused this run',
        );
        expect(await raisedBy({ exchanged: true, choose: answering, },),).toBe('hyper',);
      },
    },),
    it({
      name: 'LEAVES A REFUSAL RAISED BEFORE ANY CALL UNMARKED, and any other error as it is',
      fn: async () => {
        const refusal = await raisedBy({ exchanged: false, choose: refusing, },);
        expect(refusal instanceof NoProviderForModelError ? refusal.afterExchange : 'not a refusal',).toBe(false,);
        const other = new Error('transport',);
        expect(await raisedBy({
          exchanged: true,
          choose: async function failing(): Promise<'hyper'> {
            throw other;
          },
        },),).toBe(other,);
      },
    },),
  ],
},);
