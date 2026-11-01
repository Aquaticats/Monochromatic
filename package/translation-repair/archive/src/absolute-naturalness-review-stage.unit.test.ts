/**
 Tests independent absolute naturalness review settlement and delayed rejection.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
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
} from '@monochromatic-dev/translation-repair-provider';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { reviewAbsoluteNaturalness, } from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { scriptedStageSettingsWaiting, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Invented reviewer roster.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Even reviewer roster making exact-half quorum visible.
 */
const SIX_SEAT_ROSTER = [
  ...ROSTER,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
] as const;

/**
 Seats each provider contributes to production-shaped fixture.
 */
const PROVIDER_SEAT_COUNT = 4;

/**
 Production-shaped roster grouped by provider family.
 */
const PROVIDER_GROUPED_ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
] as const;

/**
 Builds reviewer client from per-model status and optional delayed rejection.

 @param unavailable - models returning no usable structured reply

 @param rejecting - model returning actionable rejection

 @param delayed - whether rejecting model answers after accepting peers

 @param held - seat whose answer the case releases after the review settled,
 so a straggler arrives after the close on every run

 @param refused - models the router refuses for want of a wet provider

 @returns Scripted absolute reviewer

 @example
 ```ts
 const client = reviewClient({ rejecting: ROSTER[2], delayed: true, });
 ```
 */
function reviewClient(
  {
    unavailable = [],
    rejecting,
    delayed = false,
    held,
    refused = [],
  }: {
    readonly unavailable?: readonly RosterModelId[];
    readonly rejecting?: RosterModelId;
    readonly delayed?: boolean;
    readonly held?: {
      /**
       Model whose answer holds on the gate.
       */
      readonly modelId: RosterModelId;
      /**
       Gate the answer waits on until the case resolves it.
       */
      readonly answer: PromiseWithResolvers<undefined>;
    };
    readonly refused?: readonly RosterModelId[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by absolute review',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (refused.includes(request.modelId,)) {
        throw new NoProviderForModelError({
          modelId: request.modelId,
          reason: 'every provider serving this cat is out of budget',
        },);
      }
      // ORDERED BY A GATE where the case holds one (ledger T5's recurrence of
      // 2026-10-03): a 30 ms sleeper against its peers' microtask answers let
      // the reply land inside the 0 ms close 2 full-suite runs of 5. A round
      // that never settles leaves the seat held, so the case fails on the
      // run's deadline rather than passing.
      if ((held !== undefined) && (request.modelId === held.modelId))
        await held.answer.promise;
      else if (delayed && (request.modelId === rejecting))
        await wait(30,);
      if (unavailable.includes(request.modelId,)) {
        return {
          kind: 'schema-mismatch',
          rawText: '{}',
          detail: 'scripted unusable seat',
        };
      }
      /**
       Candidate verdict for this seat.
       */
      const value: unknown = (request.modelId === rejecting)
        ? {
          isAcceptable: false,
          findings: [{
            paragraph: 1,
            problem: 'Replace stiff source-language word order.',
          },],
          assessment: 'candidate retains translationese',
        }
        : {
          isAcceptable: true,
          findings: [],
          assessment: 'whole candidate is publication-ready',
        };
      if (!request.validate(value,))
        throw new Error('scripted absolute review failed validation',);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by absolute review',);
    },
  };
}

/**
 Runs one invented absolute review.

 @param client - scripted reviewer

 @param messages - optional destination for operational logging

 @param modelIds - reviewer roster, defaulting to three-seat fixture

 @param graceMs - bounded time to retain post-quorum responses

 @param quorumOver - wider bench the quorum is taken over, absent for the
 seats asked

 @returns Absolute review outcome

 @example
 ```ts
 const review = await runReview({ client, });
 ```
 */
async function runReview(
  {
    client,
    messages,
    modelIds = ROSTER,
    graceMs = 0,
    quorumOver,
  }: {
    readonly client: ModelClient;
    readonly messages?: string[];
    readonly modelIds?: readonly RosterModelId[];
    readonly graceMs?: number;
    readonly quorumOver?: number;
  },
): ReturnType<typeof reviewAbsoluteNaturalness> {
  return await reviewAbsoluteNaturalness({
    // Whole bench: this case scripts every seat and reads over the bench it wrote.
    fanOut: 'whole-bench',
    client,
    modelIds,
    ...((quorumOver === undefined) ? {} : { quorumOver, }),
    subject: {
      lineStructured: false,
      sourceText: '猫猫在窗台上睡觉。',
      candidateText: 'The cat sleeps on the windowsill.',
      paragraphs: ['The cat sleeps on the windowsill.',],
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    exchangeTimeoutMs: HANG_STOP_MS,
    stageSettings: scriptedStageSettingsWaiting({ graceMs, },),
    l: (messages === undefined)
      ? tagged({ tag: 'absolute-review-test', },)
      : capturingLogger({ messages, },),
  },);
}

await describe({
  name: reviewAbsoluteNaturalness.name,
  children: [
    it({
      name: 'ACCEPTS only when every usable reviewer accepts and quorum stands',
      fn: async () => {
        const review = await runReview({ client: reviewClient({}), },);
        expect(review.verdict,).toBe('acceptable',);
        expect(review.usable,).toBe(3,);
        expect(review.seats.map(function status(seat,): string {
          return seat.status;
        },),).toEqual([
          'acceptable',
          'acceptable',
          'acceptable',
        ],);
      },
    },),

    it({
      name: 'STARTS GRACE AT HALF instead of requiring delayed final seat',
      fn: async () => {
        const messages: string[] = [];
        /**
         Gate the final seat's answer holds on, released after the review settled.
         */
        const heldAnswer = Promise.withResolvers<undefined>();
        const review = await runReview({
          client: reviewClient({
            rejecting: ROSTER[2],
            held: {
              modelId: ROSTER[2],
              answer: heldAnswer,
            },
          },),
          messages,
        },);
        expect(review.verdict,).toBe('acceptable',);
        expect(review.usable,).toBe(2,);
        expect(review.seats.map(function status(seat,): string {
          return seat.status;
        },),).toEqual([
          'acceptable',
          'acceptable',
          'unusable',
        ],);
        expect(review.findings,).toEqual([],);
        // RELEASED AFTER THE REVIEW SETTLED, so the straggler's words still
        // flow through the abandoned ask and the no-leak check covers them;
        // `wait(0)` drains that microtask chain before the check reads.
        heldAnswer.resolve(undefined,);
        await wait(0,);
        expect(messages.some(function leaksPrivateReview(line,): boolean {
          return line.includes('Replace stiff source-language word order.',)
            || line.includes('candidate retains translationese',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'KEEPS REJECTION that arrives inside bounded post-quorum grace',
      fn: async () => {
        const review = await runReview({
          client: reviewClient({
            rejecting: ROSTER[2],
            delayed: true,
          },),
          // FAR LONGER THAN THE 30 ms DELAY, since the round ends when every seat
          // has answered and so never waits this long; a window a loaded machine
          // can outrun would fail the case on a slow timer alone. At `graceMs: 0`
          // the late rejection is cut and this case fails, which shows the window
          // is what keeps it.
          graceMs: HANG_STOP_MS,
        },);
        expect(review.verdict,).toBe('unacceptable',);
        expect(review.usable,).toBe(3,);
        expect(review.findings,).toEqual([{
          paragraph: 1,
          problem: 'Replace stiff source-language word order.',
        },],);
      },
    },),

    ...([
      {
        label: 'Synthetic only',
        unavailable: PROVIDER_GROUPED_ROSTER.slice(PROVIDER_SEAT_COUNT,),
      },
      {
        label: 'Hyper only',
        unavailable: PROVIDER_GROUPED_ROSTER.slice(0, PROVIDER_SEAT_COUNT,),
      },
    ] as const).map(function providerOnlyCase(testCase,) {
      return it({
        name: `ACCEPTS ${testCase.label} exact-half participation as normal operation`,
        fn: async () => {
          const review = await runReview({
            client: reviewClient({ unavailable: testCase.unavailable, },),
            modelIds: PROVIDER_GROUPED_ROSTER,
          },);
          expect(review.verdict,).toBe('acceptable',);
          expect(review.usable,).toBe(PROVIDER_SEAT_COUNT,);
        },
      },);
    },),

    it({
      name: 'REFUSES TWO USABLE SEATS when six-seat roster needs exact half',
      fn: async () => {
        const review = await runReview({
          client: reviewClient({
            unavailable: SIX_SEAT_ROSTER.slice(2,),
          },),
          modelIds: SIX_SEAT_ROSTER,
        },);
        expect(review.verdict,).toBe('quorum-not-met',);
        expect(review.usable,).toBe(2,);
      },
    },),

    it({
      name: 'REFUSES THIN REVIEW with one usable seat',
      fn: async () => {
        const review = await runReview({
          client: reviewClient({
            unavailable: [
              ROSTER[1],
              ROSTER[2],
            ],
          },),
        },);
        expect(review.verdict,).toBe('quorum-not-met',);
        expect(review.usable,).toBe(1,);
      },
    },),

    it({
      name: 'SIZES ON THE SEATS THAT COULD ANSWER when the router refuses some, and records how many were '
        + 'out of reach (ledger E3; XingZ624, 2026-09-23): five seats asked over a bench of eight, as a '
        + 'confirmation asks, two refused out of budget and three accepting pass, where a quorum of '
        + 'four had refused them',
      fn: async () => {
        /**
         Five seats asked, as the discovery before it asked them.
         */
        const asked = PROVIDER_GROUPED_ROSTER.slice(0, 5,);
        const review = await runReview({
          client: reviewClient({ refused: asked.slice(0, 2,), },),
          modelIds: asked,
          quorumOver: PROVIDER_GROUPED_ROSTER.length,
        },);
        expect(review.verdict,).toBe('acceptable',);
        expect(review.usable,).toBe(3,);
        expect(review.unreachable,).toBe(5,);
      },
    },),

    it({
      name: 'STILL REFUSES THE SAME SHAPE WHEN THE TWO WERE LOST RATHER THAN REFUSED, the control that '
        + 'shows only the router\'s refusal moves the quorum',
      fn: async () => {
        /**
         Five seats asked.
         */
        const asked = PROVIDER_GROUPED_ROSTER.slice(0, 5,);
        const review = await runReview({
          client: reviewClient({ unavailable: asked.slice(0, 2,), },),
          modelIds: asked,
          quorumOver: PROVIDER_GROUPED_ROSTER.length,
        },);
        expect(review.verdict,).toBe('quorum-not-met',);
        expect(review.usable,).toBe(3,);
        expect(review.unreachable,).toBe(3,);
      },
    },),
  ],
},);
