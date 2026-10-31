/**
 Tests for a gather whose seats run away together (ledger B1076, B1080): the seats of one question share one record of
 the models asked for them, so the recovery round never asks one model for two seats, and the seat that finds no model
 left is asked of its own model, nudged, on a warning.

 ONE MODEL GIVES AT MOST ONE VOICE PER QUESTION ("one judge alone never decides", 2026-10-25; "Two judges must name it",
 2026-10-25).

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  alternateModelsFor,
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createReaskClaims,
  type ModelClient,
  type ReaskClaims,
  type ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import {
  gatherStageVoices,
  seatOfVoice,
} from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Quorum claims tests

/**
 The two seats that run away, and the seat that answers.
 */
const BENCH: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
];

/**
 Seats whose first ask ends in a length stop with no content.
 */
const RUNAWAYS: ReadonlySet<RosterModelId> = new Set([
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
],);

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
 Runs one gather over a bench whose runaway seats each answer with a length stop with no content once.

 @param bench - models seated on the question, in roster order

 @param reaskClaims - record of the question that a caller sharing it with a later step hands the gather; absent for a
 gather that is the whole question

 @returns Gather, the asks the client saw and the lines logged

 @example
 ```ts
 const { gather, seen, } = await gatherWithRunaways({ bench: BENCH, },);
 ```
 */
async function gatherWithRunaways(
  {
    bench,
    reaskClaims,
  }: {
    readonly bench: readonly RosterModelId[];
    readonly reaskClaims?: ReaskClaims;
  },
) {
  /**
   Asks the client saw.
   */
  const seen: SeenAsk[] = [];

  /**
   Models that have already run away once.
   */
  const ranAway = new Set<RosterModelId>();

  /**
   Lines the gather logged.
   */
  const lines: string[] = [];

  /**
   Client whose runaway seats answer a length stop with no content on their first ask and whose every other ask is
   answered.
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
      if (RUNAWAYS.has(request.modelId,) && (!ranAway.has(request.modelId,))) {
        ranAway.add(request.modelId,);
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
    lines,
    gather: await gatherStageVoices({
      reask: 'other-model',
      client,
      modelIds: bench,
      messages: [{ role: 'user', content: 'meow', },],
      signal: new AbortController().signal,
      exchangeTimeoutMs: HANG_STOP_MS,
      responseFormat: MEOW_FORMAT,
      validate: isMeowReply,
      stage: 'panel',
      l: levelCapturingLogger({ lines, },),
      graceMs: HANG_STOP_MS,
      fanOut: 'whole-bench',
      role: 'roster',
      ...((reaskClaims === undefined) ? {} : { reaskClaims, }),
    },),
  };
}

await describe({
  name: `${gatherStageVoices.name} (seats that run away together)`,
  concurrency: 1,
  children: [
    it({
      name: 'SHARES ONE RECORD OF THE MODELS ASKED ACROSS EVERY SEAT OF THE GATHER: every ask of the first round carries '
        + 'the same record, so a model one seat takes is not taken again by another',
      fn: async () => {
        const { seen, } = await gatherWithRunaways({ bench: BENCH, },);
        const firstRound = seen.slice(0, BENCH.length,);
        expect(firstRound,).toHaveLength(BENCH.length,);
        const [first,] = firstRound;
        expect(first?.reaskSeat?.claims === undefined,).toBe(false,);
        for (const ask of firstRound) {
          expect(ask.reaskSeat?.role,).toBe('roster',);
          expect(ask.reaskSeat?.bench,).toEqual(BENCH,);
          expect(ask.reaskSeat?.claims === first?.reaskSeat?.claims,).toBe(true,);
        }
      },
    },),

    it({
      name: 'CARRIES THE RECORD A CALLER HANDS IT on every ask and takes the models of its re-asks from that record, so '
        + 'a later step of the same question (the repair of an invalid candidate) shares the models already taken',
      fn: async () => {
        const claims = createReaskClaims();
        const { seen, } = await gatherWithRunaways({ bench: BENCH, reaskClaims: claims, },);
        for (const ask of seen.slice(0, BENCH.length,)) {
          expect(ask.reaskSeat?.claims === claims,).toBe(true,);
        }
        expect(claims.list().length > 0,).toBe(true,);
      },
    },),

    it({
      name: 'RE-ASKS TWO SEATS THAT RAN AWAY TOGETHER OF TWO DIFFERENT MODELS in the recovery round: the second takes the '
        + 'next model of its order that the first did not take, and each answer is recorded as its own seat\'s voice',
      fn: async () => {
        const { gather, seen, } = await gatherWithRunaways({ bench: BENCH, },);

        /**
         Model the first seat is re-asked of.
         */
        const [firstTakes,] = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          role: 'roster',
          bench: BENCH,
        },).alternates;
        if (firstTakes === undefined)
          throw new Error('the first seat has a model left to take',);

        /**
         Model the second seat is re-asked of once the first has taken its own.
         */
        const [secondTakes,] = alternateModelsFor({
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          role: 'roster',
          bench: BENCH,
          askedForOtherSeats: [firstTakes,],
        },).alternates;
        expect(seen.filter(function isAlternate(ask,): boolean {
          return !BENCH.includes(ask.modelId,);
        },).map(function modelOf(ask,) {
          return ask.modelId;
        },),).toEqual([firstTakes, secondTakes,],);
        // THE VOICE NAMES THE MODEL THAT ANSWERED (ledger B1142) and the seat it answered for apart, so a seat re-asked of
        // another model reads as that seat's voice written by that model.
        expect(gather.voices.every(function namesItsAuthor(voice,): boolean {
          return voice.modelId === voice.value.meow;
        },),).toBe(true,);
        expect(gather.voices.map(function summary(voice,): string {
          return `${seatOfVoice({ voice, },)} wrote by ${voice.value.meow}`;
        },).toSorted(),).toEqual([
          `${SEAT_HYPER_OPENROUTER_UNMEASURED} wrote by ${firstTakes}`,
          `${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} wrote by ${secondTakes}`,
          `${SEAT_HYPER_OPENROUTER_VISION_EDITOR} wrote by ${SEAT_HYPER_OPENROUTER_VISION_EDITOR}`,
        ].toSorted(),);
      },
    },),

    it({
      name: 'ASKS THE SECOND SEAT OF ITS OWN MODEL, NUDGED, ON A WARNING when the first took the one model that was left: '
        + 'one model is never asked for two seats',
      fn: async () => {
        /**
         Models either seat could be re-asked of, with the bench holding the three seats.
         */
        const { alternates, } = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          role: 'roster',
          bench: BENCH,
        },);
        const [onlyOne, ...others] = alternates;
        if (onlyOne === undefined)
          throw new Error('the first seat has a model left to take',);

        /**
         The bench with every model but one filling a seat, so one model is left to take a re-ask.
         */
        const crowded = [...BENCH, ...others,];
        const { gather, seen, lines, } = await gatherWithRunaways({ bench: crowded, },);
        expect(seen.filter(function isOnlyOne(ask,): boolean {
          return ask.modelId === onlyOne;
        },),).toHaveLength(1,);

        /**
         The second runaway seat's asks, the second of which is its own model asked again.
         */
        const second = seen.filter(function isSecondSeat(ask,): boolean {
          return ask.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
        },);
        expect(second,).toHaveLength(2,);
        expect(second[1]?.contents.at(-1,),).toContain('was cut off at the length limit',);
        expect(gather.voices.map(function summary(voice,): string {
          return `${voice.modelId} wrote by ${voice.value.meow}`;
        },),).toContain(`${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} wrote by ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}`,);
        expect(lines.filter(function warnsOfNoModelLeft(line,): boolean {
          return line.startsWith('warn [runRecoveryRound]',) && line.includes('no other model could take the re-ask',);
        },),).toEqual([
          `warn [runRecoveryRound] [recoverRunaway] panel: ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}: no other model `
            + `could take the re-ask of a reasoning runaway (0 tried); re-asking ${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}, `
            + 'nudged',
        ],);
      },
    },),
  ],
},);

//endregion Quorum claims tests
