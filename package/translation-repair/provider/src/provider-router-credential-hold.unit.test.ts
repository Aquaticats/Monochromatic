/**
 Guards the credential hold (zheermao19, 2026-10-20): the Bedrock key expired,
 every Bedrock call answered HTTP 401, and the seat tally withheld
 `gemma-4-26b-a4b-it` after four failed asks though OpenRouter, wet all run,
 serves it. A 401 is the provider's credential refused, and one credential
 serves every model, so the router now holds the PROVIDER out for the rest of
 the run: the call goes on to the next provider serving the model, every other
 model skips the provider without a call, and a model only that provider serves
 is refused without one. The warn is said once per provider and names where the
 seat goes. Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  exchangeFailureLogText,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_DECISIONS,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type BudgetView,
  createCredentialHolds,
  createRoutingClient,
  type DecisionReply,
  type DecisionRequest,
} from '../dist/final/node/index.mjs';
import { holdNothing, } from './provider-router-wet-budgets.test-fixture.ts';
import { textCallOutcome, } from './provider-router-text-call.test-fixture.ts';

/**
 Status a provider answers with when it rejects the request's credential.
 */
const HTTP_UNAUTHORIZED = 401;

/**
 Words the refused credential's reply carries.
 */
const EXPIRED_WORDS = '{"error":{"code":"invalid_api_key","message":"The security token included in the request has expired."}}';

/**
 Status a provider answers with when its budget is spent.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

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
 What the stubs record: each provider and model asked, in call order.
 */
type Asked = string[];

/**
 Budgets reading NeuralWatt and Synthetic dry and the rest wet, so a model
 Bedrock serves is asked of Bedrock first, then Hyper, then OpenRouter.

 @param alsoDry - providers that read dry besides those two

 @returns Budgets of the shape the router reads, marking nothing

 @example
 ```ts
 const budgets = budgetsWhereBedrockComesFirst();
 ```
 */
function budgetsWhereBedrockComesFirst(
  { alsoDry = [], }: { readonly alsoDry?: readonly ProviderName[]; } = {},
): {
  readonly read: () => Promise<BudgetView>;
  readonly markRefused: () => Promise<void>;
  readonly holds: () => ProviderRecord<number>;
} {
  return {
    read: async function readOnlyOlderProvidersDry(): Promise<BudgetView> {
      return {
        neuralwatt: true,
        synthetic: true,
        hyper: alsoDry.includes('hyper',),
        bedrock: alsoDry.includes('bedrock',),
        deepseek: alsoDry.includes('deepseek',),
        xiaomi: alsoDry.includes('xiaomi',),
        anthropic: alsoDry.includes('anthropic',),
        openrouter: alsoDry.includes('openrouter',),
      };
    },
    markRefused: async function markNothingRefused(): Promise<void> {
      return undefined;
    },
    holds: holdNothing,
  };
}

/**
 Builds a router whose named providers answer every call with HTTP 401 or, for
 the budget refusers, HTTP 429, and whose other providers answer with their
 own name.

 @param refusers - providers whose credential is refused

 @param budgetRefusers - providers whose budget refuses every call, none when
 none are named

 @param alsoDry - providers whose meter reads dry besides NeuralWatt and Synthetic

 @param credentialHolds - holds shared with another router, a fresh set when none are given

 @param decide - the decisions endpoint, absent for none

 @returns Router, the calls asked, and the log lines written

 @example
 ```ts
 const { client, asked, lines, } = routerWhereCredentialRefused({ refusers: ['bedrock',], },);
 ```
 */
function routerWhereCredentialRefused(
  {
    refusers,
    budgetRefusers = [],
    alsoDry = [],
    credentialHolds,
    decide,
  }: {
    readonly refusers: readonly ProviderName[];
    readonly budgetRefusers?: readonly ProviderName[];
    readonly alsoDry?: readonly ProviderName[];
    readonly credentialHolds?: ReturnType<typeof createCredentialHolds>;
    readonly decide?: (request: DecisionRequest,) => Promise<DecisionReply>;
  },
): {
  readonly client: ReturnType<typeof createRoutingClient>;
  readonly asked: Asked;
  readonly lines: string[];
} {
  /**
   Providers and models asked, in call order.
   */
  const asked: Asked = [];
  /**
   Lines the router writes.
   */
  const lines: string[] = [];

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records the call and answers or refuses the credential
   */
  function callerFor(provider: ProviderName,): { readonly chatText: (request: { readonly modelId: string; },) => Promise<{ readonly text: string; }>; } {
    return {
      chatText: async function chatText({ modelId, }: { readonly modelId: string; },): Promise<{ readonly text: string; }> {
        asked.push(`${provider}:${modelId}`,);
        if (refusers.includes(provider,))
          throw new ProviderHttpError({
            provider,
            status: HTTP_UNAUTHORIZED,
            bodyText: EXPIRED_WORDS,
          },);
        if (budgetRefusers.includes(provider,))
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
    asked,
    lines,
    client: createRoutingClient({
      callers: {
        neuralwatt: callerFor('neuralwatt',),
        synthetic: callerFor('synthetic',),
        hyper: callerFor('hyper',),
        bedrock: callerFor('bedrock',),
        openrouter: callerFor('openrouter',),
      },
      budgets: budgetsWhereBedrockComesFirst({ alsoDry, },),
      l: levelCapturingLogger({ lines, },),
      ...((credentialHolds === undefined) ? {} : { credentialHolds, }),
      ...((decide === undefined) ? {} : { decider: { decide, }, }),
    },),
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
 The refusal a provider raises when it rejects the credential, as the log
 line quotes it.

 @param provider - provider that refused

 @returns The failure and the provider's words, as the warn renders them

 @example
 ```ts
 const said = refusalSaid({ provider: 'bedrock', },);
 ```
 */
function refusalSaid({ provider, }: { readonly provider: ProviderName; },): string {
  return exchangeFailureLogText({
    error: new ProviderHttpError({
      provider,
      status: HTTP_UNAUTHORIZED,
      bodyText: EXPIRED_WORDS,
    },),
  },);
}

await describe({
  name: 'a provider that rejects the credential with HTTP 401 is held out for the rest of the run (zheermao19)',
  children: [
    it({
      name: 'HEARS A SEAT FROM THE NEXT PROVIDER SERVING IT when the first answers 401, in the same call',
      fn: async () => {
        const { client, asked, } = routerWhereCredentialRefused({ refusers: ['bedrock',], },);

        const outcome = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect(outcome,).toEqual({ text: 'hyper answers', },);
        expect(asked,).toEqual([
          `bedrock:${SEAT_HYPER_TEXT_BEDROCK}`,
          `hyper:${SEAT_HYPER_TEXT_BEDROCK}`,
        ],);
      },
    },),
    it({
      name: 'SKIPS THE REFUSED PROVIDER FOR EVERY OTHER MODEL without asking it, the seat that met the 401 included',
      fn: async () => {
        const { client, asked, } = routerWhereCredentialRefused({ refusers: ['bedrock',], },);

        await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        const other = await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        const again = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect([other, again,],).toEqual([{ text: 'hyper answers', }, { text: 'hyper answers', },],);
        expect(asked,).toEqual([
          `bedrock:${SEAT_HYPER_TEXT_BEDROCK}`,
          `hyper:${SEAT_HYPER_TEXT_BEDROCK}`,
          `hyper:${SEAT_SYNTHETIC_TEXT_EVERYWHERE}`,
          `hyper:${SEAT_HYPER_TEXT_BEDROCK}`,
        ],);
      },
    },),
    it({
      name: 'WITHHOLDS A MODEL ONLY THE REFUSED PROVIDER SERVES without a call after the first, and says no provider is left',
      fn: async () => {
        const { client, asked, lines, } = routerWhereCredentialRefused({ refusers: ['bedrock',], },);

        const first = await ask({
          client,
          modelId: SEAT_BEDROCK_ONLY_TEXT,
        },);
        const second = await ask({
          client,
          modelId: SEAT_BEDROCK_ONLY_TEXT,
        },);

        expect({
          first: ('thrown' in first) && (first.thrown instanceof NoProviderForModelError),
          second: ('thrown' in second) ? String(second.thrown,) : 'answered',
          asked,
        },).toEqual({
          first: true,
          second: `NoProviderForModelError: no provider can take ${SEAT_BEDROCK_ONLY_TEXT}: `
            + 'every provider serving this model had its credential refused this run (bedrock)',
          asked: [`bedrock:${SEAT_BEDROCK_ONLY_TEXT}`,],
        },);
        expect(lines,).toEqual([
          `warn [callOn] SEAT CREDENTIAL REFUSED ${SEAT_BEDROCK_ONLY_TEXT} on bedrock: ${refusalSaid({ provider: 'bedrock', },)}. `
            + 'bedrock is held out for every model for the rest of the run, because one credential serves them all. '
            + `No provider that serves ${SEAT_BEDROCK_ONLY_TEXT} is left, so the seat is withheld for the rest of the run. `
            + 'Renew bedrock\'s credential before the next run, or leave it.',
        ],);
      },
    },),
    it({
      name: 'SAYS IT ONCE PER PROVIDER, naming the provider the seat moves to, and says nothing for a later 401 or another model',
      fn: async () => {
        const { client, lines, } = routerWhereCredentialRefused({ refusers: ['bedrock', 'hyper',], },);

        await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);

        expect(lines,).toEqual([
          `warn [callOn] SEAT CREDENTIAL REFUSED ${SEAT_HYPER_TEXT_BEDROCK} on bedrock: ${refusalSaid({ provider: 'bedrock', },)}. `
            + 'bedrock is held out for every model for the rest of the run, because one credential serves them all. '
            + `${SEAT_HYPER_TEXT_BEDROCK} moves to hyper, then openrouter (every provider that serves it and can take a call now). `
            + 'Renew bedrock\'s credential before the next run, or leave it.',
          `warn [callOn] SEAT CREDENTIAL REFUSED ${SEAT_HYPER_TEXT_BEDROCK} on hyper: ${refusalSaid({ provider: 'hyper', },)}. `
            + 'hyper is held out for every model for the rest of the run, because one credential serves them all. '
            + `${SEAT_HYPER_TEXT_BEDROCK} moves to openrouter (every provider that serves it and can take a call now). `
            + 'Renew hyper\'s credential before the next run, or leave it.',
        ],);
      },
    },),
    it({
      name: 'HOLDS OPENROUTER OUT OF THE DECISIONS ENDPOINT TOO, the one credential serving chat and decisions alike',
      fn: async () => {
        /** Decisions asked of the endpoint. */
        const decided: string[] = [];
        const { client, } = routerWhereCredentialRefused({
          refusers: ['openrouter',],
          decide: async function decide(request,): Promise<DecisionReply> {
            decided.push(request.modelId,);
            return { answers: ['windowsill',], } as unknown as DecisionReply;
          },
        },);

        await ask({
          client,
          modelId: SEAT_OPENROUTER_ONLY,
        },);
        await expect(client.decide?.({
          modelId: SEAT_OPENROUTER_DECISIONS,
          signal: SIGNAL,
        } as unknown as DecisionRequest,),).rejects.toThrow(NoProviderForModelError,);

        expect(decided,).toEqual([],);
      },
    },),
    it({
      name: 'HOLDS OPENROUTER OUT OF CHAT WHEN THE DECISIONS ENDPOINT MEETS THE REFUSAL, and says no provider is left for '
        + 'the decisions seat',
      fn: async () => {
        const refusal = new ProviderHttpError({
          provider: 'openrouter',
          status: HTTP_UNAUTHORIZED,
          bodyText: EXPIRED_WORDS,
        },);
        const { client, asked, lines, } = routerWhereCredentialRefused({
          refusers: [],
          decide: async function decide(): Promise<DecisionReply> {
            throw refusal;
          },
        },);

        await expect(client.decide?.({
          modelId: SEAT_OPENROUTER_DECISIONS,
          signal: SIGNAL,
        } as unknown as DecisionRequest,),).rejects.toBe(refusal,);
        const chat = await ask({
          client,
          modelId: SEAT_OPENROUTER_ONLY,
        },);

        expect(('thrown' in chat) && (chat.thrown instanceof NoProviderForModelError),).toBe(true,);
        expect(asked,).toEqual([],);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain(`No provider that serves ${SEAT_OPENROUTER_DECISIONS} is left, so the seat is withheld`,);
      },
    },),
    it({
      name: 'COUNTS THE ATTEMPTS OVER THE PROVIDERS STILL OPEN: after Bedrock is held, Hyper and OpenRouter refusing on '
        + 'their budgets are each asked once and the last one\'s refusal ends the call',
      fn: async () => {
        const { client, asked, } = routerWhereCredentialRefused({
          refusers: ['bedrock',],
          budgetRefusers: ['hyper', 'openrouter',],
        },);

        await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        asked.length = 0;
        const second = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect(asked,).toEqual([`hyper:${SEAT_HYPER_TEXT_BEDROCK}`, `openrouter:${SEAT_HYPER_TEXT_BEDROCK}`,],);
        expect(('thrown' in second) && (second.thrown instanceof ProviderHttpError),).toBe(true,);
      },
    },),
    it({
      name: 'NAMES NO DRY PROVIDER AS WHERE THE SEAT MOVES TO, and says the seat is not withheld when every provider '
        + 'left serving it is dry',
      fn: async () => {
        const { client, lines, } = routerWhereCredentialRefused({
          refusers: ['bedrock',],
          alsoDry: ['hyper', 'openrouter',],
        },);

        await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect(lines[0],).toContain(
          `No provider that serves ${SEAT_HYPER_TEXT_BEDROCK} can take a call now (hyper is dry, openrouter is dry), `
            + 'so the seat is not withheld on this refusal: the router goes on by its rules for a dry or held provider. ',
        );
        expect(lines[0],).not.toContain('is withheld for the rest of the run',);
      },
    },),
    it({
      name: 'LEAVES A DRY PROVIDER OUT OF THE LIST but names the one still wet',
      fn: async () => {
        const { client, lines, } = routerWhereCredentialRefused({
          refusers: ['bedrock',],
          alsoDry: ['hyper',],
        },);

        await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect(lines[0],).toContain(
          `${SEAT_HYPER_TEXT_BEDROCK} moves to openrouter (every provider that serves it and can take a call now). `,
        );
      },
    },),
    it({
      name: 'HOLDS THE PROVIDER OUT ON A 401 MET BY THE CROSS-PROVIDER RE-ASK exactly as on a first ask: said once, and '
        + 'skipped for every model after',
      fn: async () => {
        const { client, asked, lines, } = routerWhereCredentialRefused({ refusers: ['bedrock',], },);

        /** What the re-ask, hinted away from Hyper, came to: Bedrock is the next provider and refuses. */
        let reasked: unknown;
        try {
          reasked = await client.chatText({
            modelId: SEAT_HYPER_TEXT_BEDROCK,
            messages: MESSAGES,
            signal: SIGNAL,
            otherThan: 'hyper',
          },);
        }
        catch (error) {
          reasked = error;
        }
        const later = await ask({
          client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect(reasked,).toBeInstanceOf(ProviderHttpError,);
        expect(later,).toEqual({ text: 'hyper answers', },);
        expect(asked,).toEqual([`bedrock:${SEAT_HYPER_TEXT_BEDROCK}`, `hyper:${SEAT_HYPER_TEXT_BEDROCK}`,],);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain(`SEAT CREDENTIAL REFUSED ${SEAT_HYPER_TEXT_BEDROCK} on bedrock: `,);
      },
    },),
    it({
      name: 'SHARES A HOLD LEARNED BY ONE ROUTER with another built over the same holds, so a run with several routers '
        + 'learns a 401 once',
      fn: async () => {
        const credentialHolds = createCredentialHolds();
        const first = routerWhereCredentialRefused({ refusers: ['bedrock',], credentialHolds, },);
        const second = routerWhereCredentialRefused({ refusers: [], credentialHolds, },);

        await ask({
          client: first.client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);
        const outcome = await ask({
          client: second.client,
          modelId: SEAT_HYPER_TEXT_BEDROCK,
        },);

        expect(outcome,).toEqual({ text: 'hyper answers', },);
        expect(second.asked,).toEqual([`hyper:${SEAT_HYPER_TEXT_BEDROCK}`,],);
        expect(second.lines,).toEqual([],);
      },
    },),
  ],
},);
