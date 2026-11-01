/**
 Tests what a stage makes of the router's refusal of a first ask its card holds off the provider the policy chose
 (ledger B1365): Qwen3.8-27B asked as an editor when Synthetic reads dry and Hyper would take the call.

 The router is the real one over stub callers that record what they were sent, and the stage is the real one, so each
 case reads the whole path from the stage's ask to the provider that was never reached. The refusal must be read as one
 seat unreachable, as a seat no provider serves is: it never throws out of the stage, never ends the entry or the run,
 stays out of the quorum's denominator, is not asked again and is not counted as an ask. Fixtures are cat-themed
 invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alternateModelsFor,
  type ChatTextRequest,
  createReaskClaims,
  createRoutingClient,
  type ModelClient,
  type ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import { holdNothing, } from '@monochromatic-dev/translation-repair-provider/ts/provider-router-wet-budgets.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
  type MeowReply,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  type DrynessReading,
  HYPER_WET_ONLY,
  readingWith,
  SYNTHETIC_WET_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/dryness-readings.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  attemptStageCall,
  createSeatTally,
  gatherStageVoices,
  reachableQuorum,
  runRecoveryRound,
  type SeatCount,
  seatTallyClient,
  SEAT_WITHHOLD_ASKS,
  type UnreadableCause,
} from '../dist/final/node/index.mjs';

/**
 Abort signal every call here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Logger of the calling stage.
 */
const l = tagged({ tag: 'stage-first-ask-hold-test', },);

/**
 Role of the seat the cases ask in: an editor, which honours the hold `hyper-slow` of Qwen3.8-27B's card.
 */
const SEAT_ROLE = 'editor';

/**
 Seat of a run in the editor role, with the bench and the record of asked models it carries.

 @param bench - models seated on the question

 @returns Seat naming the editor role and the run's mode of re-ask

 @example
 ```ts
 const seat = editorSeat({ bench: [], },);
 ```
 */
function editorSeat({ bench, }: { readonly bench: readonly RosterModelId[]; },): ReaskSeat {
  return { role: SEAT_ROLE, bench, reask: 'other-model', claims: createReaskClaims(), };
}

/**
 The providers that read wet when Synthetic is the one that ran dry: those the stub callers stand for, so a model the
 router sends to a wet provider always meets a caller.
 */
const WET_WITHOUT_SYNTHETIC = ['neuralwatt', 'hyper', 'openrouter',] as const;

/**
 One request a stub caller was sent.
 */
type Sent = {
  /**
   Provider the router sent it to.
   */
  readonly provider: ProviderName;

  /**
   Model it was addressed to.
   */
  readonly modelId: RosterModelId;
};

/**
 A client over the real router, whose callers record what they were sent and answer with the model that wrote the reply.

 @param reading - which providers have budget when the router chooses
 @param sent - requests the callers were sent, filled as they arrive
 @param dryness - the older reading of the meters the client reports to a re-ask, when it reports one

 @returns Client the stage can take

 @example
 ```ts
 const client = routerClient({ reading: HYPER_WET_ONLY, sent: [], },);
 ```
 */
function routerClient(
  {
    reading,
    sent,
    dryness,
  }: {
    readonly reading: DrynessReading;
    readonly sent: Sent[];
    readonly dryness?: DrynessReading;
  },
): ModelClient {
  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records the request and answers as a meow reply naming the model asked
   */
  function callerFor(provider: ProviderName,): { readonly chatText: (request: ChatTextRequest,) => Promise<{ readonly text: string; }>; } {
    return {
      chatText: function chatText(request: ChatTextRequest,): Promise<{ readonly text: string; }> {
        sent.push({ provider, modelId: request.modelId, },);
        return Promise.resolve({ text: JSON.stringify({ meow: request.modelId, },), },);
      },
    };
  }
  /**
   The real router over the stub callers.
   */
  const router = createRoutingClient({
    callers: {
      neuralwatt: callerFor('neuralwatt',),
      synthetic: callerFor('synthetic',),
      hyper: callerFor('hyper',),
      openrouter: callerFor('openrouter',),
    },
    budgets: {
      read: function read() {
        return Promise.resolve(reading,);
      },
      markRefused: function markNothingRefused(): Promise<void> {
        return Promise.resolve();
      },
      holds: holdNothing,
    },
    l: levelCapturingLogger({ lines: [], },),
  },);
  return {
    ...router,
    ...((dryness === undefined) ? {} : { providerDryness: () => Promise.resolve(dryness,), }),
    quotas: function quotasUnused(): Promise<never> {
      return Promise.reject(new Error('quotas unused',),);
    },
  };
}

/**
 Asks one model once through the stage's call, as the seat of a run in a role or with no seat.

 @param client - client the call goes through
 @param modelId - model asked
 @param reaskSeat - seat the call names, when it names one

 @returns Voice the stage made of it

 @example
 ```ts
 const voice = await stageCall({ client, modelId, reaskSeat: editorSeat({ bench: [], },), },);
 ```
 */
function stageCall(
  {
    client,
    modelId,
    reaskSeat,
  }: {
    readonly client: ModelClient;
    readonly modelId: RosterModelId;
    readonly reaskSeat?: ReaskSeat;
  },
): ReturnType<typeof attemptStageCall<MeowReply>> {
  return attemptStageCall<MeowReply>({
    client,
    modelId,
    messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
    signal: SIGNAL,
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: MEOW_FORMAT,
    validate: isMeowReply,
    stage: 'critic',
    l,
    ...((reaskSeat === undefined) ? {} : { reaskSeat, }),
  },);
}

/**
 The counts of one seat of a tally.

 @param counts - counts the tally holds

 @param modelId - seat asked about

 @returns The seat's counts

 @throws Error when no call was counted against the seat, which these cases never expect

 @example
 ```ts
 const count = countOf({ counts: tally.counts(), modelId, },);
 ```
 */
function countOf(
  { counts, modelId, }: { readonly counts: readonly SeatCount[]; readonly modelId: RosterModelId; },
): SeatCount {
  return nonNullishOrThrow(counts.find(function isSeat(count,): boolean {
    return count.modelId === modelId;
  },),);
}

await describe({
  name: 'the stage over a router that refuses a first ask its card holds off the provider chosen',
  // One case at a time: the cases share the module's seat constants and read the counts of their own tallies.
  concurrency: 1,
  children: [
    it({
      name: 'READS A HELD EDITOR SEAT AS THAT SEAT UNREACHABLE (ledger B1365): no request is sent, the voice is neither '
        + 'heard nor a crash nor the end of the run, and the same model asked with no seat, or as a measurement, is heard',
      fn: async () => {
        /**
         Requests the stub callers were sent.
         */
        const sent: Sent[] = [];
        const client = routerClient({ reading: HYPER_WET_ONLY, sent, },);
        const held = await stageCall({
          client,
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: editorSeat({ bench: [], },),
        },);
        expect(held,).toEqual({ heard: false, answered: false, unreachable: true, },);
        expect(sent,).toEqual([],);
        const unseated = await stageCall({ client, modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },);
        const measured = await stageCall({
          client,
          modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          reaskSeat: { ...editorSeat({ bench: [], },), reask: 'measured-model', },
        },);
        expect([unseated.heard, measured.heard,],).toEqual([true, true,],);
        expect(sent.map(function providerOf(request,): ProviderName {
          return request.provider;
        },),).toEqual(['hyper', 'hyper',],);
      },
    },),

    it({
      name: 'COUNTS A HELD SEAT AS A SEAT NO PROVIDER SERVES IS COUNTED (ledger B1365, B711): the same voice, and in the '
        + 'seat tally an unreachable call and no ask, so the run never withholds the seat for it',
      fn: async () => {
        /**
         What one reading of the meters made of the held model's call and of the tally's account of it.
         */
        async function describedUnder(reading: DrynessReading,) {
          const tally = createSeatTally();
          const client = seatTallyClient({
            inner: routerClient({ reading, sent: [], },),
            tally,
            minAsks: SEAT_WITHHOLD_ASKS,
          },);
          const voice = await stageCall({
            client,
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            reaskSeat: editorSeat({ bench: [], },),
          },);
          return { voice, count: countOf({ counts: tally.counts(), modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },), };
        }
        const held = await describedUnder(HYPER_WET_ONLY,);
        // OpenRouter does not serve this model, so with Synthetic and Hyper both dry no provider is left for it.
        const dry = await describedUnder(readingWith({ wet: ['openrouter',], },),);
        expect(held,).toEqual(dry,);
        expect(held.voice,).toEqual({ heard: false, answered: false, unreachable: true, },);
        expect(held.count.unreachable,).toBe(1,);
        expect(held.count.asked,).toBe(0,);
      },
    },),

    it({
      name: 'SIZES A GATHER\'S QUORUM ON THE SEATS LEFT AND NEVER ASKS THE HELD SEAT AGAIN (ledger B1365): the held seat '
        + 'is unreachable, stays out of the quorum\'s denominator, is refused once and not in each retry round, and the '
        + 'seats that can answer are heard',
      fn: async () => {
        /**
         The bench: the held model and seats Hyper serves with no hold against the editor role.
         */
        const modelIds: readonly RosterModelId[] = [
          SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_ONLY,
          SEAT_HYPER_VISION,
          SEAT_HYPER_TEXT_BEDROCK,
        ];
        const sent: Sent[] = [];
        const tally = createSeatTally();
        const gather = await gatherStageVoices<MeowReply>({
          client: seatTallyClient({
            inner: routerClient({ reading: HYPER_WET_ONLY, sent, },),
            tally,
            minAsks: SEAT_WITHHOLD_ASKS,
          },),
          modelIds,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
          exchangeTimeoutMs: HANG_STOP_MS,
          responseFormat: MEOW_FORMAT,
          validate: isMeowReply,
          stage: 'critic',
          l,
          graceMs: HANG_STOP_MS,
          fanOut: 'whole-bench',
          seatTally: tally,
          role: SEAT_ROLE,
          reask: 'other-model',
        },);
        expect([...gather.unreachable,],).toEqual([SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],);
        expect(gather.quorum,).toBe(reachableQuorum({ benchSize: modelIds.length, unreachable: 1, },).needed,);
        expect(gather.quorumMet,).toBe(true,);
        expect(gather.voices.map(function modelOf(voice,): RosterModelId {
          return voice.modelId;
        },).toSorted(),).toEqual(modelIds.slice(1,).toSorted(),);
        expect(gather.asked.has(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,),).toBe(true,);
        expect(sent.some(function isHeldModel(request,): boolean {
          return request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
        },),).toBe(false,);
        expect(countOf({ counts: tally.counts(), modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },).unreachable,).toBe(1,);
      },
    },),

    it({
      name: 'FALLS TO THE NEXT ALTERNATE OF A RUNAWAY WHEN SYNTHETIC DRIED BETWEEN THE READING AND THE ASK (ledger B1365): '
        + 'the older reading lists Qwen3.8-27B first, the router refuses the ask it would send to Hyper, and the seat '
        + 'is answered by the next model of the order',
      fn: async () => {
        /**
         Models seated on the question, which the alternates leave out.
         */
        const bench: readonly RosterModelId[] = [
          'hf:zai-org/GLM-5.3-Flash',
          'claude-sonnet-5-5',
          'glm-5.3',
          SEAT_HYPER_OPENROUTER_UNMEASURED,
        ];
        /**
         Alternates the older reading, which has Synthetic serving Qwen3.8-27B, gives for the runaway editor.
         */
        const { alternates, } = alternateModelsFor({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          role: SEAT_ROLE,
          bench,
          askedAlready: [],
          askedForOtherSeats: [],
          routeTimeHolds: { applied: 'in-reading', dry: SYNTHETIC_WET_ONLY, },
        },);
        // THE CASE STANDS ON THIS: with Qwen3.8-27B first, the ask the router refuses is the first one of the round.
        expect(alternates[0],).toBe(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
        const sent: Sent[] = [];
        const lines: string[] = [];
        const recovered = await runRecoveryRound<MeowReply>({
          roundRequest: {
            // The router chooses by the meters as they are now, which have Synthetic dry; the client reports the older
            // reading the alternates were listed by.
            client: routerClient({
              reading: readingWith({ wet: WET_WITHOUT_SYNTHETIC, },),
              sent,
              dryness: SYNTHETIC_WET_ONLY,
            },),
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: SIGNAL,
            exchangeTimeoutMs: HANG_STOP_MS,
            responseFormat: MEOW_FORMAT,
            validate: isMeowReply,
            stage: 'gate',
            l: levelCapturingLogger({ lines, },),
            graceMs: HANG_STOP_MS,
            reaskSeat: editorSeat({ bench, },),
          },
          unreadable: [SEAT_HYPER_OPENROUTER_UNMEASURED,],
          causeOf: new Map<RosterModelId, UnreadableCause>([[SEAT_HYPER_OPENROUTER_UNMEASURED, 'runaway',],],),
        },);
        expect(sent.some(function isHeldModel(request,): boolean {
          return request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
        },),).toBe(false,);
        expect(lines.some(function saysItCannotTakeIt(line,): boolean {
          return line.includes(`${SEAT_SYNTHETIC_VISION_NO_OPENROUTER} cannot take the re-ask`,);
        },),).toBe(true,);
        expect(recovered.map(function summarized(outcome,) {
          return {
            modelId: outcome.modelId,
            heard: outcome.voice.heard,
            answeredByTheHeldModel: outcome.voice.heard && (outcome.voice.answeredBy === SEAT_SYNTHETIC_VISION_NO_OPENROUTER),
          };
        },),).toEqual([{ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, heard: true, answeredByTheHeldModel: false, },],);
      },
    },),
  ],
},);
