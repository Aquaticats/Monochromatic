/**
 Tests for the recovery round's handling of a reasoning runaway (ledger B1073, B1074, B1075, B1076): a seat whose
 answer was a length stop with no content is re-asked of another model with the plain question, not of the same model
 under a nudge, and the answer is recorded as the seat's voice.

 THE OWNER'S ANSWERS OF 2026-10-30 are the cases: "Everywhere (Recommended)" sends the round to another model,
 "Honour holds (Recommended)" leaves out a model the seat's role is held out of, and "Plain question (Recommended)" sends
 no nudge to the other model. The lead's adoption of the same day leaves out the bench.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import type {
  RosterModelId,
  SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createReaskClaims,
  type ModelClient,
  type ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';
import {
  CUT_SHORT_RECOVERY_NUDGE,
  runRecoveryRound,
  type UnreadableCause,
} from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
  type MeowReply,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Recovery runaway tests

/**
 Text the stage's own prompt carries.
 */
const STAGE_PROMPT = 'meow';

/**
 The seat that ran away in every case: `deepseek-v4.1-flash`, the model of the evidence.
 */
const RUNAWAY_SEAT = SEAT_HYPER_OPENROUTER_UNMEASURED;

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
   Seat the ask carried for the re-ask of its own runaway, when it carried one.
   */
  readonly reaskSeat?: ReaskSeat;
};

/**
 Client that answers every model readably except the ones scripted to have no provider, and records each ask.

 @param seen - asks, filled as they arrive

 @param noProvider - models the router refuses for want of a provider with budget

 @param transportFails - models whose exchange fails in transport, which the stage reads as a lost voice that never
 arrived (ledger B1200)

 @returns Client honouring that script

 @example
 ```ts
 const client = scriptedClient({ seen, noProvider: [], transportFails: [], },);
 ```
 */
function scriptedClient(
  {
    seen,
    noProvider,
    transportFails,
  }: {
    readonly seen: SeenAsk[];
    readonly noProvider: readonly RosterModelId[];
    readonly transportFails: readonly RosterModelId[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      seen.push({
        modelId: request.modelId,
        contents: request.messages.map(function textOf(message,): string {
          if ((typeof message.content) !== 'string')
            throw new Error('scripted asks carry text content only',);
          return message.content;
        },),
        ...((request.reaskSeat === undefined) ? {} : { reaskSeat: request.reaskSeat, }),
      },);
      if (noProvider.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this model is out of budget',
        },);
      }
      if (transportFails.includes(request.modelId,))
        throw new Error(`connection reset while ${request.modelId} was answering`,);

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
}

/**
 What one recovery round did.
 */
type RunawayRun = {
  /**
   Asks the client saw, in order.
   */
  readonly seen: readonly SeenAsk[];

  /**
   Each outcome's seat, whether it was heard and the model the value says wrote it.
   */
  readonly outcomes: readonly {
    readonly modelId: RosterModelId;
    readonly heard: boolean;
    readonly wroteBy?: string;
    readonly answeredBy?: RosterModelId;
  }[];

  /**
   Lines the round logged, behind their levels.
   */
  readonly lines: readonly string[];
};

/**
 Runs one recovery round over a seat whose answer ran away.

 @param noProvider - models the router refuses for want of a provider

 @param transportFails - models whose exchange fails in transport, none by default

 @param reaskSeat - role and bench the round request carries, when it carries any

 @param askedInstead - model an earlier re-ask for the seat already went to

 @returns What the round asked, returned and logged

 @example
 ```ts
 const run = await recoverRunaway({ noProvider: [], },);
 ```
 */
async function recoverRunaway(
  {
    noProvider,
    transportFails = [],
    // A ROUND NAMES ITS SEAT (ledger B1079): the roster-wide role with no bench is what the cases that care about
    // neither the role nor the bench were written under.
    reaskSeat = { role: 'roster', reask: 'other-model', bench: [], },
    askedInstead,
  }: {
    readonly noProvider: readonly RosterModelId[];
    readonly transportFails?: readonly RosterModelId[];
    readonly reaskSeat?: ReaskSeat;
    readonly askedInstead?: RosterModelId;
  },
): Promise<RunawayRun> {
  /**
   Asks the client saw.
   */
  const seen: SeenAsk[] = [];

  /**
   Every line the round logged, behind its level.
   */
  const lines: string[] = [];

  /**
   Outcomes of the round under test.
   */
  const recovered = await runRecoveryRound<MeowReply>({
    roundRequest: {
      client: scriptedClient({
        seen,
        noProvider,
        transportFails,
      },),
      messages: [{ role: 'user', content: STAGE_PROMPT, },],
      signal: new AbortController().signal,
      exchangeTimeoutMs: HANG_STOP_MS,
      responseFormat: MEOW_FORMAT,
      validate: isMeowReply,
      stage: 'gate',
      l: levelCapturingLogger({ lines, },),
      graceMs: HANG_STOP_MS,
      reaskSeat,
    },
    unreadable: [RUNAWAY_SEAT,],
    causeOf: new Map<RosterModelId, UnreadableCause>([[RUNAWAY_SEAT, 'runaway',],],),
    ...((askedInstead === undefined) ? {} : { askedInstead: new Map([[RUNAWAY_SEAT, askedInstead,],],), }),
  },);
  return {
    seen,
    outcomes: recovered.map(function summarized(outcome,): RunawayRun['outcomes'][number] {
      return {
        modelId: outcome.modelId,
        heard: outcome.voice
          .heard,
        ...(outcome.voice
          .heard
          ? {
            wroteBy: outcome.voice
              .value
              .meow,
            // Conditional spread keeps the model absent where the seat's own model answered.
            ...((outcome.voice.answeredBy === undefined) ? {} : { answeredBy: outcome.voice.answeredBy, }),
          }
          : {}),
      };
    },),
    lines,
  };
}

/**
 Bench of a wide-judge question that seats the runaway and Gemma 4 26B.
 */
const WIDE_SEAT: ReaskSeat = {
  role: 'wide-judge' satisfies SeatRole,
  reask: 'other-model',
  bench: [RUNAWAY_SEAT, SEAT_HYPER_TEXT_BEDROCK,],
};

await describe({
  name: `${runRecoveryRound.name} (a reasoning runaway)`,
  concurrency: 1,
  children: [
    it({
      name: 'ASKS ANOTHER MODEL WITH THE PLAIN QUESTION for a seat whose answer ran away, never the seat\'s own model, '
        + 'and returns the answer as the seat\'s voice',
      fn: async () => {
        const run = await recoverRunaway({ noProvider: [], },);
        expect(run.seen.map(function ask(seen,) {
          return [seen.modelId, seen.contents,];
        },),).toEqual([[SEAT_HYPER_TEXT_BEDROCK, [STAGE_PROMPT,],],],);
        // THE VOICE NAMES THE MODEL THAT WROTE IT (ledger B1142), apart from the seat the outcome stands under.
        expect(run.outcomes,).toEqual([
          { modelId: RUNAWAY_SEAT, heard: true, wroteBy: SEAT_HYPER_TEXT_BEDROCK, answeredBy: SEAT_HYPER_TEXT_BEDROCK, },
        ],);
      },
    },),

    it({
      name: 'ASKS THE SEAT\'S OWN MODEL, NUDGED, WHERE THE GATHER IS A MEASUREMENT: a rival\'s answer recorded as the '
        + 'seat\'s would be scored as the measured model\'s, so no other model is asked and no warning is written',
      fn: async () => {
        const run = await recoverRunaway({
          noProvider: [],
          reaskSeat: { role: 'roster', bench: [], reask: 'measured-model', },
        },);
        expect(run.seen.map(function ask(seen,) {
          return [seen.modelId, seen.contents,];
        },),).toEqual([[RUNAWAY_SEAT, [STAGE_PROMPT, CUT_SHORT_RECOVERY_NUDGE.content,],],],);
        expect(run.outcomes,).toEqual([{ modelId: RUNAWAY_SEAT, heard: true, wroteBy: RUNAWAY_SEAT, },],);
        expect(run.lines.some(function isWarning(line,): boolean {
          return line.startsWith('warn ',) && line.includes('reasoning runaway',);
        },),).toBe(false,);
        expect(run.lines.some(function saysMeasurement(line,): boolean {
          return line.startsWith('info ',) && line.includes('measurement',) && line.includes(RUNAWAY_SEAT,);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'MOVES ON TO THE NEXT MODEL when one has no provider with budget, and asks nothing of the seat\'s own model '
        + 'while another model is left',
      fn: async () => {
        const run = await recoverRunaway({ noProvider: [SEAT_HYPER_TEXT_BEDROCK,], },);
        expect(run.seen.map(function modelOf(seen,) {
          return seen.modelId;
        },),).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,],);
        expect(run.outcomes,).toEqual([
          { modelId: RUNAWAY_SEAT, heard: true, wroteBy: SEAT_BEDROCK_ONLY_TEXT, answeredBy: SEAT_BEDROCK_ONLY_TEXT, },
        ],);
      },
    },),

    it({
      name: 'MOVES ON TO THE NEXT MODEL when one\'s exchange fails in transport (ledger B1200), as it does for one with no '
        + 'provider: the seat is lost only when every model of the order has failed, and its own model is not asked '
        + 'while another is left',
      fn: async () => {
        const run = await recoverRunaway({ noProvider: [], transportFails: [SEAT_HYPER_TEXT_BEDROCK,], },);
        expect(run.seen.map(function modelOf(seen,) {
          return seen.modelId;
        },),).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,],);
        expect(run.outcomes,).toEqual([
          { modelId: RUNAWAY_SEAT, heard: true, wroteBy: SEAT_BEDROCK_ONLY_TEXT, answeredBy: SEAT_BEDROCK_ONLY_TEXT, },
        ],);
      },
    },),

    it({
      name: 'RELEASES THE CLAIM OF A MODEL THE ROUTER REFUSED (ledger B1200) and keeps that of one whose exchange failed '
        + 'in transport, since only the second was asked',
      fn: async () => {
        const claims = createReaskClaims();
        await recoverRunaway({
          noProvider: [SEAT_HYPER_TEXT_BEDROCK,],
          transportFails: [SEAT_BEDROCK_ONLY_TEXT,],
          reaskSeat: { role: 'roster', reask: 'other-model', bench: [], claims, },
        },);
        expect(claims.list(),).not.toContain(SEAT_HYPER_TEXT_BEDROCK,);
        expect(claims.list(),).toContain(SEAT_BEDROCK_ONLY_TEXT,);
      },
    },),

    it({
      name: 'LEAVES OUT THE MODEL AN EARLIER RE-ASK FOR THE SEAT WENT TO, so the round asks the next model and does not '
        + 'replay the first model\'s claimed reply as a second voice',
      fn: async () => {
        const run = await recoverRunaway({ noProvider: [], askedInstead: SEAT_HYPER_TEXT_BEDROCK, },);
        expect(run.seen.map(function modelOf(seen,) {
          return seen.modelId;
        },),).toEqual([SEAT_BEDROCK_ONLY_TEXT,],);
      },
    },),

    it({
      name: 'SKIPS A MODEL THE ROLE IS HELD OUT OF AND A MODEL ON THE BENCH, then falls back to the seat\'s own model '
        + 'under the cut-short nudge and says so on a warning, the nudge on that ask alone',
      fn: async () => {
        /**
         Every model that could take the re-ask for a wide judge off this bench, in the order of their runaway shares.
         */
        const takers = [
          SEAT_BEDROCK_ONLY_TEXT,
          SEAT_OPENROUTER_ONLY,
          SEAT_SYNTHETIC_VISION_WITHHELD,
          SEAT_HYPER_VISION,
        ] as const;
        const run = await recoverRunaway({ noProvider: takers, reaskSeat: WIDE_SEAT, },);
        expect(run.seen.map(function modelOf(seen,) {
          return seen.modelId;
        },),).toEqual([...takers, RUNAWAY_SEAT,],);
        for (const plain of run.seen.slice(0, -1,)) {
          expect(plain.contents,).toEqual([STAGE_PROMPT,],);
        }
        expect(run.seen.at(-1,)?.contents,).toEqual([STAGE_PROMPT, CUT_SHORT_RECOVERY_NUDGE.content,],);
        expect(run.outcomes,).toEqual([{ modelId: RUNAWAY_SEAT, heard: true, wroteBy: RUNAWAY_SEAT, },],);
        expect(run.lines.filter(function isWarning(line,): boolean {
          return line.startsWith('warn ',) && line.includes('no other model could take the re-ask',);
        },),).toEqual([
          `warn [runRecoveryRound] [recoverRunaway] gate: ${RUNAWAY_SEAT}: no other model could take the re-ask of a `
            + `reasoning runaway (${String(takers.length,)} tried); re-asking ${RUNAWAY_SEAT}, nudged`,
        ],);
      },
    },),

    it({
      name: 'FALLS BACK TO THE SEAT\'S OWN MODEL WITH THE WARNING when the bench holds every model that could take the '
        + 're-ask: nothing else is asked',
      fn: async () => {
        const run = await recoverRunaway({
          noProvider: [],
          reaskSeat: {
            role: 'roster',
            reask: 'other-model',
            bench: [
              RUNAWAY_SEAT,
              SEAT_HYPER_TEXT_BEDROCK,
              SEAT_BEDROCK_ONLY_TEXT,
              SEAT_OPENROUTER_ONLY,
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_VISION,
              'glm-5.3',
              'hf:Qwen/Qwen3.8-27B',
              'hf:zai-org/GLM-5.3-Flash',
            ],
          },
        },);
        expect(run.seen.map(function modelOf(seen,) {
          return seen.modelId;
        },),).toEqual([RUNAWAY_SEAT,],);
        expect(run.seen[0]?.contents,).toEqual([STAGE_PROMPT, CUT_SHORT_RECOVERY_NUDGE.content,],);
      },
    },),

    it({
      name: 'LOGS THE SEAT, THE REASON, THE MODEL CHOSEN AND EACH MODEL LEFT OUT WITH WHY, and counts the runaway as its '
        + 'own cause in the round\'s lines',
      fn: async () => {
        const run = await recoverRunaway({
          noProvider: [],
          reaskSeat: { role: 'roster', reask: 'other-model', bench: [RUNAWAY_SEAT, SEAT_HYPER_VISION,], },
        },);
        expect(run.lines.filter(function isRunawayLine(line,): boolean {
          return line.includes('reasoning runaway',)
            || line.includes('instead, plain question',)
            || line.includes(': recovery round ',);
        },),).toEqual([
          'warn [runRecoveryRound] gate: recovery round for 1 unreadable answer (1 runaway)',
          `info [runRecoveryRound] [recoverRunaway] gate: ${RUNAWAY_SEAT}: reasoning runaway (a length stop with no content); `
            + 'left out: hf:Qwen/Qwen3.8-27B (held out of the roster role by hyper-slow), '
            + 'hf:openai/gpt-oss-120b (held out of the roster role by owner-culled), '
            + `${SEAT_HYPER_VISION} (seated on the bench of this question), `
            + 'google.gemma-4-31b (held out of the roster role by judge-unmeasured), '
            + 'xiaomi/mimo-v2.6-flash (held out of the roster role by judge-unmeasured); to ask in turn: '
            + 'gemma-4-26b-a4b-it, google.gemma-4-e2b, inception/mercury-2.5, hf:moonshotai/Kimi-K3, glm-5.3, '
            + 'hf:zai-org/GLM-5.3-Flash',
          `info [runRecoveryRound] [recoverRunaway] gate: ${RUNAWAY_SEAT}: re-asking ${SEAT_HYPER_TEXT_BEDROCK} instead, `
            + 'plain question',
          'info [runRecoveryRound] gate: recovery round heard 1 of 1 re-asked voice',
        ],);
      },
    },),

    it({
      name: 'WRITES THE MODELS LEFT OUT ONCE, BEFORE THE FIRST ASK, so a model without a provider moves the round on '
        + 'without the left-out line being written again',
      fn: async () => {
        const run = await recoverRunaway({ noProvider: [SEAT_HYPER_TEXT_BEDROCK,], },);
        expect(run.lines.filter(function namesLeftOut(line,): boolean {
          return line.includes('left out:',);
        },).length,).toBe(1,);
        expect(run.lines.filter(function asksOne(line,): boolean {
          return line.includes('instead, plain question',);
        },).length,).toBe(2,);
      },
    },),

    it({
      name: 'WRITES THE MODELS LEFT OUT ALSO WHERE NONE IS LEFT TO ASK, ahead of the warning of the fall back, and says '
        + 'that none is to be asked',
      fn: async () => {
        const run = await recoverRunaway({
          noProvider: [],
          reaskSeat: {
            role: 'roster',
            reask: 'other-model',
            bench: [
              RUNAWAY_SEAT,
              SEAT_HYPER_TEXT_BEDROCK,
              SEAT_BEDROCK_ONLY_TEXT,
              SEAT_OPENROUTER_ONLY,
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_VISION,
              'glm-5.3',
              'hf:Qwen/Qwen3.8-27B',
              'hf:zai-org/GLM-5.3-Flash',
            ],
          },
        },);
        expect(run.lines.filter(function namesLeftOut(line,): boolean {
          return line.includes('left out:',) && line.includes('to ask in turn: none',);
        },).length,).toBe(1,);
      },
    },),

    it({
      name: 'CARRIES THE BENCH AND THE SEAT ON THE ASK, so a runaway of the other model is not re-asked of a model seated '
        + 'on the question or back of the seat that began it',
      fn: async () => {
        const run = await recoverRunaway({
          noProvider: [],
          reaskSeat: { role: 'wide-judge', reask: 'other-model', bench: [SEAT_BEDROCK_ONLY_TEXT,], },
        },);
        expect(run.seen[0]?.modelId,).toBe(SEAT_HYPER_TEXT_BEDROCK,);
        expect(run.seen[0]?.reaskSeat,).toEqual({
          role: 'wide-judge',
          reask: 'other-model',
          bench: [SEAT_BEDROCK_ONLY_TEXT, RUNAWAY_SEAT,],
        },);
      },
    },),
  ],
},);

//endregion Recovery runaway tests
