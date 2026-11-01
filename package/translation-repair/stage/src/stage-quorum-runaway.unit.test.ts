/**
 Tests for a gather whose seat ran away reasoning (ledger B1073, B1074, B1076): the gather names its bench and its
 role on every ask, and the recovery round asks the seat's question of another model, never of the seat's own, so
 the run of `XingZ6020` slice 92 (four runaways of one model for one question) cannot repeat through the round.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
  ReaskMode,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import {
  gatherStageVoices,
  reachableQuorum,
} from '../dist/final/node/index.mjs';
import { HYPER_WET_ONLY, } from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Quorum runaway tests

/**
 The bench of the gather: the seat that runs away and the two that answer.
 */
const BENCH: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
];

/**
 One ask the scripted client saw.
 */
type SeenAsk = {
  /**
   Model the ask was addressed to.
   */
  readonly modelId: RosterModelId;

  /**
   Contents of the ask's messages, in order.
   */
  readonly contents: readonly string[];

  /**
   Seat the ask carried, when it carried one.
   */
  readonly reaskSeat?: ReaskSeat;
};

/**
 Runs one gather over the bench, the first seat answering with a length stop with no content.

 @param role - role the gather names, or none

 @returns Gather and the asks the client saw

 @example
 ```ts
 const { gather, seen, } = await gatherWithARunaway({ role: 'wide-judge', },);
 ```
 */
async function gatherWithARunaway(
  {
    role,
    reask = 'other-model',
  }: {
    readonly role: 'roster' | 'wide-judge';
    readonly reask?: ReaskMode;
  },
) {
  /**
   Asks the client saw.
   */
  const seen: SeenAsk[] = [];

  /**
   Client whose first bench seat runs away and whose every other model answers.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      seen.push({
        modelId: request.modelId,
        contents: request.messages.map(function contentOf(message,): string {
          return ((typeof message.content) === 'string') ? message.content : '';
        },),
        ...((request.reaskSeat === undefined) ? {} : { reaskSeat: request.reaskSeat, }),
      },);
      if (request.modelId === SEAT_HYPER_OPENROUTER_UNMEASURED) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          reason: 'truncated-completion',
          detail: 'provider reported a truncating completion',
        };
      }

      /**
       Scripted payload for the answering call, naming the model that wrote it.
       */
      const scripted: unknown = { meow: request.modelId, };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
  return {
    seen,
    gather: await gatherStageVoices({
      client,
      modelIds: BENCH,
      messages: [{ role: 'user', content: 'meow', },],
      signal: new AbortController().signal,
      exchangeTimeoutMs: HANG_STOP_MS,
      responseFormat: MEOW_FORMAT,
      validate: isMeowReply,
      stage: 'panel',
      l: levelCapturingLogger({ lines: [], },),
      graceMs: HANG_STOP_MS,
      fanOut: 'whole-bench',
      role,
      reask,
    },),
  };
}

/**
 Runs one gather over the whole roster in an editor role, where the seat of Qwen3.8-27B answers with a length stop with
 no content, the seats listed have no provider, and the client's reading has Hyper serving what Synthetic does not.

 No model is left off the bench for the recovery round to ask instead, so the seat's own model is the one it would ask,
 and the card of that model holds it off Hyper for the role (ledger B1371, B1372).

 @param refused - seats the router refuses for want of a provider with budget

 @returns Gather and the models the client was asked, in order

 @example
 ```ts
 const { gather, seen, } = await gatherWithALostRecovery({ refused: [], },);
 ```
 */
async function gatherWithALostRecovery(
  { refused, }: { readonly refused: readonly RosterModelId[]; },
) {
  /**
   Models the client was asked, in order.
   */
  const seen: RosterModelId[] = [];

  /**
   Client whose Qwen3.8-27B seat runs away, whose listed seats have no provider and whose reading has only Hyper wet.
   */
  const client: ModelClient = {
    providerDryness: () => Promise.resolve(HYPER_WET_ONLY,),
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      seen.push(request.modelId,);
      if (refused.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }
      if (request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          reason: 'truncated-completion',
          detail: 'provider reported a truncating completion',
        };
      }
      /**
       Scripted payload for the answering call, naming the model that wrote it.
       */
      const scripted: unknown = { meow: request.modelId, };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
  return {
    seen,
    gather: await gatherStageVoices({
      client,
      modelIds: ROSTER_MODEL_IDS,
      messages: [{ role: 'user', content: 'meow', },],
      signal: new AbortController().signal,
      exchangeTimeoutMs: HANG_STOP_MS,
      responseFormat: MEOW_FORMAT,
      validate: isMeowReply,
      stage: 'panel',
      l: levelCapturingLogger({ lines: [], },),
      graceMs: HANG_STOP_MS,
      fanOut: 'whole-bench',
      role: 'editor',
      reask: 'other-model',
    },),
  };
}

await describe({
  name: `${gatherStageVoices.name} (a reasoning runaway)`,
  concurrency: 1,
  children: [
    it({
      name: 'NAMES ITS BENCH AND THE ROLE THE CALLER GAVE ON EVERY ASK of the first round, so the re-ask of a runaway can '
        + 'leave the bench out and honour the holds of the role',
      fn: async () => {
        for (const role of ['wide-judge', 'roster',] as const) {
          // oxlint-disable-next-line no-await-in-loop -- one gather at a time, so each role's asks stay apart
          const named = await gatherWithARunaway({ role, },);
          for (const ask of named.seen.slice(0, BENCH.length,)) {
            expect(ask.reaskSeat?.role,).toBe(role,);
            expect(ask.reaskSeat?.bench,).toEqual(BENCH,);
          }
        }
      },
    },),

    it({
      name: 'NAMES THE MODE THE CALLER GAVE ON EVERY ASK and asks no model off the bench of a measurement: the runaway '
        + 'seat is asked its own question again, nudged, and stays unreadable, since a rival\'s answer would be scored as '
        + 'its own',
      fn: async () => {
        const { gather, seen, } = await gatherWithARunaway({ role: 'wide-judge', reask: 'measured-model', },);
        for (const ask of seen)
          expect(ask.reaskSeat?.reask,).toBe('measured-model',);
        expect(seen.filter(function isAlternate(ask,): boolean {
          return !BENCH.includes(ask.modelId,);
        },),).toEqual([],);
        expect(seen.filter(function isSeat(ask,): boolean {
          return ask.modelId === SEAT_HYPER_OPENROUTER_UNMEASURED;
        },),).toHaveLength(2,);
        expect([...gather.unreadable,],).toEqual([SEAT_HYPER_OPENROUTER_UNMEASURED,],);
      },
    },),

    it({
      name: 'RE-ASKS A SEAT THAT RAN AWAY OF ANOTHER MODEL IN THE RECOVERY ROUND with the plain question, records the '
        + 'answer as the seat\'s voice, and asks the seat\'s own model once only',
      fn: async () => {
        const { gather, seen, } = await gatherWithARunaway({ role: 'wide-judge', },);
        expect(seen.filter(function isSeat(ask,): boolean {
          return ask.modelId === SEAT_HYPER_OPENROUTER_UNMEASURED;
        },),).toHaveLength(1,);
        expect(seen.filter(function isAlternate(ask,): boolean {
          return !BENCH.includes(ask.modelId,);
        },).map(function summary(ask,) {
          return [ask.modelId, ask.contents,];
        },),).toEqual([[SEAT_HYPER_TEXT_BEDROCK, ['meow',],],],);
        // THE VOICE NAMES THE MODEL THAT ANSWERED and, apart, the seat it answered for (ledger B1142): weight, half
        // weight, the check of a model on the text it wrote, the tallies and the records read the first.
        expect(gather.voices.map(function summary(voice,): string {
          return `${voice.modelId}${(voice.seat === undefined) ? '' : ` for ${voice.seat}`} wrote by ${voice.value.meow}`;
        },).toSorted(),).toEqual([
          `${SEAT_HYPER_TEXT_BEDROCK} for ${SEAT_HYPER_OPENROUTER_UNMEASURED} wrote by ${SEAT_HYPER_TEXT_BEDROCK}`,
          `${SEAT_HYPER_OPENROUTER_VISION_EDITOR} wrote by ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}`,
          `${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} wrote by ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}`,
        ].toSorted(),);
        expect([...gather.unreadable,],).toEqual([],);
      },
    },),

    it({
      name: 'COUNTS A SEAT THE RECOVERY ROUND LOST AS UNREACHABLE OUT OF THE QUORUM (ledger B1371): the seat answered '
        + 'unreadably, its own-model re-ask was withheld by its hold, and the gather lists it unreachable and not '
        + 'unreadable and closes on the quorum of the bench that is left, as it does for a seat lost in a quorum round',
      fn: async () => {
        /**
         Seats the router refuses outright, sized so the seat the recovery round loses is the one that takes the
         reachable bench down to its quorum: one more reachable seat than the quorum is left before the recovery round.
         */
        const { benchQuorum, } = reachableQuorum({ benchSize: ROSTER_MODEL_IDS.length, unreachable: 0, },);
        const refused = ROSTER_MODEL_IDS
          .filter(function isOtherSeat(modelId,): boolean {
            return modelId !== SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
          },)
          .slice(0, ROSTER_MODEL_IDS.length - benchQuorum - 1,);
        const { gather, seen, } = await gatherWithALostRecovery({ refused, },);
        // The own-model re-ask never reached the client: the first ask is the only one the seat was sent.
        expect(seen.filter(function isSeat(modelId,): boolean {
          return modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
        },),).toHaveLength(1,);
        expect([...gather.unreachable,].toSorted(),).toEqual([...refused, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,].toSorted(),);
        expect([...gather.unreadable,],).toEqual([],);
        /**
         What the quorum is with the lost seat counted out, and with it counted in as it was before this entry.
         */
        const countedOut = reachableQuorum({ benchSize: ROSTER_MODEL_IDS.length, unreachable: gather.unreachable.size, },);
        const countedIn = reachableQuorum({ benchSize: ROSTER_MODEL_IDS.length, unreachable: refused.length, },);
        // The two differ on this bench by the grace of the last reachable seat, so the case can tell them apart.
        expect(countedOut.needed,).not.toBe(countedIn.needed,);
        expect(gather.quorum,).toBe(countedOut.needed,);
        expect(gather.quorumMet,).toBe(true,);
      },
    },),
  ],
},);

//endregion Quorum runaway tests
