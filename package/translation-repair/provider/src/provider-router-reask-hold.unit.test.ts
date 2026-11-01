/**
 Tests that a re-ask of the same model on another provider honours the holds the run applies at seating time (ledger
 B1350): Qwen3.8-27B seated through Synthetic that runs away there is not re-asked on Hyper, the provider the hold
 `hyper-slow` exists to keep it off, and a model without that hold is.

 The callers are stubs and no HTTP is made; the budgets read Synthetic and Hyper wet, so the only thing that can keep
 the re-ask off Hyper is the hold. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NoProviderForModelError,
  refusedBeforeAnyExchange,
} from '@monochromatic-dev/translation-repair-markup';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  createSlotLedger,
  type ProviderName,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  SYNTHETIC_WET_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  alternateModelsFor,
  type ChatTextReply,
  createRoutingClient,
  type ReaskSeat,
  type RoutedCore,
  routedTextElsewhere,
} from '../dist/final/node/index.mjs';
import { routerRecordingProviders, } from './provider-router-recording.test-fixture.ts';
import {
  holdNothing,
  markNothingRefused,
  readOlderProvidersWet,
} from './provider-router-wet-budgets.test-fixture.ts';

/**
 Abort signal every routed call carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Conversation reused across the re-asks.
 */
const MESSAGES = [{ role: 'user' as const, content: '这只猫睡在哪里？', },];

/**
 Re-asks a model of the provider that served it first, as the nudged re-ask does, and says where it went.

 @param modelId - model to re-ask
 @param reaskSeat - seat the request names, when it names one

 @returns Provider that served the re-ask, or the refusal the router gave it

 @example
 ```ts
 const came = await reaskedElsewhere({ modelId, reaskSeat: { role: 'editor', bench: [], reask: 'other-model', }, },);
 ```
 */
async function reaskedElsewhere(
  { modelId, reaskSeat, }: { readonly modelId: RosterModelId; readonly reaskSeat?: ReaskSeat; },
): Promise<{ readonly servedBy?: ProviderName; readonly refusal?: unknown; readonly reached: readonly ProviderName[]; }> {
  const { client, reached, } = routerRecordingProviders({ read: readOlderProvidersWet, },);
  try {
    const reply = await client.chatText({
      modelId,
      messages: MESSAGES,
      signal: SIGNAL,
      otherThan: ['synthetic',],
      ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
    },);
    return { ...((reply.servedBy === undefined) ? {} : { servedBy: reply.servedBy, }), reached, };
  }
  catch (error) {
    return { refusal: error, reached, };
  }
}

/**
 Slots every provider grants, so the ledger of a case that is no test of the slots never saturates.
 */
const UNLIMITED_SLOTS = {
  neuralwatt: Number.POSITIVE_INFINITY,
  synthetic: Number.POSITIVE_INFINITY,
  hyper: Number.POSITIVE_INFINITY,
  bedrock: Number.POSITIVE_INFINITY,
  deepseek: Number.POSITIVE_INFINITY,
  xiaomi: Number.POSITIVE_INFINITY,
  anthropic: Number.POSITIVE_INFINITY,
  openrouter: Number.POSITIVE_INFINITY,
} as const;

/**
 Caller that is never reached: any call here fails the case that built it.

 @returns Nothing, since it always rejects
 */
function refusedNowhere(): Promise<ChatTextReply> {
  return Promise.reject(new Error('this provider must not be asked',),);
}

await describe({
  name: 'a re-ask of the same model on another provider honours the holds',
  children: [
    it({
      name: 'REFUSES A HINTED RE-ASK OF A MODEL HELD OFF HYPER, so a hyper-slow model seated through Synthetic that ran '
        + 'away there is not re-asked on Hyper (ledger B1350): the editor and the refiner seats',
      fn: async () => {
        /**
         What the re-ask of a runaway editor and of a runaway refiner came to.
         */
        const [editor, refiner,] = await Promise.all([
          reaskedElsewhere({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
          },),
          reaskedElsewhere({
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: { role: 'refiner', bench: [], reask: 'other-model', },
          },),
        ],);
        expect(editor?.refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(editor?.reached,).toEqual([],);
        expect(refiner?.refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(refiner?.reached,).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A HINTED RE-ASK THAT NAMES NO SEAT of a model held off Hyper, since the hold every role honours '
        + 'needs no role (ledger B1350)',
      fn: async () => {
        const came = await reaskedElsewhere({ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },);
        expect(came.refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(came.reached,).toEqual([],);
      },
    },),

    it({
      name: 'KEEPS THE FIRST ANSWER OF A HELD EDITOR WHOSE REPLY COULD NOT BE USED rather than re-asking it on Hyper '
        + '(ledger B1350): the whole nudged re-ask of the same model, through the router\'s own JSON exchange, reaches '
        + 'Synthetic once',
      fn: async () => {
        const { client, reached, } = routerRecordingProviders({ read: readOlderProvidersWet, },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: function acceptsNothingParsed(value: unknown,): value is never {
            // A parsed reply is never a symbol, so no reply of the stubs is usable.
            return (typeof value) === 'symbol';
          },
          reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
        },);
        expect(reached,).toEqual(['synthetic',],);
        // THE ANSWER KEPT IS THE FIRST ONE (ledger B1377): the text Synthetic gave, whole, and not another outcome that
        // is merely not ok, which a refusal that threw the first answer away would also be.
        /**
         The text a plain first ask of the same model is answered with, the stub's reply of Synthetic.
         */
        const { client: control, } = routerRecordingProviders({ read: readOlderProvidersWet, },);
        const firstText = (await control.chatText({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          messages: MESSAGES,
          signal: SIGNAL,
          reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
        },)).text;
        expect(outcome.kind,).toBe('schema-mismatch',);
        expect(outcome.kind === 'schema-mismatch' ? outcome.rawText : undefined,).toBe(firstText,);
      },
    },),

    it({
      name: 'SERVES THE HINTED RE-ASK OF A MEASUREMENT ON HYPER whatever the hold (ledger B1350, B1141): a measurement '
        + 'scores the model on the provider that carries it, and the same model refused as an editor is served',
      fn: async () => {
        const measured = await reaskedElsewhere({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: { role: 'editor', bench: [], reask: 'measured-model', },
        },);
        const seated = await reaskedElsewhere({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
        },);
        expect(measured.servedBy,).toBe('hyper',);
        expect(seated.refusal,).toBeInstanceOf(NoProviderForModelError,);
      },
    },),

    it({
      name: 'SKIPS THE HELD PROVIDER AND TAKES THE NEXT ONE THAT SERVES THE MODEL (ledger B1377): with a third provider '
        + 'serving Qwen3.8-27B, the editor re-ask after Synthetic goes to the third and not to Hyper, and is not refused, '
        + 'which a router that refuses whenever any provider is held would fail',
      fn: async () => {
        /**
         Providers the stub core called, in order.
         */
        const called: ProviderName[] = [];
        /**
         The router's lent core, with a model that Synthetic, Hyper and OpenRouter all serve, all with budget, since no
         roster card with a hold has a third provider.
         */
        const core: RoutedCore = {
          reachFor: () => ({
            neuralwatt: false,
            synthetic: true,
            hyper: true,
            bedrock: false,
            deepseek: false,
            xiaomi: false,
            anthropic: false,
            openrouter: true,
          }),
          budgets: {
            read: readOlderProvidersWet,
            markRefused: markNothingRefused,
            holds: holdNothing,
          },
          ledger: createSlotLedger({ limits: UNLIMITED_SLOTS, },),
          callOn: function callOn({ provider, },): Promise<ChatTextReply> {
            called.push(provider,);
            return Promise.resolve({ text: `${provider} answers`, },);
          },
          routedText: function routedTextUnused(): never {
            throw new Error('the first ask is not routed here',);
          },
        };
        const served = await routedTextElsewhere({
          core,
          request: {
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            messages: MESSAGES,
            signal: SIGNAL,
            reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
          },
          otherThan: ['synthetic',],
        },);
        expect(served.provider,).toBe('openrouter',);
        expect(called,).toEqual(['openrouter',],);
      },
    },),

    it({
      name: 'HONOURS THE HINT AND THE HOLDS IN A JSON EXCHANGE AS IN A TEXT EXCHANGE (ledger B1379): a chatJson of a held '
        + 'model that names no seat and is hinted off Synthetic is refused and reaches no provider, and the same request '
        + 'as a measurement is served on Hyper',
      fn: async () => {
        /**
         Asks a JSON exchange hinted off Synthetic, as the router's own text exchange is by the nudged re-ask.

         @param reaskSeat - seat the request names, when it names one

         @returns Providers the exchange reached first and the refusal it was given, if any
         */
        async function jsonHintedOffSynthetic(
          reaskSeat?: ReaskSeat,
        ): Promise<{ readonly reached: readonly ProviderName[]; readonly refusal?: unknown; }> {
          const { client, reached, } = routerRecordingProviders({ read: readOlderProvidersWet, },);
          try {
            await client.chatJson({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              messages: MESSAGES,
              signal: SIGNAL,
              otherThan: ['synthetic',],
              validate: function acceptsNothingParsed(value: unknown,): value is never {
                return (typeof value) === 'symbol';
              },
              ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
            },);
            return { reached, };
          }
          catch (error) {
            return { reached, refusal: error, };
          }
        }
        const unseated = await jsonHintedOffSynthetic();
        const measured = await jsonHintedOffSynthetic({ role: 'editor', bench: [], reask: 'measured-model', },);
        expect(unseated.refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(unseated.reached,).toEqual([],);
        expect(measured.reached.at(0,),).toBe('hyper',);
      },
    },),

    it({
      name: 'KEEPS THE PROVIDER A HINTED JSON EXCHANGE EXCLUDED OUT OF ITS SCHEMA-MISMATCH RE-ASK (ledger B1410): Kimi-K3 '
        + 'hinted off Synthetic is answered by Hyper with text that is no JSON, and the re-ask has nowhere to go, so the '
        + 'sequence of providers reached is Hyper alone',
      fn: async () => {
        /**
         Asks a JSON exchange of Kimi-K3 as an editor, hinted off the providers given, and says every provider it reached.

         @param hinted - whether the request carries the hint off Synthetic

         @returns Providers reached, in order
         */
        async function reachedByJson(hinted: boolean,): Promise<readonly ProviderName[]> {
          const { client, reached, } = routerRecordingProviders({ read: readOlderProvidersWet, },);
          await client.chatJson({
            modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
            messages: MESSAGES,
            signal: SIGNAL,
            validate: function acceptsNothingParsed(value: unknown,): value is never {
              return (typeof value) === 'symbol';
            },
            reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
            ...(hinted ? { otherThan: ['synthetic',] as const, } : {}),
          },);
          return reached;
        }
        // THE POSITIVE CONTROL: with no hint the exchange is routed to Synthetic and the schema-mismatch re-ask goes to
        // Hyper, so the probe reaches the retry and can see which provider it asks.
        expect(await reachedByJson(false,),).toEqual(['synthetic', 'hyper',],);
        expect(await reachedByJson(true,),).toEqual(['hyper',],);
      },
    },),

    it({
      name: 'ASKS ANOTHER MODEL OF A RUNAWAY THROUGH THE ROUTER FROM THE POOL OF NO READING (ledger B1410): routedJson '
        + 'hands the re-ask no reading of the meters, so a model with a hold at seating time is left out of the pool '
        + 'even where Synthetic serves it, and the model asked is routed as a first ask',
      fn: async () => {
        /**
         Models the stub providers were asked, with the provider each call reached, in order.
         */
        const calls: (readonly [RosterModelId, ProviderName])[] = [];
        /**
         Caller of one provider: the model that ran away answers with a length stop and no content the first time it is
         asked, and every call after answers with JSON.

         @param provider - provider this caller stands for

         @returns Caller that records the call
         */
        function callerOf(provider: ProviderName,): { readonly chatText: (request: { readonly modelId: RosterModelId; },) => Promise<ChatTextReply>; } {
          return {
            chatText: function answers(request,): Promise<ChatTextReply> {
              calls.push([request.modelId, provider,],);
              return Promise.resolve((calls.length === 1) ? { text: '', finishReason: 'length', } : { text: '{"spot":"radiator"}', },);
            },
          };
        }
        const client = createRoutingClient({
          callers: {
            neuralwatt: callerOf('neuralwatt',),
            synthetic: callerOf('synthetic',),
            hyper: callerOf('hyper',),
            openrouter: callerOf('openrouter',),
          },
          budgets: { read: readOlderProvidersWet, markRefused: markNothingRefused, holds: holdNothing, },
          l: levelCapturingLogger({ lines: [], },),
        },);
        /**
         The pool the re-ask reads: no reading, so every hold applied at seating time counts.
         */
        const pool = alternateModelsFor({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, role: 'select-judge', },);
        const outcome = await client.chatJson({
          modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          messages: MESSAGES,
          signal: SIGNAL,
          validate: function isCat(value: unknown,): value is { readonly spot: string; } {
            return ((typeof value) === 'object') && (value !== null) && ('spot' in value);
          },
          reaskSeat: { role: 'select-judge', bench: [], reask: 'other-model', },
        },);
        /**
         Models asked after the one that ran away.
         */
        const askedAfter = calls.slice(1,).map(function modelOf([modelId,],): RosterModelId {
          return modelId;
        },);
        expect(calls.at(0,)?.[0],).toBe(SEAT_SYNTHETIC_TEXT_EVERYWHERE,);
        expect(askedAfter.length,).toBeGreaterThan(0,);
        for (const modelId of askedAfter)
          expect(pool.alternates.includes(modelId,),).toBe(true,);
        expect(askedAfter.includes(SEAT_SYNTHETIC_TEXT_EVERYWHERE,),).toBe(false,);
        // KIMI-K3 IS HELD FOR THE SELECT SEATS, so the pool without a reading leaves it out although Synthetic serves it
        // (the positive control of the case: the probe sees a model the pool kept out), and it was never asked.
        expect(pool.leftOut.some(function isKimi(left,): boolean {
          return left.modelId === SEAT_SYNTHETIC_VISION_WITHHELD;
        },),).toBe(true,);
        expect(askedAfter.includes(SEAT_SYNTHETIC_VISION_WITHHELD,),).toBe(false,);
        expect(outcome.kind,).toBe('ok',);
      },
    },),

    it({
      name: 'MARKS THE REFUSAL OF A RE-ASK THAT REACHED A PROVIDER AS RAISED AFTER AN EXCHANGE (ledger B1380): Kimi-K3 '
        + 're-asked off Hyper, Synthetic the one provider left, answers 429; the call was made, so a reader that gives a '
        + 'claim back only for a refusal before any exchange keeps it, and a re-ask that found nowhere to go does not',
      fn: async () => {
        /**
         Router whose Synthetic refuses on budget and whose other callers answer.
         */
        const client = createRoutingClient({
          callers: {
            neuralwatt: { chatText: refusedNowhere, },
            synthetic: {
              chatText: function refuses(): Promise<ChatTextReply> {
                return Promise.reject(new ProviderHttpError({ provider: 'synthetic', status: 429, bodyText: 'refused', },),);
              },
            },
            hyper: { chatText: refusedNowhere, },
            openrouter: { chatText: refusedNowhere, },
          },
          budgets: {
            read: async function readSyntheticWet() {
              return SYNTHETIC_WET_ONLY;
            },
            markRefused: markNothingRefused,
            holds: holdNothing,
          },
          l: levelCapturingLogger({ lines: [], },),
        },);
        /**
         What a re-ask of Kimi-K3 off a provider came to.

         @param otherThan - provider the hint rules out

         @returns Refusal thrown, or undefined when the exchange was served
         */
        async function reaskedOff(otherThan: ProviderName,): Promise<unknown> {
          try {
            await client.chatText({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              messages: MESSAGES,
              signal: SIGNAL,
              otherThan: [otherThan,],
              reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
            },);
            return undefined;
          }
          catch (error) {
            return error;
          }
        }
        const reached = await reaskedOff('hyper',);
        const nowhere = await reaskedOff('synthetic',);
        expect(reached,).toBeInstanceOf(NoProviderForModelError,);
        expect(nowhere,).toBeInstanceOf(NoProviderForModelError,);
        expect(reached instanceof NoProviderForModelError ? reached.afterExchange : undefined,).toBe(true,);
        expect(refusedBeforeAnyExchange({ error: reached, },),).toBe(false,);
        expect(refusedBeforeAnyExchange({ error: nowhere, },),).toBe(true,);
      },
    },),

    it({
      name: 'SERVES A HINTED RE-ASK ON HYPER WHERE THE ROLE DOES NOT HONOUR THE HOLD OF THE MODEL, and refuses it where '
        + 'it does (ledger B1350): Kimi-K3 carries the hold of the select seats only, so its editor re-ask reaches '
        + 'Hyper and its select-judge re-ask does not, which shows the probe can see the difference',
      fn: async () => {
        const editor = await reaskedElsewhere({
          modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
          reaskSeat: { role: 'editor', bench: [], reask: 'other-model', },
        },);
        const select = await reaskedElsewhere({
          modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
          reaskSeat: { role: 'select-judge', bench: [], reask: 'other-model', },
        },);
        expect(editor.servedBy,).toBe('hyper',);
        expect(select.refusal,).toBeInstanceOf(NoProviderForModelError,);
        expect(select.reached,).toEqual([],);
      },
    },),
  ],
},);
