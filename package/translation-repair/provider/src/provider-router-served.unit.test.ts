/**
 Tests that the router reports to the budgets which provider served each call, which is what lets the budgets tell a
 refusal right after a refusal cooldown ran out (nothing served in between) from a refusal of a provider that came
 back (`provider-budget-refusal-holds.ts`).

 The providers that answer are Hyper and OpenRouter, which the owner keeps funded; Bedrock is kept off and never topped
 up, so no case routes to it.

 Fixtures are cat-themed invention; the callers are stubs and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type BudgetView,
  createRoutingClient,
} from '../dist/final/node/index.mjs';
import { holdNothing, } from './provider-router-wet-budgets.test-fixture.ts';
import { textCallOutcome, } from './provider-router-text-call.test-fixture.ts';

/**
 Status a provider answers with when its budget is spent.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Abort signal every routed call carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Builds a router over callers that refuse or answer, and records who was reported as having served.

 @param refusers - providers whose budget refuses every call

 @returns Router, and the providers reported as having served in report order

 @example
 ```ts
 const { client, served, } = routerReportingService({ refusers: ['hyper',], },);
 ```
 */
function routerReportingService(
  { refusers = [], }: { readonly refusers?: readonly ProviderName[]; } = {},
): {
  readonly client: ReturnType<typeof createRoutingClient>;
  readonly served: ProviderName[];
} {
  /**
   Providers reported as having served a call.
   */
  const served: ProviderName[] = [];

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that refuses the budget or answers with its name
   */
  function callerFor(provider: ProviderName,): { readonly chatText: () => Promise<{ readonly text: string; }>; } {
    return {
      chatText: async function chatText(): Promise<{ readonly text: string; }> {
        if (refusers.includes(provider,))
          throw new ProviderHttpError({
            provider,
            status: HTTP_TOO_MANY_REQUESTS,
            bodyText: `${provider} napping`,
          },);
        return { text: `${provider} answers`, };
      },
    };
  }
  return {
    served,
    client: createRoutingClient({
      callers: {
        neuralwatt: callerFor('neuralwatt',),
        synthetic: callerFor('synthetic',),
        hyper: callerFor('hyper',),
        openrouter: callerFor('openrouter',),
      },
      budgets: {
        read: async function readOnlyOlderProvidersDry(): Promise<BudgetView> {
          return {
            neuralwatt: true,
            synthetic: true,
            hyper: false,
            bedrock: true,
            deepseek: false,
            xiaomi: false,
            anthropic: false,
            openrouter: false,
          };
        },
        markRefused: async function markNothingRefused(): Promise<void> {
          return undefined;
        },
        markServed: function markServed({ provider, }: { readonly provider: ProviderName; },): void {
          served.push(provider,);
        },
        holds: holdNothing,
      },
      l: levelCapturingLogger({ lines: [], },),
    },),
  };
}

await describe({
  name: `${createRoutingClient.name} service reports`,
  concurrency: 1,
  children: [
    it({
      name: 'REPORTS THE PROVIDER THAT ANSWERED a call as having served it',
      fn: async () => {
        const { client, served, } = routerReportingService();
        expect(await textCallOutcome({
          client,
          modelId: SEAT_HYPER_ONLY,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
        },),).toEqual({ text: 'hyper answers', },);
        expect(served,).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'REPORTS ONLY THE PROVIDER THAT ANSWERED when the first refused and the call went on to the next',
      fn: async () => {
        const { client, served, } = routerReportingService({ refusers: ['hyper',], },);
        expect(await textCallOutcome({
          client,
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
        },),).toEqual({ text: 'openrouter answers', },);
        expect(served,).toEqual(['openrouter',],);
      },
    },),

    it({
      name: 'REPORTS NOBODY when every provider serving the model refuses',
      fn: async () => {
        const { client, served, } = routerReportingService({
          refusers: [
            'hyper',
            'openrouter',
          ],
        },);
        expect('thrown' in await textCallOutcome({
          client,
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
        },),).toBe(true,);
        expect(served,).toEqual([],);
      },
    },),
  ],
},);
