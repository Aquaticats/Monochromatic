/**
 Guards class one hundred forty-eight (hulicaijia29, 2026-09-26): Bedrock
 served one Gemma model at 219 to 263 s a stream for fifty minutes while its
 other models kept their pace, and every round waited on that seat up to the
 360 s deadline. A call cut at its card's measured stream bound now holds
 that provider out for that model: the call is asked again of the next
 provider serving the model, a model no other provider serves reads
 unreachable without a call, and the provider's other models are served.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  StreamCutShortError,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type BudgetView,
  createRoutingClient,
  StreamBoundError,
} from '../dist/final/node/index.mjs';
import { textCallOutcome, } from './provider-router-text-call.test-fixture.ts';
import {
  readOlderProvidersWet,
  stubWetBudgets,
  holdNothing,
} from './provider-router-wet-budgets.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 How long the router holds a provider out for one model in these cases.
 */
const HOLD_MS = 1_000;

/**
 The bound the stub's cut names.
 */
const BOUND_MS = 60_000;

/**
 Plain text conversation reused across routed calls.
 */
const MESSAGES = [
  {
    role: 'user' as const,
    content: '这只猫在窗台上睡了多久？',
  },
];

/**
 Abort signal every routed call carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Budgets reading every provider wet, none holding a call and none marked
 refused, so a call is free to go to each provider in turn.

 @returns Budgets of the shape the router reads

 @example
 ```ts
 const budgets = stubFullyWetBudgets();
 ```
 */
function stubFullyWetBudgets(): ReturnType<typeof stubWetBudgets> {
  return {
    read: readOlderProvidersWet,
    markRefused: async function markNobodyRefused(): Promise<void> {
      return undefined;
    },
    holds: holdNothing,
  };
}

/**
 Budgets that read a provider dry from the moment the router marks it
 refused and wet before, none holding a call, so a call a provider refuses
 goes on to one that has not refused yet.

 @returns Budgets of the shape the router reads

 @example
 ```ts
 const budgets = stubBudgetsDryOnceRefused();
 ```
 */
function stubBudgetsDryOnceRefused(): ReturnType<typeof stubWetBudgets> {
  /**
   Providers the router has marked refused.
   */
  const refused = new Set<ProviderName>();
  return {
    read: async function readDryWhereRefused(): Promise<BudgetView> {
      return {
        neuralwatt: true,
        synthetic: refused.has('synthetic',),
        hyper: refused.has('hyper',),
        bedrock: refused.has('bedrock',),
        deepseek: refused.has('deepseek',),
        xiaomi: refused.has('xiaomi',),
        anthropic: refused.has('anthropic',),
        openrouter: refused.has('openrouter',),
      };
    },
    markRefused: async function rememberRefused({ provider, }: { readonly provider: ProviderName; },): Promise<void> {
      refused.add(provider,);
    },
    holds: stubWetBudgets().holds,
  };
}

/**
 Status a provider answers with when its budget is spent.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Builds stub providers where the cutting providers (Bedrock alone by default)
 cut each named model once at its stream bound, then answer, and the refusing
 providers (none by default) refuse every call on their budget.

 @param slow - models the cutting providers cut on their first call

 @param cutters - providers that cut, Bedrock alone when none are named

 @param refusers - providers whose budget refuses every call, none when none
 are named

 @returns Callers plus every provider and model asked, in order

 @example
 ```ts
 const { callers, asked, } = stubProviders({ slow: [SEAT_BEDROCK_ONLY_TEXT,], },);
 ```
 */
function stubProviders(
  {
    slow,
    cutters = ['bedrock',],
    refusers = [],
  }: {
    readonly slow: readonly string[];
    readonly cutters?: readonly ProviderName[];
    readonly refusers?: readonly ProviderName[];
  },
): {
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
   Provider and model pairs already cut once.
   */
  const cut = new Set<string>();

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records the call and answers, cuts or refuses
   */
  function callerFor(provider: ProviderName,): { readonly chatText: (request: { readonly modelId: string; },) => Promise<{ readonly text: string; }>; } {
    return {
      chatText: async function chatText({ modelId, }: { readonly modelId: string; },): Promise<{ readonly text: string; }> {
        asked.push(`${provider}:${modelId}`,);
        if (refusers.includes(provider,))
          throw new ProviderHttpError({
            provider,
            status: HTTP_TOO_MANY_REQUESTS,
            bodyText: `${provider} napping`,
          },);
        if (cutters.includes(provider,) && slow.includes(modelId,) && (!cut.has(`${provider}:${modelId}`,))) {
          cut.add(`${provider}:${modelId}`,);
          throw new StreamCutShortError({
            label: modelId,
            partialText: '',
            progress: {
              firstByteMs: 15_000,
              firstContentMs: 15_000,
              maxGapMs: 15_000,
              chars: 30,
              elapsedMs: BOUND_MS,
            },
            cause: new StreamBoundError({
              label: modelId,
              boundMs: BOUND_MS,
            },),
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
 Routes one text call and reports what happened.

 @param client - router under test

 @param modelId - model to ask

 @returns Reply text, or the error thrown

 @example
 ```ts
 const outcome = await ask({ client, modelId: SEAT_BEDROCK_ONLY_TEXT, },);
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

/**
 Builds a router over the stubs with a clock the case advances.

 @param slow - models the cutting providers cut on their first call

 @param cutters - providers that cut, Bedrock alone when none are named

 @param refusers - providers whose budget refuses every call, none when none
 are named

 @param budgets - what every provider's budget reads as, Synthetic dry and the
 rest wet when none are given

 @returns Router, the calls asked, and the clock

 @example
 ```ts
 const { client, asked, clock, } = routerOver({ slow: [SEAT_BEDROCK_ONLY_TEXT,], },);
 ```
 */
function routerOver(
  {
    slow,
    cutters,
    refusers,
    budgets = stubWetBudgets(),
  }: {
    readonly slow: readonly string[];
    readonly cutters?: readonly ProviderName[];
    readonly refusers?: readonly ProviderName[];
    readonly budgets?: ReturnType<typeof stubWetBudgets>;
  },
): {
  readonly client: ReturnType<typeof createRoutingClient>;
  readonly asked: string[];
  readonly clock: { now: number; };
} {
  const { callers, asked, } = stubProviders({
    slow,
    ...((cutters === undefined) ? {} : { cutters, }),
    ...((refusers === undefined) ? {} : { refusers, }),
  },);
  /**
   Clock the router reads, advanced by the case.
   */
  const clock = { now: 0, };
  return {
    asked,
    clock,
    client: createRoutingClient({
      callers,
      budgets,
      modelHoldMs: HOLD_MS,
      now: function now(): number {
        return clock.now;
      },
    },),
  };
}

await describe({
  name: 'a call cut at its card\'s stream bound holds that provider out for that model (class one hundred forty-eight, hulicaijia29)',
  children: [
    it({
      name: 'ASKS the next provider serving the model in the same call, and keeps Bedrock out for the model until the hold ends',
      fn: async () => {
        const { client, asked, clock, } = routerOver({ slow: [SEAT_HYPER_TEXT_BEDROCK,], },);

        const first = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        expect(first,).toEqual({ text: 'hyper answers', },);

        const held = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        expect(held,).toEqual({ text: 'hyper answers', },);
        expect(asked,).toEqual([
          `bedrock:${SEAT_HYPER_TEXT_BEDROCK}`,
          `hyper:${SEAT_HYPER_TEXT_BEDROCK}`,
          `hyper:${SEAT_HYPER_TEXT_BEDROCK}`,
        ],);

        clock.now = HOLD_MS + 1;
        const again = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        expect(again,).toEqual({ text: 'bedrock answers', },);
      },
    },),
    it({
      name: 'READS a model only Bedrock serves unreachable without a call while the hold runs, and serves Bedrock\'s other models',
      fn: async () => {
        const { client, asked, } = routerOver({ slow: [SEAT_BEDROCK_ONLY_TEXT,], },);

        const first = await ask({
          client,
          modelId: SEAT_BEDROCK_ONLY_TEXT,
        },);
        expect(('thrown' in first),).toBe(true,);

        const held = await ask({
          client,
          modelId: SEAT_BEDROCK_ONLY_TEXT,
        },);
        // Refused without a call, which is what lets a round count the seat
        // out of its quorum instead of waiting on it.
        expect(('thrown' in held) && (held.thrown instanceof NoProviderForModelError),).toBe(true,);
        expect(asked,).toEqual([`bedrock:${SEAT_BEDROCK_ONLY_TEXT}`,],);

        const other = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        expect(other,).toEqual({ text: 'bedrock answers', },);
      },
    },),
    it({
      name: 'ENDS A CALL CUT AT ITS STREAM BOUND ON EVERY ONE OF THE FOUR PROVIDERS SERVING IT with the refusal a '
        + 'model fewer providers serve ends with, naming the stream bound and the hold, after asking each once in '
        + 'order, though the loop has no attempt left to read the holds on',
      fn: async () => {
        const { client, asked, } = routerOver({
          slow: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          cutters: ['synthetic', 'bedrock', 'hyper', 'openrouter',],
          budgets: stubFullyWetBudgets(),
        },);

        const outcome = await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);

        expect({
          asked,
          thrown: ('thrown' in outcome) ? String(outcome.thrown,) : 'answered',
          class: ('thrown' in outcome) && (outcome.thrown instanceof NoProviderForModelError),
        },).toEqual({
          asked: [
            `synthetic:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
            `bedrock:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
            `hyper:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
            `openrouter:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
          ],
          thrown: `NoProviderForModelError: no provider can take ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: `
            + `every provider serving this model ran past its stream bound; held out for another ${String(HOLD_MS,)}ms`,
          class: true,
        },);
      },
    },),
    it({
      name: 'ENDS A CALL THREE PROVIDERS REFUSED ON THEIR BUDGETS AND THE FOURTH CUT AT ITS STREAM BOUND with a '
        + 'refusal that names both, since the three are not held out on a stream bound and nobody is left to ask',
      fn: async () => {
        const { client, asked, } = routerOver({
          slow: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
          cutters: ['openrouter',],
          refusers: ['synthetic', 'bedrock', 'hyper',],
          budgets: stubBudgetsDryOnceRefused(),
        },);

        const outcome = await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);

        expect({
          asked,
          thrown: ('thrown' in outcome) ? String(outcome.thrown,) : 'answered',
          class: ('thrown' in outcome) && (outcome.thrown instanceof NoProviderForModelError),
        },).toEqual({
          asked: [
            `synthetic:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
            `bedrock:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
            `hyper:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
            `openrouter:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
          ],
          thrown: `NoProviderForModelError: no provider can take ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: `
            + 'every provider serving this model refused this call or ran past its stream bound on it',
          class: true,
        },);
      },
    },),
  ],
},);
