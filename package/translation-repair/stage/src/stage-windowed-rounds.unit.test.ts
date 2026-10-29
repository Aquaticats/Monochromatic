/**
 Tests for the windowed rounds the six self-reading stages ask through.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

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
import {
  firstRoundWindow,
  runWindowedRounds,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Six-seat bench in roster order.
 */
const BENCH: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_VISION,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
];

/**
 Quorum over the whole bench.
 */
const QUORUM = Math.ceil(BENCH.length / 2,);

/**
 Test logger.
 */
const l = tagged({ tag: 'stage-windowed-rounds-test', },);

/**
 Client answering every seat, except those scripted to fail the first time
 they are asked or every time.

 @param failsOnce - seats whose first ask throws and whose second answers

 @param failsAlways - seats whose every ask throws

 @param unreadable - seats that answer a shape the guard refuses

 @param refused - seats the router refuses for want of a wet provider

 @returns Client plus the seats asked, in call order

 @example
 ```ts
 const { client, asked, } = scriptedClient({ failsOnce: [], failsAlways: [], unreadable: [], },);
 ```
 */
function scriptedClient(
  {
    failsOnce,
    failsAlways,
    unreadable,
    refused = [],
  }: {
    readonly failsOnce: readonly RosterModelId[];
    readonly failsAlways: readonly RosterModelId[];
    readonly unreadable: readonly RosterModelId[];
    readonly refused?: readonly RosterModelId[];
  },
): {
  readonly client: ModelClient;
  readonly asked: RosterModelId[];
  readonly requests: readonly ChatJsonRequest<unknown>[];
} {
  /**
   Seats asked, in call order.
   */
  const asked: RosterModelId[] = [];
  /**
   Requests seen, in call order, for cases that read the knobs on them.
   */
  const requests: ChatJsonRequest<unknown>[] = [];
  /**
   Seats that have already thrown once.
   */
  const thrown = new Set<RosterModelId>();
  return {
    asked,
    requests,
    client: {
      chatText: async () => {
        throw new Error('chatText unused',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        asked.push(request.modelId,);
        requests.push(request as ChatJsonRequest<unknown>,);
        if (refused.includes(request.modelId,)) {
          throw new NoProviderForModelError({
            modelId: request.modelId,
            reason: 'every provider serving this cat is out of budget',
          },);
        }
        if (failsAlways.includes(request.modelId,))
          throw new Error('scripted loss',);
        if (failsOnce.includes(request.modelId,) && (!thrown.has(request.modelId,))) {
          thrown.add(request.modelId,);
          throw new Error('scripted first loss',);
        }
        if (unreadable.includes(request.modelId,)) {
          return {
            kind: 'schema-mismatch',
            rawText: 'purr',
            detail: 'scripted unreadable reply',
          };
        }
        /**
         Scripted payload for the answering call.
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
    },
  };
}

/**
 Runs the windowed rounds over the bench with one scripted client.

 @param script - which seats fail or answer unreadably

 @param fanOut - window or whole bench, absent for the production default

 @param quorumOver - wider bench the quorum is taken over, absent for the
 seats asked

 @param maxAnswerChars - answer cap handed to every request, absent for none

 @param graceMs - how long a round waits for stragglers, the hang stop unless a
 case says otherwise

 @returns Outcomes, the quorum closed on and the seats counted out of reach,
 plus the seats asked in call order

 @example
 ```ts
 const { outcomes, asked, } = await runBench({ script: { failsOnce: [], failsAlways: [], unreadable: [], }, },);
 ```
 */
async function runBench(
  {
    script,
    fanOut,
    quorumOver,
    maxAnswerChars,
    graceMs = HANG_STOP_MS,
  }: {
    readonly script: Parameters<typeof scriptedClient>[0];
    readonly fanOut?: 'window' | 'whole-bench';
    readonly quorumOver?: number;
    readonly maxAnswerChars?: number;
    readonly graceMs?: number;
  },
) {
  const { client, asked, requests, } = scriptedClient(script,);
  const {
    outcomes,
    quorum,
    unreachable,
  } = await runWindowedRounds({
    client,
    modelIds: BENCH,
    messages: [{ role: 'user', content: 'meow?', },],
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: MEOW_FORMAT,
    validate: isMeowReply,
    stage: 'meow',
    l,
    graceMs,
    ...((fanOut === undefined) ? {} : { fanOut, }),
    ...((quorumOver === undefined) ? {} : { quorumOver, }),
    ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
  },);
  return {
    outcomes,
    quorum,
    unreachable,
    asked,
    requests,
  };
}

/**
 Seats the router refuses in the short-bench cases: four of the six.
 */
const FOUR_REFUSED: readonly RosterModelId[] = BENCH.slice(
  0,
  4,
);

await describe({
  name: runWindowedRounds.name,
  children: [
    it({
      name: 'CARRIES the maxAnswerChars knob on every request when the caller sets one, and carries no '
        + 'such field when it does not',
      fn: async () => {
        /**
         Runs with the knob and without it, and reads both request shapes.
         */
        const withKnob = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], },
          maxAnswerChars: 500,
        },);
        const withoutKnob = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], },
        },);
        /**
         Requests a healthy bench makes: its first window and no more.
         */
        const windowSize = firstRoundWindow({ benchSize: BENCH.length, },);
        expect(withKnob.requests.map(function capOf(request,): unknown {
          return request.maxAnswerChars;
        },),).toEqual(Array.from(
          { length: windowSize, },
          function cap(): number {
            return 500;
          },
        ),);
        expect(withoutKnob.requests.map(function carriesCap(request,): boolean {
          return Object.hasOwn(
            request,
            'maxAnswerChars',
          );
        },),).toEqual(Array.from(
          { length: windowSize, },
          function absent(): boolean {
            return false;
          },
        ),);
      },
    },),
    it({
      name: 'ASKS quorum plus one seat of a healthy bench and returns exactly those, in roster order, '
        + 'so the seats the window spared are neither heard nor lost',
      fn: async () => {
        const { outcomes, asked, } = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], },
        },);
        expect(asked,).toHaveLength(firstRoundWindow({ benchSize: BENCH.length, },),);
        expect(outcomes.map(function idOf(outcome,): RosterModelId {
          return outcome.modelId;
        },),).toEqual(BENCH.filter(function wasAsked(modelId,): boolean {
          return asked.includes(modelId,);
        },),);
        expect(outcomes.every(function heard(outcome,): boolean {
          return outcome.voice.heard;
        },),).toBe(true,);
      },
    },),
    it({
      name: 'RE-ASKS lost seats after the unasked ones, reaching quorum on their second answers, and '
        + 'reports every seat once, heard where a later ask heard it',
      fn: async () => {
        /**
         Every seat lost on its first ask, so no fresh seat can fill quorum
         and the retry rounds must come back to the lost ones.
         */
        const { outcomes, asked, } = await runBench({
          script: { failsOnce: BENCH, failsAlways: [], unreadable: [], },
        },);
        /**
         Seats reported, each once.
         */
        const reported = outcomes.map(function idOf(outcome,): RosterModelId {
          return outcome.modelId;
        },);
        expect(new Set(reported,).size,).toBe(reported.length,);
        expect(asked.length,).toBeGreaterThan(BENCH.length,);
        /**
         Voices heard, which reach quorum only through second asks.
         */
        const heard = outcomes.filter(function isHeard(outcome,): boolean {
          return outcome.voice.heard;
        },);
        expect(heard.length,).toBeGreaterThanOrEqual(QUORUM,);
      },
    },),
    it({
      name: 'REPORTS a bench lost every time once per seat, lost, after the retry rounds run out',
      fn: async () => {
        const { outcomes, asked, } = await runBench({
          script: { failsOnce: [], failsAlways: BENCH, unreadable: [], },
        },);
        expect(outcomes.map(function idOf(outcome,): RosterModelId {
          return outcome.modelId;
        },),).toEqual(BENCH,);
        expect(outcomes.every(function isLost(outcome,): boolean {
          return !outcome.voice.heard;
        },),).toBe(true,);
        expect(asked.length,).toBeGreaterThan(BENCH.length,);
      },
    },),
    it({
      name: 'NEVER re-asks a seat that answered unreadably, since it had its chance, and asks the whole '
        + 'bench once on request, as these stages always did',
      fn: async () => {
        const unreadableRun = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_HYPER_VISION, SEAT_HYPER_OPENROUTER_UNMEASURED,], },
        },);
        expect(new Set(unreadableRun.asked,).size,).toBe(unreadableRun.asked.length,);
        const wholeRun = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], },
          fanOut: 'whole-bench',
        },);
        expect(wholeRun.asked.toSorted(),).toEqual([...BENCH,].toSorted(),);
        expect(wholeRun.outcomes,).toHaveLength(BENCH.length,);
      },
    },),
    it({
      name: 'CLOSES ON THE REACHABLE SHARE when the router refuses seats (ledger X8; the short-bench rule '
        + 'of 2026-09-09): four of six refused leaves two that can answer, so the rounds close on those '
        + 'two and say the bench was short, where they used to chase a quorum of three',
      fn: async () => {
        const run = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], refused: FOUR_REFUSED, },
        },);
        expect(run.quorum,).toStrictEqual({
          needed: 2,
          reachable: 2,
          benchQuorum: QUORUM,
          short: true,
        },);
        expect(run.unreachable,).toBe(FOUR_REFUSED.length,);
        expect(run.outcomes.filter(function isHeard(outcome,): boolean {
          return outcome.voice.heard;
        },),).toHaveLength(2,);
      },
    },),
    it({
      name: 'COUNTS THE BENCH SEATS IT MAY NOT ASK AS OUT OF REACH, the way a confirmation asks only the '
        + 'seats the discovery asked (ledger E3): over a bench of eight with six asked, the quorum stays '
        + 'four while the asked seats can meet it and drops to the reachable share when they cannot',
      fn: async () => {
        /**
         Bench the quorum is taken over, two seats wider than the six asked.
         */
        const wider = BENCH.length + 2;
        // The whole bench is asked, so the seats the script refuses are the seats the round saw refused: a window of quorum plus
        // one, rotated by the prompt, may leave a refused seat unasked, and then the count of seats out of reach falls short of the seats the script refused.

        const oneRefused = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], refused: BENCH.slice(0, 1,), },
          quorumOver: wider,
          fanOut: 'whole-bench',
        },);
        expect(oneRefused.unreachable,).toBe(3,);
        expect(oneRefused.quorum.short,).toBe(false,);
        expect(oneRefused.quorum.needed,).toBe(4,);

        const threeRefused = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], refused: BENCH.slice(0, 3,), },
          quorumOver: wider,
          fanOut: 'whole-bench',
        },);
        expect(threeRefused.unreachable,).toBe(5,);
        expect(threeRefused.quorum,).toStrictEqual({
          needed: 2,
          reachable: 3,
          benchQuorum: 4,
          short: true,
        },);
      },
    },),
    it({
      name: 'SIZES THE WHOLE-BENCH ROUND THE SAME WAY, from the seats its one round saw refused',
      fn: async () => {
        const run = await runBench({
          script: { failsOnce: [], failsAlways: [], unreadable: [], refused: FOUR_REFUSED, },
          fanOut: 'whole-bench',
        },);
        expect(run.unreachable,).toBe(FOUR_REFUSED.length,);
        expect(run.quorum.short,).toBe(true,);
        expect(run.quorum.needed,).toBe(2,);
      },
    },),
  ],
},);
