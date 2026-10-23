/**
 Guards class one hundred forty-nine (hulicaijia30, 2026-09-26): Hyper's
 thousand request starts in a rolling hour were spent in the pass's first 33
 minutes, the pacer made the next Hyper call wait 1,640,012 ms, and the
 router kept sending Hyper the models OpenRouter serves too, so eight
 translate slices stalled 27 to 34 minutes. A provider whose request window
 would make the call wait is saturated: the call overflows to a usable
 provider behind it, and queues on the paced provider only when nobody
 usable stands behind. Cat-themed invention throughout; no corpus content
 appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { createRoutingClient, } from '../dist/final/node/index.mjs';
import { stubWetBudgets, } from './provider-router-wet-budgets.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Wait Hyper's window names while it is full in these cases.
 */
const FULL_WINDOW_WAIT_MS = 1_640_012;

/**
 Plain text conversation reused across routed calls.
 */
const MESSAGES = [
  {
    role: 'user' as const,
    content: '这只猫今天要吃几顿？',
  },
];

/**
 Abort signal every routed call carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Builds stub providers that record every call and answer at once.

 @returns Callers plus every provider and model asked, in order

 @example
 ```ts
 const { callers, asked, } = stubProviders();
 ```
 */
function stubProviders(): {
  readonly callers: ProviderRecord<{
    readonly chatText: (request: { readonly modelId: string; },) => Promise<{ readonly text: string; }>;
  }>;
  readonly asked: string[];
} {
  /**
   Providers and models asked, in call order.
   */
  const asked: string[] = [];

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records the call and answers
   */
  function callerFor(provider: ProviderName,): { readonly chatText: (request: { readonly modelId: string; },) => Promise<{ readonly text: string; }>; } {
    return {
      chatText: async function chatText({ modelId, }: { readonly modelId: string; },): Promise<{ readonly text: string; }> {
        asked.push(`${provider}:${modelId}`,);
        return { text: `${provider} answers`, };
      },
    };
  }
  return {
    asked,
    callers: {
      neuralwatt: callerFor('neuralwatt',),
      synthetic: callerFor('synthetic',),
      hyper: callerFor('hyper',),
      bedrock: callerFor('bedrock',),
      deepseek: callerFor('deepseek',),
      xiaomi: callerFor('xiaomi',),
      anthropic: callerFor('anthropic',),
      openrouter: callerFor('openrouter',),
    },
  };
}

/**
 Routes one call over the stubs with Hyper's window reading the given wait.

 @param modelId - model to ask

 @param hyperWaitMs - how long Hyper's pacer would make a call wait now

 @returns Every provider and model asked

 @example
 ```ts
 const asked = await routedWith({ modelId: SEAT_HYPER_VISION, hyperWaitMs: 0, },);
 ```
 */
async function routedWith(
  {
    modelId,
    hyperWaitMs,
  }: {
    readonly modelId: typeof SEAT_HYPER_VISION | typeof SEAT_HYPER_ONLY;
    readonly hyperWaitMs: number;
  },
): Promise<readonly string[]> {
  const { callers, asked, } = stubProviders();
  const client = createRoutingClient({
    callers,
    budgets: stubWetBudgets(),
    paces: {
      hyper: {
        waitMs: function waitMs(): number {
          return hyperWaitMs;
        },
      },
    },
  },);
  await client.chatText({
    modelId,
    messages: MESSAGES,
    signal: SIGNAL,
  },);
  return asked;
}

await describe({
  name: 'a provider whose request window would make the call wait is saturated (class one hundred forty-nine, hulicaijia30)',
  children: [
    it({
      name: 'SENDS a model Hyper and OpenRouter both serve to OpenRouter while Hyper\'s window is full',
      fn: async () => {
        expect(await routedWith({
          modelId: SEAT_HYPER_VISION,
          hyperWaitMs: FULL_WINDOW_WAIT_MS,
        },),).toEqual([`openrouter:${SEAT_HYPER_VISION}`,],);
      },
    },),
    it({
      name: 'KEEPS the same model on Hyper while its window has room',
      fn: async () => {
        expect(await routedWith({
          modelId: SEAT_HYPER_VISION,
          hyperWaitMs: 0,
        },),).toEqual([`hyper:${SEAT_HYPER_VISION}`,],);
      },
    },),
    it({
      name: 'QUEUES a model only Hyper serves on Hyper even while its window is full',
      fn: async () => {
        expect(await routedWith({
          modelId: SEAT_HYPER_ONLY,
          hyperWaitMs: FULL_WINDOW_WAIT_MS,
        },),).toEqual([`hyper:${SEAT_HYPER_ONLY}`,],);
      },
    },),
  ],
},);
