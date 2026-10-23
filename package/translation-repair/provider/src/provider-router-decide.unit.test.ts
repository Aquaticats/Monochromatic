/**
 Tests the router's typed-decision exchange (reach under ledger T8), which
 OpenRouter alone serves: refused with no decisions client and while the
 OpenRouter meter reads dry, the endpoint's reply returned otherwise, and a
 budget refusal marking the OpenRouter meter as a chat refusal would before
 it is thrown on. Model ids and replies are cat-themed invention; requests
 carry only what the router reads, cast past their full types.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type BudgetView,
  createRoutingClient,
  type DecisionReply,
  type DecisionRequest,
  statedWaitMsOf,
} from '../dist/final/node/index.mjs';
import { holdNothing, } from './provider-router-wet-budgets.test-fixture.ts';

/**
 Abort signal every call in these cases carries.
 */
const SIGNAL = new AbortController().signal;

/**
 A decision request naming one model, cast past the fields the router never
 reads.
 */
const REQUEST = {
  modelId: 'hf:cat/Tabby',
  signal: SIGNAL,
} as unknown as DecisionRequest;

/**
 The endpoint's reply in these cases.
 */
const REPLY = { answers: ['windowsill',], } as unknown as DecisionReply;

/**
 A chat caller no case reaches.
 */
const UNUSED_CALLER = {
  chatText: async function chatText(): Promise<never> {
    throw new Error('chatText unused by the decision exchange',);
  },
};

/**
 Budgets reading OpenRouter wet or dry, recording every refusal marked.

 @param openrouterDry - whether the OpenRouter meter reads dry

 @returns The budgets and the refusals marked, without their signals
 */
function budgetsReading({ openrouterDry, }: { readonly openrouterDry: boolean; },) {
  /**
   Refusals marked.
   */
  const marked: {
    readonly provider: ProviderName;
    readonly statedWaitMs?: number;
    readonly paymentRequired?: boolean;
  }[] = [];
  return {
    marked,
    budgets: {
      read: async function read(): Promise<BudgetView> {
        return {
          neuralwatt: true,
          synthetic: false,
          hyper: false,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: openrouterDry,
        };
      },
      markRefused: async function markRefused(
        {
          provider,
          statedWaitMs,
          paymentRequired,
        }: {
          readonly provider: ProviderName;
          readonly statedWaitMs?: number;
          readonly paymentRequired?: boolean;
        },
      ): Promise<void> {
        marked.push({
          provider,
          ...((statedWaitMs === undefined) ? {} : { statedWaitMs, }),
          ...((paymentRequired === undefined) ? {} : { paymentRequired, }),
        },);
      },
      holds: holdNothing,
    },
  };
}

/**
 A router over unused chat callers, with the budgets and decider given.

 @param budgets - budgets the router reads

 @param decide - the decisions endpoint, absent for none

 @returns The router
 */
function routerWith(
  {
    budgets,
    decide,
  }: {
    readonly budgets: ReturnType<typeof budgetsReading>['budgets'];
    readonly decide?: (request: DecisionRequest,) => Promise<DecisionReply>;
  },
) {
  return createRoutingClient({
    callers: {
      neuralwatt: UNUSED_CALLER,
      synthetic: UNUSED_CALLER,
      hyper: UNUSED_CALLER,
      bedrock: UNUSED_CALLER,
      openrouter: UNUSED_CALLER,
    },
    budgets,
    ...((decide === undefined) ? {} : { decider: { decide, }, }),
  },);
}

/**
 Status of a balance that cannot pay for the call.
 */
const HTTP_PAYMENT_REQUIRED = 402;

/**
 Status of a rate limit.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Asserts a budget refusal from the endpoint marks the OpenRouter meter with
 its stated wait and payment flag, and reaches the caller unchanged.

 @param refusal - what the endpoint throws

 @param paymentRequired - whether the mark must say the balance refused
 */
async function expectMarkedAndThrown(
  {
    refusal,
    paymentRequired,
  }: {
    readonly refusal: InstanceType<typeof ProviderHttpError>;
    readonly paymentRequired: boolean;
  },
): Promise<void> {
  const {
    budgets,
    marked,
  } = budgetsReading({ openrouterDry: false, },);
  const router = routerWith({
    budgets,
    decide: async function decide(): Promise<never> {
      throw refusal;
    },
  },);
  await expect(router.decide?.(REQUEST,),).rejects.toBe(refusal,);
  expect(marked,).toStrictEqual([{
    provider: 'openrouter',
    statedWaitMs: statedWaitMsOf({ error: refusal, },),
    paymentRequired,
  },],);
}

await describe({
  name: `${createRoutingClient.name} decide`,
  children: [
    it({
      name: 'REFUSES WITH NO DECISIONS CLIENT configured',
      fn: async () => {
        const router = routerWith({ budgets: budgetsReading({ openrouterDry: false, },).budgets, },);
        await expect(router.decide?.(REQUEST,),).rejects.toThrow(NoProviderForModelError,);
      },
    },),
    it({
      name: 'REFUSES WHILE THE OPENROUTER METER READS DRY, before the endpoint is asked',
      fn: async () => {
        /**
         Requests the endpoint received.
         */
        const asked: DecisionRequest[] = [];
        const router = routerWith({
          budgets: budgetsReading({ openrouterDry: true, },).budgets,
          decide: async function decide(request,) {
            asked.push(request,);
            return REPLY;
          },
        },);
        await expect(router.decide?.(REQUEST,),).rejects.toThrow(NoProviderForModelError,);
        expect(asked,).toStrictEqual([],);
      },
    },),
    it({
      name: 'RETURNS THE ENDPOINT\'S REPLY while OpenRouter reads wet, marking nothing',
      fn: async () => {
        const { budgets, marked, } = budgetsReading({ openrouterDry: false, },);
        const router = routerWith({
          budgets,
          decide: async function decide() {
            return REPLY;
          },
        },);
        expect(await router.decide?.(REQUEST,),).toBe(REPLY,);
        expect(marked,).toStrictEqual([],);
      },
    },),
    it({
      name: 'MARKS OPENROUTER REFUSED ON A BUDGET REFUSAL, with its stated wait and whether it was a payment refusal, '
        + 'and throws it on',
      fn: async () => {
        await expectMarkedAndThrown({
          refusal: new ProviderHttpError({
            provider: 'openrouter',
            status: HTTP_PAYMENT_REQUIRED,
            bodyText: 'This request requires more credits',
          },),
          paymentRequired: true,
        },);
        await expectMarkedAndThrown({
          refusal: new ProviderHttpError({
            provider: 'openrouter',
            status: HTTP_TOO_MANY_REQUESTS,
            bodyText: 'Please try again in 1m30s',
          },),
          paymentRequired: false,
        },);
      },
    },),
    it({
      name: 'THROWS ANY OTHER FAILURE ON without marking a meter',
      fn: async () => {
        const { budgets, marked, } = budgetsReading({ openrouterDry: false, },);
        const failure = new Error('the cat sat on the keyboard',);
        const router = routerWith({
          budgets,
          decide: async function decide(): Promise<never> {
            throw failure;
          },
        },);
        await expect(router.decide?.(REQUEST,),).rejects.toBe(failure,);
        expect(marked,).toStrictEqual([],);
      },
    },),
  ],
},);
