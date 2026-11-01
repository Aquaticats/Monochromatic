/**
 Tests for the in-run memo of slices asking the same question.

 WHAT THESE PIN: a slice with no twin buying its key buys and registers; a
 twin arriving during that buy waits and reuses what was stored; a twin
 arriving during a buy that stored nothing asks for itself; a third twin
 waits for the second rather than buying beside it; a failed buy withdraws
 its entry, warns, and leaves the waiting twin to ask for itself; a buy
 ending in the caller's own reason withdraws its entry and writes nothing,
 while a buy's own failure under the caller's stop keeps its line; and a
 different key never waits.

 Every purchase finishes when the test opens its gate, so which twin is
 still buying when another arrives is the test's to choose.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  reuseTwinOrBuy,
  type TwinMemo,
  type TwinOrBought,
  type TwinStored,
} from '../dist/final/node/index.mjs';
import { quotingFailure, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';

//region Fixtures
// A buyer whose purchases finish when the test opens their gates, in the
// order the purchases were started.

/**
 What the fixture's buyer returns.
 */
type Purchase = {
  readonly record: string;
  readonly persisted: boolean;
};

/**
 One buyer and what it records.
 */
type Buyer = {
  /**
   One gate per purchase started, in start order.
   */
  readonly gates: readonly PromiseWithResolvers<Purchase>[];

  /**
   Starts a purchase that finishes at its gate.
   */
  readonly buy: () => Promise<Purchase>;

  /**
   Every line the memo warned.
   */
  readonly warned: readonly string[];

  /**
   Logger whose warnings land in `warned`.
   */
  readonly l: Logger;
};

/**
 Builds a gated buyer.

 @returns Buyer with empty records

 @example
 ```ts
 const shop = buyer();
 ```
 */
function buyer(): Buyer {
  /**
   Gates in start order.
   */
  const gates: PromiseWithResolvers<Purchase>[] = [];

  /**
   Warnings in the order they were logged.
   */
  const warned: string[] = [];
  return {
    gates,
    warned,
    buy: async function gated(): Promise<Purchase> {
      /**
       Gate this purchase finishes at.
       */
      const gate = Promise.withResolvers<Purchase>();
      gates.push(gate,);
      return await gate.promise;
    },
    l: {
      ...tagged({ tag: 'twin-memo-test', },),
      warn: function record(message,): void {
        warned.push(message,);
      },
    },
  };
}

/**
 Reads what a purchase left for its twins.

 @param bought - purchase

 @returns Record when stored

 @example
 ```ts
 persistedOf({ record: 'r', persisted: true, },);
 ```
 */
function persistedOf(bought: Purchase,): TwinStored<string> {
  return bought.persisted
    ? {
      kind: 'stored',
      record: bought.record,
    }
    : { kind: 'nothing', };
}

/**
 Caller's signal for the cases where nobody stops the work.
 */
const NEVER_STOPPED = new AbortController().signal;

/**
 Asks under the memo with the fixture's buyer.

 @param memo - shared memo

 @param shop - buyer

 @param key - question asked

 @param signal - caller's stop, never stopped unless a case steers it

 @returns What came of asking

 @example
 ```ts
 const first = asking({ memo, shop, },);
 ```
 */
async function asking(
  {
    memo,
    shop,
    key = 'shared',
    signal = NEVER_STOPPED,
  }: {
    readonly memo: TwinMemo<string>;
    readonly shop: Buyer;
    readonly key?: string;
    readonly signal?: AbortSignal;
  },
): Promise<TwinOrBought<string, Purchase>> {
  return await reuseTwinOrBuy({
    key,
    memo,
    buy: shop.buy,
    persistedOf,
    l: shop.l,
    signal,
  },);
}

/**
 Lets every settled continuation run.

 @example
 ```ts
 await settle();
 ```
 */
async function settle(): Promise<void> {
  await wait(0,);
}

/**
 Resolves with what a run threw, or `undefined` when it finished.

 @param run - promise under test

 @returns What it threw

 @example
 ```ts
 const failure = collected({ run, },);
 ```
 */
async function collected({ run, }: { readonly run: Promise<unknown>; },): Promise<unknown> {
  try {
    await run;
    return undefined;
  }
  catch (error) {
    return error;
  }
}

//endregion Fixtures

await describe({
  name: reuseTwinOrBuy.name,
  children: [
    it({
      name: 'buys when nobody is buying the key, and registers the buy under it',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = asking({
          memo,
          shop,
        },);
        await settle();
        expect(shop.gates,).toHaveLength(1,);
        expect(memo.has('shared',),).toBe(true,);

        nonNullishOrThrow(shop.gates[0],).resolve({
          record: 'r1',
          persisted: true,
        },);
        expect(await first,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r1',
            persisted: true,
          },
        },);
        expect(
          await nonNullishOrThrow(memo.get('shared',),),
        ).toEqual({
          kind: 'stored',
          record: 'r1',
        },);
      },
    },),

    it({
      name: 'a twin arriving during the buy waits for it and reuses what it stored',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = asking({
          memo,
          shop,
        },);
        await settle();
        const second = asking({
          memo,
          shop,
        },);
        await settle();
        expect(shop.gates,).toHaveLength(1,);

        nonNullishOrThrow(shop.gates[0],).resolve({
          record: 'r1',
          persisted: true,
        },);
        expect(await first,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r1',
            persisted: true,
          },
        },);
        expect(await second,).toEqual({
          kind: 'reused',
          twin: 'r1',
        },);
        expect(shop.gates,).toHaveLength(1,);
      },
    },),

    it({
      name: 'a twin arriving during a buy that stored nothing asks for itself, exactly as a warm run would',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = asking({
          memo,
          shop,
        },);
        await settle();
        const second = asking({
          memo,
          shop,
        },);
        await settle();

        nonNullishOrThrow(shop.gates[0],).resolve({
          record: 'unheard',
          persisted: false,
        },);
        expect(await first,).toEqual({
          kind: 'bought',
          bought: {
            record: 'unheard',
            persisted: false,
          },
        },);
        await settle();
        expect(shop.gates,).toHaveLength(2,);

        nonNullishOrThrow(shop.gates[1],).resolve({
          record: 'r2',
          persisted: true,
        },);
        expect(await second,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r2',
            persisted: true,
          },
        },);
        expect(
          await nonNullishOrThrow(memo.get('shared',),),
        ).toEqual({
          kind: 'stored',
          record: 'r2',
        },);
      },
    },),

    it({
      name: 'a third twin waits for the second rather than buying beside it',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = asking({
          memo,
          shop,
        },);
        await settle();
        const second = asking({
          memo,
          shop,
        },);
        const third = asking({
          memo,
          shop,
        },);
        await settle();

        nonNullishOrThrow(shop.gates[0],).resolve({
          record: 'unheard',
          persisted: false,
        },);
        await first;
        await settle();
        // Only the second registered a buy; the third found its entry and waits.
        expect(shop.gates,).toHaveLength(2,);

        nonNullishOrThrow(shop.gates[1],).resolve({
          record: 'r2',
          persisted: true,
        },);
        expect(await second,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r2',
            persisted: true,
          },
        },);
        expect(await third,).toEqual({
          kind: 'reused',
          twin: 'r2',
        },);
        expect(shop.gates,).toHaveLength(2,);
      },
    },),

    it({
      name: 'a failed buy withdraws its entry and warns, and the waiting twin asks for itself',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = collected({
          run: asking({
            memo,
            shop,
          },),
        },);
        await settle();
        const second = asking({
          memo,
          shop,
        },);
        await settle();

        /**
         What the first purchase throws.
         */
        const fault = new Error('the provider dropped the first purchase',);
        nonNullishOrThrow(shop.gates[0],).reject(fault,);
        expect(await first,).toBe(fault,);
        await settle();
        expect(shop.gates,).toHaveLength(2,);
        expect(shop.warned,).toHaveLength(1,);
        expect(nonNullishOrThrow(shop.warned[0],),).toContain('asks for itself',);

        nonNullishOrThrow(shop.gates[1],).resolve({
          record: 'r2',
          persisted: true,
        },);
        expect(await second,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r2',
            persisted: true,
          },
        },);
      },
    },),

    it({
      name: 'a failed buy warns of its failure by class and never by its message, which an unsendable header quotes',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = collected({
          run: asking({
            memo,
            shop,
          },),
        },);
        await settle();
        const second = asking({
          memo,
          shop,
        },);
        await settle();

        nonNullishOrThrow(shop.gates[0],).reject(quotingFailure(),);
        await first;
        await settle();
        expect(shop.warned,).toEqual([
          'twin memo: the buy for a shared key was abandoned (refused by TypeError), so any twin waiting on it asks for itself',
        ],);
        nonNullishOrThrow(shop.gates[1],).resolve({
          record: 'r3',
          persisted: true,
        },);
        await second;
      },
    },),

    it({
      name: 'a failed buy that was a provider status failure warns of its class and status, then the provider\'s own '
        + 'words labelled as its own',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = collected({
          run: asking({
            memo,
            shop,
          },),
        },);
        await settle();
        const second = asking({
          memo,
          shop,
        },);
        await settle();

        nonNullishOrThrow(shop.gates[0],).reject(new ProviderHttpError({ provider: 'synthetic', status: 429, bodyText: 'the cat is out of credit', },),);
        await first;
        await settle();
        expect(shop.warned,).toEqual([
          'twin memo: the buy for a shared key was abandoned (refused by ProviderHttpError from synthetic with HTTP 429 '
            + '(the provider said: "the cat is out of credit")), so any twin waiting on it asks for itself',
        ],);
        nonNullishOrThrow(shop.gates[1],).resolve({
          record: 'r3',
          persisted: true,
        },);
        await second;
      },
    },),

    it({
      name: 'WRITES NO ABANDONED-BUY LINE for a buy ending in the caller\'s own reason once the caller has stopped, '
        + 'still withdraws the key so the waiting twin asks for itself, and leaves the buyer to throw that reason',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        /**
         Why the caller stopped the work.
         */
        const reason = new Error('the owner called the cats in',);
        const first = collected({
          run: asking({
            memo,
            shop,
            signal: steering.signal,
          },),
        },);
        await settle();
        const second = collected({
          run: asking({
            memo,
            shop,
            signal: steering.signal,
          },),
        },);
        await settle();

        steering.abort(reason,);
        nonNullishOrThrow(shop.gates[0],).reject(reason,);
        expect(await first,).toBe(reason,);
        await settle();
        // The key was withdrawn, so the waiting twin started a buy of its own,
        // which the real buyers end in the same reason under the same stop.
        expect(shop.gates,).toHaveLength(2,);

        nonNullishOrThrow(shop.gates[1],).reject(reason,);
        expect(await second,).toBe(reason,);
        expect(memo.has('shared',),).toBe(false,);
        expect(shop.warned,).toEqual([],);
      },
    },),

    it({
      name: 'KEEPS THE ABANDONED-BUY LINE, with the provider\'s words, for a buy\'s own failure that ends once the '
        + 'caller has stopped',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        /**
         The caller's steering.
         */
        const steering = new AbortController();
        const first = collected({
          run: asking({
            memo,
            shop,
            signal: steering.signal,
          },),
        },);
        await settle();

        steering.abort(new Error('the owner called the cats in',),);
        /**
         What the buy ends in, of its own.
         */
        const refusal = new ProviderHttpError({
          provider: 'synthetic',
          status: 503,
          bodyText: 'the cat is napping in the sun',
        },);
        nonNullishOrThrow(shop.gates[0],).reject(refusal,);
        expect(await first,).toBe(refusal,);
        await settle();
        expect(shop.warned,).toEqual([
          'twin memo: the buy for a shared key was abandoned (refused by ProviderHttpError from synthetic with HTTP 503 '
            + '(the provider said: "the cat is napping in the sun")), so any twin waiting on it asks for itself',
        ],);
      },
    },),

    it({
      name: 'a different key never waits',
      fn: async () => {
        const memo: TwinMemo<string> = new Map();
        const shop = buyer();
        const first = asking({
          memo,
          shop,
          key: 'one question',
        },);
        const second = asking({
          memo,
          shop,
          key: 'another question',
        },);
        await settle();
        expect(shop.gates,).toHaveLength(2,);

        nonNullishOrThrow(shop.gates[1],).resolve({
          record: 'r2',
          persisted: true,
        },);
        nonNullishOrThrow(shop.gates[0],).resolve({
          record: 'r1',
          persisted: true,
        },);
        expect(await first,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r1',
            persisted: true,
          },
        },);
        expect(await second,).toEqual({
          kind: 'bought',
          bought: {
            record: 'r2',
            persisted: true,
          },
        },);
      },
    },),
  ],
},);
