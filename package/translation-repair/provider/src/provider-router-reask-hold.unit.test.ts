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

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  type ChatTextReply,
  createRoutingClient,
  type ReaskSeat,
} from '../dist/final/node/index.mjs';
import {
  holdNothing,
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
 Builds a router over stub callers that record which provider each call reached.

 @returns Router and the providers its calls reached, in order

 @example
 ```ts
 const { client, reached, } = routerRecordingProviders();
 ```
 */
function routerRecordingProviders(): {
  readonly client: ReturnType<typeof createRoutingClient>;
  readonly reached: ProviderName[];
} {
  /**
   Providers the calls reached.
   */
  const reached: ProviderName[] = [];

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records its provider and answers
   */
  function callerFor(provider: ProviderName,): { readonly chatText: () => Promise<ChatTextReply>; } {
    return {
      chatText: async function chatText(): Promise<ChatTextReply> {
        reached.push(provider,);
        return { text: `${provider} answers`, };
      },
    };
  }
  return {
    reached,
    client: createRoutingClient({
      callers: {
        neuralwatt: callerFor('neuralwatt',),
        synthetic: callerFor('synthetic',),
        hyper: callerFor('hyper',),
        openrouter: callerFor('openrouter',),
      },
      budgets: {
        read: readOlderProvidersWet,
        markRefused: async function markNothingRefused(): Promise<void> {
          return undefined;
        },
        holds: holdNothing,
      },
      l: levelCapturingLogger({ lines: [], },),
    },),
  };
}

/**
 Re-asks a model of the provider that served it first, as the nudged re-ask does, and says where it went.

 @param modelId - model to re-ask
 @param reaskSeat - seat the request names, when it names one

 @returns The provider that served the re-ask, or the refusal the router gave it

 @example
 ```ts
 const came = await reaskedElsewhere({ modelId, reaskSeat: { role: 'editor', bench: [], reask: 'other-model', }, },);
 ```
 */
async function reaskedElsewhere(
  { modelId, reaskSeat, }: { readonly modelId: RosterModelId; readonly reaskSeat?: ReaskSeat; },
): Promise<{ readonly servedBy?: ProviderName; readonly refusal?: unknown; readonly reached: readonly ProviderName[]; }> {
  const { client, reached, } = routerRecordingProviders();
  try {
    const reply = await client.chatText({
      modelId,
      messages: MESSAGES,
      signal: SIGNAL,
      otherThan: 'synthetic',
      ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
    },);
    return { ...((reply.servedBy === undefined) ? {} : { servedBy: reply.servedBy, }), reached, };
  }
  catch (error) {
    return { refusal: error, reached, };
  }
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
        const { client, reached, } = routerRecordingProviders();
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
        expect(outcome.kind,).not.toBe('ok',);
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
