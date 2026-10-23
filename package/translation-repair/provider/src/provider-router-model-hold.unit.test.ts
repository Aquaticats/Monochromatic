/**
 Guards class sixty-four (XingZ613, 2026-09-19): a model whose upstream
 endpoint rate-limited it (class sixty-two's passed-on 429) was asked again
 on every round, and every round waited its five-attempt ladder out in grace;
 the repair lane took 1h39m against XingZ608's 35m. The router now holds
 that one model out for the rate-limit backoff and refuses it at call time
 as unreachable, so a round neither asks nor waits for it, while every other
 model on the same provider is served and the provider's own budget is
 untouched. Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type BudgetView,
  createRoutingClient,
} from '../dist/final/node/index.mjs';
import {
  holdNothing,
  readOlderProvidersWet,
} from './provider-router-wet-budgets.test-fixture.ts';
import { textCallOutcome, } from './provider-router-text-call.test-fixture.ts';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Status a rate limit answers with.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Body OpenRouter answers with when one model's upstream endpoint rate-limits
 its shared pool, with the model renamed.
 */
const UPSTREAM_POOL_BODY = '{"error":{"message":"Provider returned error","code":429,"metadata":{"raw":"cats/whisker-1 is temporarily'
  + ' rate-limited upstream. Please retry shortly.","provider_name":"Catnip","provider_error_code":"rate_limit_exceeded",'
  + '"limit_source":"upstream_provider_shared_pool"}}}';

/**
 How long the router holds a rate-limited model out in these cases.
 */
const HOLD_MS = 1_000;

/**
 Plain text conversation reused across routed calls.
 */
const MESSAGES = [
  {
    role: 'user' as const,
    content: '这只猫睡在哪里？',
  },
];

/**
 Abort signal every routed call carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Builds stub providers whose first provider answers the rate-limited model
 with the upstream body once, then answers.

 @returns Callers plus the models each provider was asked for, in order

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
   Models asked of any provider, in call order.
   */
  const asked: string[] = [];
  /**
   Whether the first provider has refused the rate-limited model yet.
   */
  const refused = { once: false, };

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records the model and answers or refuses
   */
  function callerFor(provider: ProviderName,): { readonly chatText: (request: { readonly modelId: string; },) => Promise<{ readonly text: string; }>; } {
    return {
      chatText: async function chatText({ modelId, }: { readonly modelId: string; },): Promise<{ readonly text: string; }> {
        asked.push(`${provider}:${modelId}`,);
        /**
         Whether this is the first provider's first call for the model whose
         upstream rate-limits it.
         */
        const limited = (provider === 'synthetic') && (modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER) && (!refused.once);
        if (limited) {
          refused.once = true;
          throw new ProviderHttpError({
            provider,
            status: HTTP_TOO_MANY_REQUESTS,
            bodyText: UPSTREAM_POOL_BODY,
          },);
        }
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
 Builds a budget view where every provider reads wet and records who was
 marked as refusing.

 @returns Budgets plus the providers marked refused

 @example
 ```ts
 const { budgets, refused, } = stubBudgets();
 ```
 */
function stubBudgets(): {
  readonly budgets: {
    readonly read: () => Promise<BudgetView>;
    readonly markRefused: (args: { readonly provider: ProviderName; },) => Promise<void>;
    readonly holds: () => ProviderRecord<number>;
  };
  readonly refused: ProviderName[];
} {
  /**
   Providers marked as having refused us.
   */
  const refused: ProviderName[] = [];
  return {
    refused,
    budgets: {
      read: readOlderProvidersWet,
      markRefused: async function markRefused({ provider, }: { readonly provider: ProviderName; },): Promise<void> {
        refused.push(provider,);
      },
      holds: holdNothing,
    },
  };
}

/**
 Routes one text call and reports what happened.

 @param client - router under test

 @param modelId - model to ask

 @returns Reply text, or the error thrown

 @example
 ```ts
 const outcome = await ask({ client, modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },);
 ```
 */
function ask(
  {
    client,
    modelId,
  }: {
    readonly client: ReturnType<typeof createRoutingClient>;
    readonly modelId: Parameters<ReturnType<typeof createRoutingClient>['chatText']>[0]['modelId'];
  },
): Promise<{ readonly text: string; } | { readonly thrown: unknown; }> {
  return textCallOutcome({
    client,
    modelId,
    messages: MESSAGES,
    signal: SIGNAL,
  },);
}

await describe({
  name: 'a model whose upstream endpoint rate-limited it is held out alone (class sixty-four, XingZ613)',
  children: [
    it({
      name: 'HOLDS the model for the backoff, serves the provider\'s other models, and asks again once it ends',
      fn: async () => {
        const { callers, asked, } = stubProviders();
        const { budgets, refused, } = stubBudgets();
        /**
         Clock the router reads, advanced by the case.
         */
        const clock = { now: 0, };
        const client = createRoutingClient({
          callers,
          budgets,
          modelHoldMs: HOLD_MS,
          now: function now(): number {
            return clock.now;
          },
        },);

        const first = await ask({
          client,
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        },);
        // The passed-on limit is the seat's loss for the round, not a
        // re-route and not the provider's budget (class sixty-two).
        expect(('thrown' in first) && (first.thrown instanceof ProviderHttpError),).toBe(true,);
        expect(refused,).toEqual([],);

        const held = await ask({
          client,
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        },);
        // Inside the hold the model is refused at call time without a call,
        // which is what lets a round leave the seat out instead of waiting.
        expect(('thrown' in held) && (held.thrown instanceof NoProviderForModelError),).toBe(true,);
        expect(asked,).toEqual([`synthetic:${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}`,],);

        const other = await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        // The hold is the model's, not the provider's.
        expect(('text' in other) && (other.text === 'synthetic answers'),).toBe(true,);

        clock.now = HOLD_MS + 1;
        const again = await ask({
          client,
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
        },);
        expect(('text' in again) && (again.text === 'synthetic answers'),).toBe(true,);
        expect(asked.length,).toBe(3,);
      },
    },),
  ],
},);
