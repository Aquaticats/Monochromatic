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
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import { gatherStageVoices, } from '../dist/final/node/index.mjs';
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
  { role, }: { readonly role?: 'wide-judge'; },
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
      ...((role === undefined) ? {} : { role, }),
    },),
  };
}

await describe({
  name: `${gatherStageVoices.name} (a reasoning runaway)`,
  concurrency: 1,
  children: [
    it({
      name: 'NAMES ITS BENCH AND ITS ROLE ON EVERY ASK of the first round, the roster-wide role where the caller names '
        + 'none, so the re-ask of a runaway can leave the bench out and honour the holds of the role',
      fn: async () => {
        const named = await gatherWithARunaway({ role: 'wide-judge', },);
        for (const ask of named.seen.slice(0, BENCH.length,)) {
          expect(ask.reaskSeat,).toEqual({ role: 'wide-judge', bench: BENCH, },);
        }
        const unnamed = await gatherWithARunaway({},);
        for (const ask of unnamed.seen.slice(0, BENCH.length,)) {
          expect(ask.reaskSeat,).toEqual({ role: 'roster', bench: BENCH, },);
        }
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
        expect(gather.voices.map(function summary(voice,): string {
          return `${voice.modelId} wrote by ${voice.value.meow}`;
        },).toSorted(),).toEqual([
          `${SEAT_HYPER_OPENROUTER_UNMEASURED} wrote by ${SEAT_HYPER_TEXT_BEDROCK}`,
          `${SEAT_HYPER_OPENROUTER_VISION_EDITOR} wrote by ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}`,
          `${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} wrote by ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}`,
        ].toSorted(),);
        expect([...gather.unreadable,],).toEqual([],);
      },
    },),
  ],
},);

//endregion Quorum runaway tests
