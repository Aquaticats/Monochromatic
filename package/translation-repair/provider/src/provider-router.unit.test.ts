/**
 Tests for the provider router: which provider takes a call, what happens
 when one refuses, and what a picture narrows.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { wait, } from '@monochromatic-dev/module-async-time/ts';
import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type BudgetView,
  createProviderBudgets,
  createRoutingClient,
  EveryProviderDryError,
  type NeuralWattReturn,
} from '../dist/final/node/index.mjs';
import { textCallOutcome, } from './provider-router-text-call.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

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
 Conversation carrying a picture, which narrows where it can go.
 */
const PICTURE_MESSAGES = [
  {
    role: 'user' as const,
    content: [
      { type: 'text' as const, text: 'Read this picture.', },
      { type: 'image_url' as const, image_url: { url: 'data:image/png;base64,AAAA', }, },
    ],
  },
];

/**
 Abort signal every routed call in these tests carries.
 */
const SIGNAL = new AbortController().signal;

/**
 How long the slow first-provider stub holds its slot: long enough for a
 concurrent caller to find it busy, short enough not to slow the suite.
 */
const SLOT_HOLD_MS = 50;

/**
 Synthetic provider stub that holds its one slot long enough for a concurrent
 caller to find it busy, recording that it was called.

 @param called - call log this stub appends to

 @returns Caller usable as the routing client's `synthetic` provider

 @example
 ```ts
 const synthetic = slotHoldingSynthetic({ called, },);
 ```
 */
function slotHoldingSynthetic(
  { called, }: { readonly called: ProviderName[]; },
): { chatText: () => Promise<{ text: string; }>; } {
  return {
    chatText: async function chatText() {
      called.push('synthetic',);
      await wait(SLOT_HOLD_MS,);
      return { text: '{"spot":"windowsill"}', };
    },
  };
}

/**
 How many same-model calls the default-routing case holds in flight at once: more than any width Synthetic once granted.
 */
const CONCURRENT_SYNTHETIC_CALLS = 12;

/**
 One Synthetic slot per model, no ceiling elsewhere.
 */
const ONE_SYNTHETIC_SLOT = {
  neuralwatt: Number.POSITIVE_INFINITY,
  synthetic: 1,
  hyper: Number.POSITIVE_INFINITY,
  bedrock: Number.POSITIVE_INFINITY,
  deepseek: Number.POSITIVE_INFINITY,
  xiaomi: Number.POSITIVE_INFINITY,
  anthropic: Number.POSITIVE_INFINITY,
  openrouter: Number.POSITIVE_INFINITY,
};

/**
 A value per provider, each one optional because a case names only the
 providers it scripts.
 */
type PerProvider<ValueT,> = {
  readonly neuralwatt?: ValueT;
  readonly synthetic?: ValueT;
  readonly hyper?: ValueT;
  readonly bedrock?: ValueT;
  readonly deepseek?: ValueT;
  readonly xiaomi?: ValueT;
  readonly anthropic?: ValueT;
  readonly openrouter?: ValueT;
};

/**
 Builds stub providers recording which took each call.

 @param status - status each provider refuses with, zero to answer

 @param refusals - how many calls each provider refuses before answering,
 every call by default when it has a refusing status

 @param text - what each provider answers when it answers

 @param bodyText - what each provider's refusal says, `refused` by default

 @returns Every provider's caller plus the log of who was called

 @example
 ```ts
 const { callers, called, } = stubProviders({},);
 ```
 */
function stubProviders(
  {
    status = {},
    refusals = {},
    text = {},
    bodyText = {},
  }: {
    readonly status?: PerProvider<number>;
    readonly refusals?: PerProvider<number>;
    readonly text?: PerProvider<string>;
    readonly bodyText?: PerProvider<string>;
  },
) {
  /**
   Providers asked, in call order.
   */
  const called: ProviderName[] = [];

  /**
   Answers each provider gives, distinguishable by default.
   */
  const answers: ProviderRecord<string> = {
    neuralwatt: text.neuralwatt ?? '{"spot":"warm lap"}',
    synthetic: text.synthetic ?? '{"spot":"windowsill"}',
    hyper: text.hyper ?? '{"spot":"radiator"}',
    bedrock: text.bedrock ?? '{"spot":"cat tree"}',
    deepseek: text.deepseek ?? '{"spot":"laundry basket"}',
    xiaomi: text.xiaomi ?? '{"spot":"laundry basket"}',
    anthropic: text.anthropic ?? '{"spot":"laundry basket"}',
    openrouter: text.openrouter ?? '{"spot":"laundry basket"}',
  };

  /**
   How many more calls each provider refuses before answering.
   */
  const refusalsLeft: Record<ProviderName, number> = {
    neuralwatt: refusals.neuralwatt ?? (((status.neuralwatt ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    synthetic: refusals.synthetic ?? (((status.synthetic ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    hyper: refusals.hyper ?? (((status.hyper ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    bedrock: refusals.bedrock ?? (((status.bedrock ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    deepseek: refusals.deepseek ?? (((status.deepseek ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    xiaomi: refusals.xiaomi ?? (((status.xiaomi ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    anthropic: refusals.anthropic ?? (((status.anthropic ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
    openrouter: refusals.openrouter ?? (((status.openrouter ?? 0) === 0) ? 0 : Number.POSITIVE_INFINITY),
  };

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records itself, refuses as told, then answers
   */
  function callerFor(provider: ProviderName,) {
    return {
      chatText: async function chatText() {
        called.push(provider,);
        if (refusalsLeft[provider] > 0) {
          refusalsLeft[provider] -= 1;
          throw new ProviderHttpError({
            provider,
            status: status[provider] ?? 0,
            bodyText: bodyText[provider] ?? 'refused',
          },);
        }
        return { text: answers[provider], };
      },
    };
  }

  return {
    called,
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
 Builds a budget view that answers as told and records refusals.

 @param dry - which providers read as out of budget

 @param holdsMs - what the holds report

 @param onHoldEnd - what a read after the holds were asked for does first

 @param neuralwatt - the NeuralWatt day the budgets report to a refusal that ends the run

 @returns Budget view plus the providers marked as having refused us

 @example
 ```ts
 const { budgets, refused, } = stubBudgets({},);
 ```
 */
function stubBudgets(
  {
    dry = {},
    holdsMs = {
      neuralwatt: 0,
      synthetic: 0,
      hyper: 0,
      bedrock: 0,
      deepseek: 0,
      xiaomi: 0,
      anthropic: 0,
      openrouter: 0,
    },
    onHoldEnd,
    neuralwatt = { kind: 'unanchored', },
  }: {
    readonly dry?: PerProvider<boolean>;
    readonly holdsMs?: ProviderRecord<number>;
    readonly onHoldEnd?: () => void;
    readonly neuralwatt?: NeuralWattReturn;
  },
) {
  /**
   Providers that reported themselves out of budget, in order.
   */
  const refused: ProviderName[] = [];

  /**
   Wait each refusal named, in the same order, zero for none.
   */
  const statedWaits: number[] = [];

  /**
   View as the meters read it, which a refusal then overrides.
   */
  const view: Record<ProviderName, boolean> = {
    // DRY UNLESS A CASE SAYS OTHERWISE, like Bedrock: these cases pin the order
    // of the four older providers, and NeuralWatt has its own cases
    // (`neuralwatt-routing.unit.test.ts`).
    neuralwatt: dry.neuralwatt ?? true,
    synthetic: dry.synthetic ?? false,
    hyper: dry.hyper ?? false,
    // DRY UNLESS A CASE SAYS OTHERWISE: the fourth provider serves none of the
    // models these cases route, and a wet meter on a provider that serves
    // nothing would turn an all-dry ending into a no-provider one.
    bedrock: dry.bedrock ?? true,
    // DRY UNLESS A CASE SAYS OTHERWISE, like Bedrock: the two direct providers
    // each serve one model, and `direct-routing.unit.test.ts` has their cases.
    deepseek: dry.deepseek ?? true,
    xiaomi: dry.xiaomi ?? true,
    // DRY UNLESS A CASE SAYS OTHERWISE, like the direct providers: the Claude key serves only its eight cards.
    anthropic: dry.anthropic ?? true,
    openrouter: dry.openrouter ?? false,
  };

  /**
   How many times the holds were asked for.
   */
  const holdReads = { count: 0, };

  return {
    refused,
    statedWaits,
    holdReads,
    view,
    budgets: {
      read: async function read(): Promise<BudgetView> {
        // A read after the holds were asked for is a read after the router
        // waited, so the stub lets the test flip the view the way an expired
        // hold would.
        if (holdReads.count > 0)
          onHoldEnd?.();
        return { ...view, };
      },
      markRefused: async function markRefused(
        {
          provider,
          statedWaitMs = 0,
        }: {
          readonly provider: ProviderName;
          readonly statedWaitMs?: number;
        },
      ): Promise<void> {
        refused.push(provider,);
        statedWaits.push(statedWaitMs,);
        view[provider] = true;
      },
      holds: function holds(): ProviderRecord<number> {
        holdReads.count += 1;
        return { ...holdsMs, };
      },
      neuralwattReturn: function neuralwattReturn(): NeuralWattReturn {
        return neuralwatt;
      },
    },
  };
}

/**
 Guards a routed JSON answer.

 @param value - parsed candidate

 @returns Whether value carries a string spot

 @example
 ```ts
 isNapSpot({ spot: 'windowsill', },);
 ```
 */
function isNapSpot(value: unknown,): value is { readonly spot: string; } {
  return isJsonRecord(value,) && ((typeof value.spot) === 'string');
}

/**
 Routes one text call for Qwen3.8-27B by default, which Synthetic and Hyper
 serve and the run buys from each, and reports what happened.

 @param client - router under test

 @param modelId - model to ask

 @returns Reply text, or the error thrown

 @example
 ```ts
 const outcome = await ask({ client, },);
 ```
 */
function ask(
  {
    client,
    modelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  }: {
    readonly client: ReturnType<typeof createRoutingClient>;
    readonly modelId?: Parameters<ReturnType<typeof createRoutingClient>['chatText']>[0]['modelId'];
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
  name: createRoutingClient.name,
  children: [
    it({
      name: 'prefers the first provider while it has budget and a free slot',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        expect(await ask({ client, },),).toEqual({ text: '{"spot":"windowsill"}', },);
        expect(called,).toEqual(['synthetic',],);
      },
    },),

    it({
      name: 'sends a model Synthetic does not serve to Hyper, the next provider that does',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        await ask({
          client,
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
        },);
        expect(called,).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'moves every call to the second provider when the first is dry',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({ dry: { synthetic: true, }, },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        await ask({ client, },);
        expect(called,).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'MOVES EVERY CALL TO OPENROUTER when Synthetic and Hyper are both dry, the owner\'s '
        + 'fallback of 2026-09-03, including a model Synthetic never served',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({
          dry: {
            synthetic: true,
            bedrock: true,
            hyper: true,
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        // The one routed Synthetic seat OpenRouter also serves since GLM-5.3-Flash
        // was withheld from Synthetic (class one hundred eighteen); Bedrock
        // serves it too, so Bedrock reads dry here.
        expect(await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },),).toEqual({ text: '{"spot":"laundry basket"}', },);
        await ask({
          client,
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
        },);
        expect(called,).toEqual(['openrouter', 'openrouter',],);
      },
    },),

    it({
      name: 'FORWARDS a refused call to the next provider instead of losing it',
      fn: async () => {
        const { callers, called, } = stubProviders({ status: { synthetic: 429, }, },);
        const { budgets, refused, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        // THE WHOLE REASON ROUTING EXISTS: a pass exhausted one provider's
        // weekly credit and 866 of 875 lost voices carried this one status.
        // Retrying the exhausted provider never succeeds, and refusing to
        // settle turns a budget problem into holes in the deliverable.
        expect(await ask({ client, },),).toEqual({ text: '{"spot":"radiator"}', },);
        expect(called,).toEqual(['synthetic', 'hyper',],);
        expect(refused,).toEqual(['synthetic',],);
      },
    },),

    it({
      name: 'PASSES THE WAIT A REFUSAL NAMES to the budgets, so a provider saying "try again in 14m40s" '
        + 'is held out until then rather than for the concurrency backoff (Hyper\'s daily limit, '
        + 'Huasheng, 2026-09-07)',
      fn: async () => {
        const { callers, called, } = stubProviders({
          status: { synthetic: 429, },
          bodyText: {
            synthetic: '{"type":"error","error":{"type":"rate_limit_error","message":'
              + '"You\'ve hit your daily rate limit. Please try again in 14m40s."}}',
          },
        },);
        const { budgets, refused, statedWaits, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        expect(await ask({ client, },),).toEqual({ text: '{"spot":"radiator"}', },);
        expect(called,).toEqual(['synthetic', 'hyper',],);
        expect(refused,).toEqual(['synthetic',],);
        expect(statedWaits,).toEqual([880_000,],);
      },
    },),

    it({
      name: 'FORWARDS TWICE when the first two providers refuse in turn, reaching the third, and '
        + 'ends on the third refusal rather than looping',
      fn: async () => {
        const { callers, called, } = stubProviders({
          status: {
            synthetic: 429,
            hyper: 402,
          },
        },);
        // Bedrock serves the three-provider seat too and reads dry, so the
        // refusals walk Synthetic, Hyper and OpenRouter.
        const { budgets, refused, } = stubBudgets({ dry: { bedrock: true, }, },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        expect(await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },),).toEqual({ text: '{"spot":"laundry basket"}', },);
        expect(called,).toEqual(['synthetic', 'hyper', 'openrouter',],);
        expect(refused,).toEqual(['synthetic', 'hyper',],);

        const everyone = stubProviders({
          status: {
            synthetic: 429,
            hyper: 429,
            bedrock: 0,
            deepseek: 0,
            xiaomi: 0,
            anthropic: 0,
            openrouter: 402,
          },
        },);
        const all = createRoutingClient({
          callers: everyone.callers,
          budgets: stubBudgets({ dry: { bedrock: true, }, },).budgets,
        },);
        const outcome = await ask({
          client: all,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        // Every provider serving the seat is now out (Bedrock dry, the other
        // three refused), so the run ends rather than looping.
        expect(('thrown' in outcome) && (outcome.thrown instanceof EveryProviderDryError),).toBe(true,);
        expect(everyone.called,).toEqual(['synthetic', 'hyper', 'openrouter',],);
      },
    },),

    it({
      name: 'FORWARDS a payment refusal, which is how a balance provider says it, and ends the run '
        + 'when it was the last provider with budget',
      fn: async () => {
        const { callers, called, } = stubProviders({ status: { openrouter: 402, }, },);
        const { budgets, refused, } = stubBudgets({
          dry: {
            synthetic: true,
            bedrock: true,
            hyper: true,
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        const outcome = await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        // A subscription reports exhaustion as a rate limit and a balance
        // reports it as payment due; both mark the provider.
        expect(refused,).toEqual(['openrouter',],);
        expect(called,).toEqual(['openrouter',],);
        expect(('thrown' in outcome) && (outcome.thrown instanceof EveryProviderDryError),).toBe(true,);
      },
    },),

    it({
      name: 'WAITS OUT THE SHORTEST HOLD when every provider is held out by refusals, then routes '
        + 'to the one that came back, instead of ending the run: the pin pass of 2026-09-02 '
        + 'failed every remaining entry inside one second on two holds while both meters read wet',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const stub = stubBudgets({
          dry: {
            neuralwatt: true,
            synthetic: true,
            hyper: true,
            bedrock: true,
            deepseek: true,
            xiaomi: true,
            anthropic: true,
            openrouter: true,
          },
          holdsMs: {
            neuralwatt: 0,
            synthetic: 5,
            hyper: 20,
            bedrock: 0,
            deepseek: 0,
            xiaomi: 0,
            anthropic: 0,
            openrouter: 20,
          },
          onHoldEnd: function syntheticComesBack(): void {
            stub.view.synthetic = false;
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets: stub.budgets,
          holdPollMs: 1,
        },);

        expect(await ask({ client, },),).toEqual({ text: '{"spot":"windowsill"}', },);
        expect(called,).toEqual(['synthetic',],);
        expect(stub.holdReads.count,).toBe(1,);
      },
    },),

    it({
      name: 'GOES BACK TO THE REFUSER once its hold has been waited out when the other providers are '
        + 'dry by meter: the refusal that routed the call is what the hold became, and a hold that '
        + 'expired is the provider coming back, not a second reason to call it dry',
      fn: async () => {
        const { callers, called, } = stubProviders({
          status: { synthetic: 429, },
          refusals: { synthetic: 1, },
        },);
        const stub = stubBudgets({
          dry: {
            hyper: true,
            bedrock: true,
            openrouter: true,
          },
          holdsMs: {
            neuralwatt: 0,
            synthetic: 5,
            hyper: 0,
            bedrock: 0,
            deepseek: 0,
            xiaomi: 0,
            anthropic: 0,
            openrouter: 0,
          },
          onHoldEnd: function syntheticComesBack(): void {
            stub.view.synthetic = false;
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets: stub.budgets,
          holdPollMs: 1,
        },);

        expect(await ask({ client, },),).toEqual({ text: '{"spot":"windowsill"}', },);
        expect(called,).toEqual(['synthetic', 'synthetic',],);
        expect(stub.refused,).toEqual(['synthetic',],);
        expect(stub.holdReads.count,).toBe(1,);
      },
    },),

    it({
      name: 'ASKS A WET REFUSER AGAIN when every other provider is dry, through the real budget layer, which '
        + 'holds such a refuser for no time: the call is answered rather than ended as every provider dry '
        + '(ledger B124)',
      fn: async () => {
        const { callers, called, } = stubProviders({
          status: { synthetic: 429, },
          refusals: { synthetic: 1, },
        },);
        /**
         Budget layer over meters with Synthetic wet and the others empty;
         Bedrock has no meter, so it reads dry.
         */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic: {
            quotas: async function quotas() {
              return {
                fiveHour: {
                  limited: false,
                  remaining: 400,
                  max: 500,
                  nextTickAt: '2026-08-24T10:00:00.000Z',
                },
                weekly: {
                  percentRemaining: 62,
                  nextRegenAt: '2026-08-24T13:24:00.000Z',
                },
              };
            },
          },
          hyper: {
            credits: async function credits() {
              return { balance: 0, };
            },
          },
          openrouter: {
            credits: async function credits() {
              return {
                purchasedUsd: 10,
                usedUsd: 10,
                remainingUsd: 0,
              };
            },
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets,
          holdPollMs: 1,
        },);

        expect(await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },),).toEqual({ text: '{"spot":"windowsill"}', },);
        expect(called,).toEqual(['synthetic', 'synthetic',],);
        expect(budgets.holds(),).toEqual({
          neuralwatt: 0,
          synthetic: 0,
          bedrock: 0,
          deepseek: 0,
          xiaomi: 0,
          anthropic: 0,
          hyper: 0,
          openrouter: 0,
        },);
      },
    },),

    it({
      name: 'ENDS THE RUN when every provider reads dry with no hold to wait out, since nothing a '
        + 'wait could change is left',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, holdReads, } = stubBudgets({
          dry: {
            neuralwatt: true,
            synthetic: true,
            hyper: true,
            bedrock: true,
            deepseek: true,
            xiaomi: true,
            anthropic: true,
            openrouter: true,
          },
          // The XingZ6018 account: its day starts at 19:15:08 UTC.
          neuralwatt: {
            kind: 'at',
            msOfDay: 69_308_000,
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets,
          holdPollMs: 1,
        },);

        const outcome = await ask({ client, },);
        if (!('thrown' in outcome))
          throw new Error('every provider reading dry must end the call with a refusal',);
        expect(outcome.thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect(String(outcome.thrown,),).toBe(
          'EveryProviderDryError: Every provider is out of budget at once: NeuralWatt has no free daily energy left, '
            + 'Synthetic has no weekly credit left (or no five-hour request left, on a reply '
            + 'that still carries that window), Charm Hyper has no balance left, Amazon Bedrock has no credit left '
            + 'short of the owner\'s card, DeepSeek has no balance left (or says its account is unavailable), Xiaomi '
            + 'has refused a call for payment, the Claude key has met a billing error this run, and OpenRouter has no '
            + 'credit left. Nothing further can be bought, so '
            + 'this run ends. NeuralWatt\'s free energy returns when its day starts again, at 19:15:08 UTC, the day '
            + 'start this run decided or learned; Synthetic regenerates on its own '
            + 'schedule; Hyper, DeepSeek and OpenRouter refill on purchase; Xiaomi, once refused for payment, is '
            + 'read again only after one of its calls settles, so a top-up is seen on the next launch; '
            + 'Bedrock\'s credit is never topped up. '
            + 'Measured at the decision: meters read neuralwatt dry, synthetic dry, bedrock dry, deepseek dry, '
            + 'xiaomi dry, anthropic dry, hyper dry, openrouter dry; '
            + 'holds neuralwatt 0ms, synthetic 0ms, bedrock 0ms, deepseek 0ms, xiaomi 0ms, anthropic 0ms, hyper 0ms, openrouter 0ms.',
        );
        expect(called,).toEqual([],);
        expect(holdReads.count,).toBe(1,);
      },
    },),

    it({
      name: 'LOSES THE SEAT, NOT THE RUN, when every provider reads dry but Xiaomi, whose always-wet meter serves one '
        + 'model, and the model asked is one Xiaomi does not serve: a wet provider is left, so the refusal is the '
        + 'seat\'s and the log names the model that went dark and the providers that were dry',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({
          dry: {
            neuralwatt: true,
            synthetic: true,
            hyper: true,
            bedrock: true,
            deepseek: true,
            xiaomi: false,
            anthropic: false,
            openrouter: true,
          },
        },);
        const lines: string[] = [];
        const client = createRoutingClient({
          callers,
          budgets,
          holdPollMs: 1,
          l: levelCapturingLogger({ lines, },),
        },);

        const outcome = await ask({
          client,
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        if (!('thrown' in outcome))
          throw new Error('a model that lost its providers must be refused',);
        expect(outcome.thrown,).toBeInstanceOf(NoProviderForModelError,);
        expect(outcome.thrown,).not.toBeInstanceOf(EveryProviderDryError,);
        expect(called,).toEqual([],);
        expect(lines.filter(function wentDark(line,): boolean {
          return line.startsWith('warn ',) && line.includes(`${SEAT_SYNTHETIC_TEXT_EVERYWHERE} went dark`,)
            && line.includes('synthetic dry',) && line.includes('xiaomi wet',);
        },).length,).toBe(1,);
      },
    },),

    it({
      name: 'LOGS A DARK MODEL ONCE PER COOLDOWN and not once per refused call: a fleet with a dark model repeated the '
        + 'warn every round, so a read of the run found the same line hundreds of times',
      fn: async () => {
        const { callers, } = stubProviders({},);
        const { budgets, } = stubBudgets({
          dry: {
            neuralwatt: true,
            synthetic: true,
            hyper: true,
            bedrock: true,
            deepseek: true,
            xiaomi: false,
            openrouter: true,
          },
        },);
        const lines: string[] = [];
        const clock = { now: 0, };
        const client = createRoutingClient({
          callers,
          budgets,
          holdPollMs: 1,
          l: levelCapturingLogger({ lines, },),
          now: () => clock.now,
        },);
        /**
         Warn lines about the model going dark.
         */
        const darkLines = (): number => lines.filter(function wentDark(line,): boolean {
          return line.startsWith('warn ',) && line.includes(`${SEAT_SYNTHETIC_TEXT_EVERYWHERE} went dark`,);
        },).length;

        await ask({ client, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },);
        await ask({ client, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },);
        await ask({ client, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },);
        expect(darkLines(),).toBe(1,);
        clock.now += 300_000;
        await ask({ client, modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, },);
        expect(darkLines(),).toBe(2,);
      },
    },),

    it({
      name: 'ENDS THE RUN, with the stop line naming Xiaomi, when every provider reads dry but Xiaomi and Xiaomi '
        + 'serves only checker seats in the run\'s roster, and loses the seat as before when Xiaomi serves '
        + 'a writer seat',
      fn: async () => {
        /**
         Providers that serve a model seated in a writing bench, Xiaomi not among them.
         */
        const checkerOnly = {
          neuralwatt: true,
          synthetic: true,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: false,
          anthropic: true,
          openrouter: true,
        };
        const dry = {
          neuralwatt: true,
          synthetic: true,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: false,
          openrouter: true,
        };
        const { callers, called, } = stubProviders({},);
        const endsTheRun = await ask({
          client: createRoutingClient({
            callers,
            budgets: stubBudgets({ dry, },).budgets,
            holdPollMs: 1,
            writing: checkerOnly,
          },),
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        if (!('thrown' in endsTheRun))
          throw new Error('a run with nothing that writes must end with a refusal',);
        expect(endsTheRun.thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect((endsTheRun.thrown as Error).message,).toContain('Xiaomi reads wet but serves no seat that writes',);
        const losesTheSeat = await ask({
          client: createRoutingClient({
            callers,
            budgets: stubBudgets({ dry, },).budgets,
            holdPollMs: 1,
            writing: { ...checkerOnly, xiaomi: true, },
          },),
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        },);
        if (!('thrown' in losesTheSeat))
          throw new Error('a model that lost its providers must be refused',);
        expect(losesTheSeat.thrown,).toBeInstanceOf(NoProviderForModelError,);
        expect(called,).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES to re-route a failure that is not about budget',
      fn: async () => {
        const { callers, called, } = stubProviders({
          status: { synthetic: 500, },
          bodyText: { synthetic: 'synthetic napping', },
        },);
        const { budgets, refused, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        const outcome = await ask({ client, },);
        // Spending another provider's budget on a fault that is not about
        // budget would hide the fault and pay for it twice.
        if (!('thrown' in outcome))
          throw new Error('the failure must be thrown to the caller',);
        expect(outcome.thrown,).toBeInstanceOf(ProviderHttpError,);
        expect(String(outcome.thrown,),).toBe('ProviderHttpError: provider API returned HTTP 500: synthetic napping',);
        expect(called,).toEqual(['synthetic',],);
        expect(refused,).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES a model whose only providers are out of budget while another is wet',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({
          dry: {
            hyper: true,
            bedrock: true,
            openrouter: true,
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        const outcome = await ask({
          client,
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
        },);
        // Synthetic does not serve this model, so the budgets of the two that
        // do ARE the model's.
        if (!('thrown' in outcome))
          throw new Error('a model whose providers are all dry must be refused',);
        expect(outcome.thrown,).toBeInstanceOf(NoProviderForModelError,);
        expect(String(outcome.thrown,),).toBe(
          'NoProviderForModelError: no provider can take deepseek-v4.1-flash: every provider serving this model '
            + 'is out of budget (neuralwatt dry, deepseek dry, hyper dry, openrouter dry); the providers that do not '
            + 'serve it read synthetic wet, bedrock dry, xiaomi dry, anthropic dry',
        );
        expect(called,).toEqual([],);
      },
    },),

    it({
      name: 'sends Qwen3.8-27B pictures and text through the first provider that reads them',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        await client.chatText({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: PICTURE_MESSAGES,
          signal: SIGNAL,
        },);
        expect(called,).toEqual(['synthetic',],);

        await client.chatText({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
        },);
        expect(called,).toEqual(['synthetic', 'synthetic',],);
      },
    },),

    it({
      name: 'KEEPS a picture off a provider that does not read it for this model, which is why '
        + 'vision reach is asked per provider: gpt-oss reads nowhere and gemma reads nowhere yet',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        /**
         What a picture to a model no provider shows pictures to produces.
         */
        let thrown: unknown;
        try {
          await client.chatText({
            modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            messages: PICTURE_MESSAGES,
            signal: SIGNAL,
          },);
        } catch (error) {
          thrown = error;
        }
        expect(thrown instanceof NoProviderForModelError,).toBe(true,);
        expect(called,).toEqual([],);
      },
    },),

    it({
      name: 'overflows to the second provider while the first one is saturated',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
          slotLimits: ONE_SYNTHETIC_SLOT,
        },);

        await Promise.all([
          ask({ client, },),
          ask({ client, },),
        ],);

        // The second call finds the one slot busy and goes elsewhere rather
        // than queueing behind it.
        expect(called.toSorted(),).toEqual(['hyper', 'synthetic',],);
      },
    },),

    it({
      name: 'KEEPS EVERY CONCURRENT CALL ON SYNTHETIC under the default slot limits, since Synthetic states no per-model ceiling '
        + 'and so never reads saturated',
      fn: async () => {
        /** Providers asked, in call order. */
        const called: ProviderName[] = [];
        /** First provider holding every admitted slot. */
        const synthetic = slotHoldingSynthetic({ called, },);
        /** Overflow provider answering without a local concurrency ceiling. */
        const hyper = {
          chatText: async function chatText() {
            called.push('hyper',);
            return { text: '{"spot":"windowsill"}', };
          },
        };
        /** Third provider, never reached while Hyper is wet. */
        const openrouter = {
          chatText: async function chatText() {
            called.push('openrouter',);
            return { text: '{"spot":"windowsill"}', };
          },
        };
        /** Fourth provider, serving none of these models. */
        const bedrock = {
          chatText: async function chatText() {
            called.push('bedrock',);
            return { text: '{"spot":"windowsill"}', };
          },
        };
        const { budgets, } = stubBudgets({},);
        /** Router using production default slot count. */
        const client = createRoutingClient({
          callers: {
            synthetic,
            hyper,
            bedrock,
            openrouter,
          },
          budgets,
        },);
        /** Many concurrent calls on one model, which Synthetic once capped at five. */
        const calls = Array.from(
          { length: CONCURRENT_SYNTHETIC_CALLS, },
          function callModel() {
            return ask({ client, },);
          },
        );

        await Promise.all(calls,);

        expect(called.filter(function syntheticCall(provider,) {
          return provider === 'synthetic';
        },),).toHaveLength(CONCURRENT_SYNTHETIC_CALLS,);
        expect(called.filter(function hyperCall(provider,) {
          return provider === 'hyper';
        },),).toHaveLength(0,);
      },
    },),

    it({
      name: 'FORWARDS a non-conformant answer to the same model on the next stack',
      fn: async () => {
        const { callers, called, } = stubProviders({ text: { synthetic: 'I will not do that.', }, },);
        const { budgets, refused, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: isNapSpot,
        },);

        // The providers extract structure by different mechanisms, a forced
        // tool on one and a response format on the others, so the same
        // weights can conform on one stack and not another.
        expect(called,).toEqual(['synthetic', 'hyper',],);
        expect(outcome.kind,).toBe('ok',);
        // A bad ANSWER is not a budget refusal; nothing is marked.
        expect(refused,).toEqual([],);
      },
    },),

    it({
      name: 'RE-ASKS ON OPENROUTER when Hyper is dry, since the next wet provider serving the model '
        + 'is the second opinion, wherever it sits in the order',
      fn: async () => {
        const { callers, called, } = stubProviders({ text: { synthetic: 'I will not do that.', }, },);
        const { budgets, } = stubBudgets({
          dry: {
            bedrock: true,
            hyper: true,
          },
        },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: isNapSpot,
        },);

        expect(called,).toEqual(['synthetic', 'openrouter',],);
        expect(outcome.kind,).toBe('ok',);
      },
    },),

    it({
      name: 'PAIRS the re-ask\'s slot release with a take, so the Synthetic count does not drift '
        + 'negative and overflow to Hyper keeps working afterwards',
      fn: async () => {
        /**
         Providers asked, in call order.
         */
        const called: ProviderName[] = [];
        /**
         First provider, holding its one slot long enough for a concurrent
         caller to find it busy.
         */
        const synthetic = slotHoldingSynthetic({ called, },);
        /**
         Second provider, answering at once and unparseably, so the caller
         re-asks the first.
         */
        const hyper = {
          chatText: async function chatText() {
            called.push('hyper',);
            return { text: 'I will not do that.', };
          },
        };
        /** Third provider, dry throughout. */
        const openrouter = {
          chatText: async function chatText() {
            called.push('openrouter',);
            return { text: '{"spot":"windowsill"}', };
          },
        };
        /** Fourth provider, serving none of these models. */
        const bedrock = {
          chatText: async function chatText() {
            called.push('bedrock',);
            return { text: '{"spot":"windowsill"}', };
          },
        };
        const { budgets, } = stubBudgets({ dry: { openrouter: true, }, },);
        /** Router under test, one Synthetic slot per model. */
        const client = createRoutingClient({
          callers: {
            synthetic,
            hyper,
            bedrock,
            openrouter,
          },
          budgets,
          slotLimits: ONE_SYNTHETIC_SLOT,
        },);

        // Phase one: the first call holds the slot, the second finds it busy,
        // goes to Hyper, gets nothing usable, and re-asks Synthetic. Before the
        // fix that re-ask released a slot nothing had taken.
        await Promise.all([
          client.chatJson({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            messages: MESSAGES,
            signal: SIGNAL,
            validate: isNapSpot,
          },),
          client.chatJson({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            messages: MESSAGES,
            signal: SIGNAL,
            validate: isNapSpot,
          },),
        ],);
        expect(called,).toEqual(['synthetic', 'hyper', 'synthetic',],);
        called.splice(0,);

        // Phase two: with every slot handed back, one concurrent call takes
        // the slot and the other overflows. A count that had drifted to minus
        // one would show both a free slot and put both on Synthetic.
        await Promise.all([
          ask({ client, },),
          ask({ client, },),
        ],);
        expect(called.toSorted(),).toEqual(['hyper', 'synthetic',],);
      },
    },),

    it({
      name: 'REFUSES to re-ask when no other provider serving the model has budget',
      fn: async () => {
        const { callers, called, } = stubProviders({ text: { hyper: 'I will not do that.', }, },);
        const { budgets, } = stubBudgets({ dry: { openrouter: true, }, },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const outcome = await client.chatJson({
          // Synthetic never served this model and OpenRouter is dry, so the
          // answer falls to the invalid-candidate path.
          modelId: SEAT_HYPER_ONLY,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: isNapSpot,
        },);

        expect(called,).toEqual(['hyper',],);
        expect(outcome.kind,).toBe('schema-mismatch',);
      },
    },),

    it({
      name: 'KEEPS the first answer and starts the next provider\'s cooldown when the re-ask is refused on '
        + 'budget, instead of raising out of an exchange that already has an answer',
      fn: async () => {
        const { callers, called, } = stubProviders({
          text: { synthetic: 'I will not do that.', },
          status: { hyper: 429, },
        },);
        const { budgets, refused, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: isNapSpot,
        },);

        expect(outcome.kind,).not.toBe('ok',);
        expect(called,).toEqual(['synthetic', 'hyper',],);
        expect(refused,).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'keeps the preferred provider\'s answer when the re-ask fails too',
      fn: async () => {
        const { callers, called, } = stubProviders({
          text: {
            synthetic: 'first refusal',
            hyper: 'second refusal',
          },
        },);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: isNapSpot,
        },);

        expect(called,).toEqual(['synthetic', 'hyper',],);
        // The preferred provider's answer is the one the caller's own handling
        // is written against.
        expect(outcome.rawText,).toBe('first refusal',);
      },
    },),

    it({
      name: 'TAGS a routed reply with the provider that served it, and serves a hinted re-ask on another '
        + 'provider, which is what lets the uniqueness wrapper re-ask elsewhere (ledger P9)',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const plain = await client.chatText({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
        },);
        const elsewhere = await client.chatText({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          otherThan: 'synthetic',
        },);
        expect(plain.servedBy,).toBe('synthetic',);
        expect(elsewhere.servedBy,).toBe('hyper',);
        expect(called,).toEqual(['synthetic', 'hyper',],);
      },
    },),

    it({
      name: 'REFUSES a hinted re-ask where no other provider serving the model has budget, rather than '
        + 'serving it on the provider it was hinted away from',
      fn: async () => {
        const { callers, called, } = stubProviders({},);
        const { budgets, } = stubBudgets({ dry: { openrouter: true, }, },);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        /**
         What the hinted re-ask threw, taken as data.
         */
        let refusal: unknown;
        try {
          await client.chatText({
            modelId: SEAT_HYPER_ONLY,
            messages: MESSAGES,
            signal: SIGNAL,
            otherThan: 'hyper',
          },);
        }
        catch (error) {
          refusal = error;
        }
        expect(refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(String(refusal,),).toBe(
          'NoProviderForModelError: no provider can take glm-5.3: no provider other than hyper serves it with budget',
        );
        expect(called,).toEqual([],);
      },
    },),

    it({
      name: 'reads a routed answer through the same ladder every provider uses',
      fn: async () => {
        const { callers, } = stubProviders({ status: { synthetic: 429, }, },);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: isNapSpot,
        },);

        expect(outcome.kind,).toBe('ok',);
        expect(
          outcome.kind === 'ok' ? outcome.value : undefined,
        ).toEqual({ spot: 'radiator', },);
      },
    },),

    it({
      name: 'TAKES THE SCHEMA\'S SPELLING of a routed answer that differs from a listed value only in its capital, '
        + 'so the guard reads the voice and no second provider is asked',
      fn: async () => {
        const { callers, called, } = stubProviders({ text: { synthetic: '{"spot":"Windowsill"}', }, },);
        const { budgets, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          responseFormat: {
            type: 'json_schema',
            json_schema: {
              name: 'nap_spot',
              schema: {
                type: 'object',
                properties: { spot: { type: 'string', enum: ['windowsill', 'radiator',], }, },
              },
            },
          },
          validate: function isListedSpot(value: unknown,): value is { readonly spot: string; } {
            return isJsonRecord(value,) && ((value.spot === 'windowsill') || (value.spot === 'radiator'));
          },
        },);

        expect(called,).toEqual(['synthetic',],);
        expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ spot: 'windowsill', },);
      },
    },),

    it({
      name: 'RERAISES the last refusal once every provider serving the model has refused on budget, '
        + 'rather than returning a silence no caller can read',
      fn: async () => {
        /**
         Both providers serving the model refuse on budget, each saying its own words.
         */
        const { callers, called, } = stubProviders({
          status: { synthetic: 429, hyper: 429, },
          bodyText: { synthetic: 'synthetic napping', hyper: 'hyper napping', },
        },);
        const { budgets, refused, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        const outcome = await ask({ client, },);
        expect(called,).toEqual(['synthetic', 'hyper',],);
        expect(refused,).toEqual(['synthetic', 'hyper',],);
        if (!('thrown' in outcome))
          throw new Error('both providers refused, so the call must throw the last refusal',);
        expect(outcome.thrown,).toBeInstanceOf(ProviderHttpError,);
        expect(String(outcome.thrown,),).toBe('ProviderHttpError: provider API returned HTTP 429: hyper napping',);
      },
    },),

    it({
      name: 'RETHROWS a failure that is no budget refusal at once, asking no other provider',
      fn: async () => {
        /**
         Every provider answering an upstream failure.
         */
        const { callers, called, } = stubProviders({
          status: { synthetic: 503, hyper: 503, openrouter: 503, },
          bodyText: { synthetic: 'synthetic napping', hyper: 'hyper napping', },
        },);
        const { budgets, refused, } = stubBudgets({},);
        const client = createRoutingClient({
          callers,
          budgets,
        },);

        const outcome = await ask({ client, },);
        expect(called,).toEqual(['synthetic',],);
        expect(refused,).toEqual([],);
        if (!('thrown' in outcome))
          throw new Error('the failure must be thrown to the caller',);
        expect(outcome.thrown,).toBeInstanceOf(ProviderHttpError,);
        expect(String(outcome.thrown,),).toBe('ProviderHttpError: provider API returned HTTP 503: synthetic napping',);
      },
    },),
  ],
},);
