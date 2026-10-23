/**
 Tests for the round boundary line, the only place a log says how long a
 fan-out took and how much of that was spent waiting after quorum already
 stood.

 THE TIMING WORK OPENED ON A LOG THAT COULD NOT ANSWER ITS OWN QUESTION.
 `doc/audit/every-volume-guard-is-blind-to-one-model.md` had to bound the
 straggler cost from above, at the grace window times the number of cut
 events, and recorded that confirming it "needs the dispatch timestamps the
 run does not currently record". These cases pin the line that records them.

 BOTH DIRECTIONS ARE COVERED, because only the pair shows the grace figure is
 a measurement rather than a constant: a round that loses a voice spends the
 whole window, and a round whose roster all answers spends almost none of it.

 NO CEILING ON THE CLOCK. A figure that must be small is held to a span the
 case reads itself around the same instants, on the same clock, at a turn of
 the event loop the round's own work cannot outlast: a stall widens both the
 figure and the span, and cannot put one past the other. Only floors are
 asserted against a delay (`mistake-prevention.md`, "Tests on the real
 clock").

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  EveryProviderDryError,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runGatherRound, } from '../dist/final/node/index.mjs';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  statusFailureLogText,
  statusFailureOf,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
  untilAborted,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

//region Fixtures

/**
 Grace the round that loses a voice waits out, short enough to finish the case
 in well under a second, standing in for the three real minutes. Every other
 round here ends before any window would close, and takes the hang stop.
 */
const GRACE_MS = 250;

/**
 Delay a slow-but-working voice takes: armed when the round asks it, before
 the round arms its grace, and shorter, so it lands inside the window on any
 machine.
 */
const SLOW_MS = 40;

/**
 Milliseconds a measured wait may fall short of the delay that produced it.

 NOT A TOLERANCE ON THE BEHAVIOUR, a tolerance on the CLOCK. `stage-round.ts`
 reads both ends of every figure with `monotonicMs`, which floors to whole
 milliseconds, and Node's timer list may fire a delay fractionally early. The
 two together let a 40 ms wait report 39, which this suite did on 2026-08-25
 under load while the rest of the package was building.

 Two rather than one, because each end can lose a fraction and the early fire
 is its own. It leaves every floor here far above the figure it has to be
 distinguished from, which is a round that did not wait at all.
 */
const CLOCK_SLACK_MS = 2;

/**
 Roster the rounds ask, named from the catalog because model identifiers are
 never invented.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 Client answering each model on its own schedule: at once, after a delay, or
 never until the round cuts it.

 The never-answering arm has NO TIMER OF ITS OWN, deliberately. A stub that
 also gave up after some duration would report a grace window whether or not
 the cut ever reached the call, which is the one thing these cases measure.

 @param slowModelId - model that answers after {@link SLOW_MS}

 @param hangingModelId - model that answers only when the round abandons it

 @param answeredAt - clock readings of each answer as it is returned, in
 answer order, so a case can anchor the round's figures on when voices really
 answered rather than on a delay a loaded machine stretches

 @returns Client the round can drive

 @example
 ```ts
 const client = scheduledClient({ slowModelId, hangingModelId, },);
 ```
 */
function scheduledClient(
  {
    slowModelId,
    hangingModelId,
    answeredAt = [],
  }: {
    readonly slowModelId?: RosterModelId;
    readonly hangingModelId?: RosterModelId;
    readonly answeredAt?: number[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.modelId === hangingModelId) {
        await untilAborted({ signal: request.signal, },);
        throw new Error('cut by the round',);
      }
      if (request.modelId === slowModelId)
        await wait(SLOW_MS,);

      /**
       Scripted payload for the answering call.
       */
      const scripted: unknown = { meow: request.modelId, };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      answeredAt.push(performance.now(),);
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
 Client answering every seat at once but one, whose call fails at once.

 @param failingModelId - seat whose call fails before any answer, a voice the
 round loses without waiting

 @returns Client the round can drive

 @example
 ```ts
 const client = clientFailingOne({ failingModelId: SEAT_SYNTHETIC_VISION_WITHHELD, },);
 ```
 */
function clientFailingOne({ failingModelId, }: { readonly failingModelId: RosterModelId; },): ModelClient {
  /**
   Client answering every seat at once.
   */
  const answering = scheduledClient({},);
  return {
    ...answering,
    chatJson: async function failingOne<ValueT,>(
      request: Parameters<typeof answering.chatJson>[0],
    ): Promise<Awaited<ReturnType<typeof answering.chatJson<ValueT>>>> {
      if (request.modelId === failingModelId)
        throw new Error('refused at once',);
      return await answering.chatJson(request as Parameters<typeof answering.chatJson<ValueT>>[0],);
    },
  };
}

/**
 Client whose first two seats wait for the caller's abort and then each fail,
 the way a call whose own deadline or transport failure lands as the caller
 aborts does; the third seat answers at once.

 @param failFirst - ends the first seat's call once the abort reached it,
 handed that call's signal so a case can fail it with the reason it carries

 @param failSecond - ends the second seat's call the same way, so a case
 chooses which of the two failures the round meets first

 @returns Client the round can drive

 @example
 ```ts
 const client = clientFailingAfterTheStop({ failFirst: refuseLate, failSecond: refuseSoon, },);
 ```
 */
function clientFailingAfterTheStop(
  {
    failFirst,
    failSecond,
  }: {
    readonly failFirst: (signal: AbortSignal,) => Promise<never>;
    readonly failSecond: (signal: AbortSignal,) => Promise<never>;
  },
): ModelClient {
  /**
   Client answering every seat at once.
   */
  const answering = scheduledClient({},);
  return {
    ...answering,
    chatJson: async function failingAfterTheStop<ValueT,>(
      request: Parameters<typeof answering.chatJson>[0],
    ): Promise<Awaited<ReturnType<typeof answering.chatJson<ValueT>>>> {
      if (request.modelId === SEAT_HYPER_OPENROUTER_VISION_EDITOR) {
        await untilAborted({ signal: request.signal, },);
        return await failFirst(request.signal,);
      }
      if (request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER) {
        await untilAborted({ signal: request.signal, },);
        return await failSecond(request.signal,);
      }
      return await answering.chatJson(request as Parameters<typeof answering.chatJson<ValueT>>[0],);
    },
  };
}

/**
 Ends a call with the reason its own signal carries, which is the caller's
 forwarded reason when the caller stopped the round.

 @param signal - signal of the call, aborted by the time this runs

 @returns Never; it rejects with the signal's reason

 @throws The signal's reason, the very value the caller aborted with

 @example
 ```ts
 const client = clientFailingAfterTheStop({ failFirst: withTheCallersReason, failSecond: withTheCallersReason, },);
 ```
 */
async function withTheCallersReason(signal: AbortSignal,): Promise<never> {
  signal.throwIfAborted();
  throw new Error('unreachable: a call that waited for its abort was handed a signal that had not aborted',);
}

/**
 Line the round writes for a seat whose call failed of its own as the caller
 stopped the round.

 @param seat - seat whose call failed

 @param failure - that failure as a log line renders it

 @returns The seat's line, as the round writes it

 @example
 ```ts
 const line = stoppedSeatLine({ seat: SEAT_HYPER_OPENROUTER_VISION_EDITOR, failure: 'refused by RangeError', },);
 ```
 */
function stoppedSeatLine(
  {
    seat,
    failure,
  }: {
    readonly seat: RosterModelId;
    readonly failure: string;
  },
): string {
  return `cat-stage ${seat}: ${failure}, as the caller stopped the round`;
}

/**
 Runs a round whose first two seats fail once the caller stops it and whose
 third answers at once, stops it with the caller's reason, and hands back
 what it rejected with once every ask has ended.

 @param failFirst - ends the first seat's call once the stop reached it

 @param failSecond - ends the second seat's call the same way

 @param stop - whether the stop arrives while no quorum stands, every seat
 being needed, or once the third seat's answer made a quorum of one stand and
 the round waits out its grace

 @param reason - what the caller stops the round with

 @param said - array the round's lines land in

 @returns What the round rejected with

 @example
 ```ts
 const refusal = await stoppedRound({ failFirst, failSecond, stop: 'after quorum', reason, said, },);
 ```
 */
async function stoppedRound(
  {
    failFirst,
    failSecond,
    stop,
    reason,
    said,
  }: {
    readonly failFirst: (signal: AbortSignal,) => Promise<never>;
    readonly failSecond: (signal: AbortSignal,) => Promise<never>;
    readonly stop: 'before quorum' | 'after quorum';
    readonly reason: Error;
    readonly said: string[];
  },
): Promise<unknown> {
  /**
   The caller's steering.
   */
  const steering = new AbortController();
  /**
   The round, started and left waiting on the first two seats.
   */
  const round = runGatherRound({
    client: clientFailingAfterTheStop({
      failFirst,
      failSecond,
    },),
    modelIds: ROSTER,
    messages: [{ role: 'user', content: 'meow', },],
    signal: steering.signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    responseFormat: MEOW_FORMAT,
    validate: isMeowReply,
    stage: 'cat-stage',
    l: capturingLogger({ messages: said, },),
    heardNeeded: (stop === 'after quorum') ? 1 : ROSTER.length,
    graceMs: HANG_STOP_MS,
  },);
  // The third seat's answer and the round's count of it take microtasks
  // alone, so a turn of the event loop leaves quorum standing and the round
  // waiting out its grace when the stop arrives.
  if (stop === 'after quorum')
    await turnOfTheLoop();
  steering.abort(reason,);
  /**
   What the round rejected with.
   */
  const refusal = await rejectionOf(async function stopped(): Promise<unknown> {
    return await round;
  },);
  // A seat that ends after the ask the round rejected on writes its line as it
  // ends. No scripted failure waits on anything but a microtask, so a turn of
  // the event loop later every ask has ended.
  await turnOfTheLoop();
  return refusal;
}

/**
 Refusal the real client raises over the real transport, typed as the error
 it is so a seat can end in it.

 @param sinon - calling case's own sandbox, which stubs `fetch` until it is restored

 @returns The provider's refusal, its key masked by the transport

 @throws Error when the client raised something other than an error

 @example
 ```ts
 const failure = await providerRefusalOf({ sinon: ctx.sinon, },);
 ```
 */
async function providerRefusalOf({ sinon, }: { readonly sinon: DisposableSandbox; },): Promise<Error> {
  /**
   What the exchange the invented provider refused raised.
   */
  const failure = await statusFailureOf({
    sinon,
    status: 401,
  },);
  if (!Error.isError(failure,))
    throw new Error('the exchange the invented provider refused raised something other than an error',);
  return failure;
}

/**
 Numbers the round line carries, pulled back out of it.
 */
type RoundTimings = {
  readonly heard: number;
  readonly asked: number;
  readonly totalMs: number;
  readonly toQuorumMs: number;
  readonly inGraceMs: number;
};

/**
 Reads the round line back into its numbers.

 SCANNED RATHER THAN MATCHED. A pattern would accept a line whose fields had
 drifted into a different order and would say nothing useful when the line is
 absent, while splitting on the separators names the missing field.

 @param said - every message the logger kept

 @returns Numbers the single round line carried

 @throws Error when no round line was logged, or its fields are not readable

 @example
 ```ts
 const timings = readRoundLine({ said, },);
 ```
 */
function readRoundLine({ said, }: { readonly said: readonly string[]; },): RoundTimings {
  /**
   Lines that look like a round report.
   */
  const rounds = said.filter(function isRound(message,): boolean {
    return message.includes(' round: ',) && message.includes('ms in grace',);
  },);
  if (rounds.length !== 1)
    throw new Error(`expected exactly one round line, got ${String(rounds.length,)}`,);

  /**
   Fields of that line, in the order it writes them.
   */
  const fields = (rounds[0] ?? '')
    .split(', ',)
    .map(function trim(field,): string {
      return field.trim();
    },);

  /**
   Reads one field's leading number, so a renamed field fails loudly.
   */
  const numberIn = (
    { index, expect: expected, }: { readonly index: number; readonly expect: string; },
  ): number => {
    /**
     Field text at that position.
     */
    const field = fields[index] ?? '';
    if (!field.includes(expected,))
      throw new Error(`field ${String(index,)} should mention ${expected}, reads "${field}"`,);
    // Every field but the first reads `<number>ms <name>`, so splitting on the
    // unit yields the digits alone and the parse needs neither a pattern nor
    // `parseInt`'s habit of stopping wherever the digits run out.
    return Number((field.split('ms ',)[0] ?? ''),);
  };

  /**
   First field with its stage label removed, leaving the ratio.
   */
  const afterStage = (fields[0] ?? '')
    .split(' round: ',)
    .at(-1,) ?? '';

  /**
   Heard and asked counts, which the first field carries as a ratio.
   */
  const counts = afterStage
    .split(' ',)[0]
    ?.split('/',) ?? [];

  return {
    heard: Number(counts[0],),
    asked: Number(counts[1],),
    totalMs: numberIn({ index: 1, expect: 'ms total', },),
    toQuorumMs: numberIn({ index: 2, expect: 'ms to quorum', },),
    inGraceMs: numberIn({ index: 3, expect: 'ms in grace', },),
  };
}

//endregion Fixtures

await describe({
  name: '',
  // ONE CASE AT A TIME, as set when the case "SEPARATES THE TIME A ROUND
  // WORKED" held the round's quorum mark to a ceiling past its first answer and
  // failed in 2 of 4 runs beside the stop cases, which build a real client over
  // the stubbed transport (commit `154da3c68`). It now reads the mark against
  // the turn of the event loop after the round starts, which work beside it
  // widens and cannot break; the setting stays as that change left it.
  concurrency: 1,
  children: [
    describe({
      name: runGatherRound.name,
      children: [
        it({
          name: 'SEPARATES THE TIME A ROUND WORKED FROM THE TIME IT WAITED, so a straggler cost is '
            + 'read off the log instead of bounded above at the whole grace window times the number '
            + 'of cut events, which is all the old log was found able to support',
          fn: async () => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             Clock before the round starts, at or before its own start mark.
             */
            const startedAt = performance.now();

            /**
             The round, every seat already asked: the first seat answered inside
             this call, and the round marks quorum in the microtasks that answer
             starts.
             */
            const round = runGatherRound({
              client: scheduledClient({
                slowModelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                hangingModelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              },),
              modelIds: ROSTER,
              messages: [{ role: 'user', content: 'meow', },],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: said, },),
              heardNeeded: 1,
              graceMs: GRACE_MS,
            },);
            // Every microtask runs before the event loop turns, so this turn
            // comes after the round's quorum mark however long the machine
            // stalls in between, and before the slow voice's timer or the
            // window could have moved the mark.
            await turnOfTheLoop();
            /**
             Milliseconds from before the round started to the next turn of the
             event loop, a span the round's own time to quorum lies inside.
             */
            const toNextTurnMs = performance.now() - startedAt;
            await round;

            /**
             What the round said about itself.
             */
            const timings = readRoundLine({ said, },);

            expect(timings.heard,).toBe(2,);
            expect(timings.asked,).toBe(ROSTER.length,);
            // The window really was spent: the hanging voice never answered, so
            // the round waited it out rather than finishing at quorum.
            expect(timings.inGraceMs,).toBeGreaterThanOrEqual(GRACE_MS - CLOCK_SLACK_MS,);
            // Quorum stood on the voice that answered at once, so the round did
            // no waiting before it: its mark lies before the next turn. Held to
            // that span rather than to a ceiling past the first answer, which
            // failed in 2 of 4 runs beside the stop cases (commit `154da3c68`).
            // The round floors both of its readings and starts no earlier than
            // the case's, so its figure is at most the span's ceiling.
            expect(timings.toQuorumMs,).toBeLessThanOrEqual(Math.ceil(toNextTurnMs,),);
            // The three numbers describe one round rather than three measurements.
            expect(timings.totalMs,).toBe(timings.toQuorumMs + timings.inGraceMs,);
          },
        },),

        it({
          name: 'REPORTS A GRACE OF NEARLY NOTHING WHEN THE WHOLE ROSTER ANSWERS, which is what makes '
            + 'the other case evidence: a figure that read as the full window either way would be a '
            + 'constant wearing a measurement\'s name',
          fn: async () => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             When each voice really answered, the slow voice last.
             */
            const answeredAt: number[] = [];

            /**
             The round, every seat already asked, the slow voice's wait among
             them.
             */
            const round = runGatherRound({
              client: scheduledClient({
                slowModelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                answeredAt,
              },),
              modelIds: ROSTER,
              messages: [{ role: 'user', content: 'meow', },],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: said, },),
              heardNeeded: ROSTER.length,
              graceMs: HANG_STOP_MS,
            },);
            // Armed after the slow voice's own wait and as long, so it fires
            // after that voice answered and after every microtask its answer
            // started, however long the machine stalls (measured 2026-10-10 on
            // Node 26.10.0, with stalls of 300 and 1,100 ms).
            await wait(SLOW_MS,);
            /**
             Milliseconds from the last answer to this turn, a span the round's
             grace lies inside when it ended in that answer's microtasks.
             */
            const sinceLastAnswerMs = performance.now() - nonNullishOrThrow(answeredAt.at(-1,),);

            /**
             What the round said about itself, read at this turn: the line is
             written as the round ends, so a round still waiting on a window has
             none yet.
             */
            const timings = readRoundLine({ said, },);
            await round;

            expect(timings.heard,).toBe(ROSTER.length,);
            // The grace is a measurement of that span, not the window: the round
            // floors both of its readings and takes them after the last answer
            // and before this turn, so its figure is at most the span's ceiling.
            expect(timings.inGraceMs,).toBeLessThanOrEqual(Math.ceil(sinceLastAnswerMs,),);
            // The slow voice is what the round waited on, and it waited before
            // quorum rather than after it.
            expect(timings.toQuorumMs,).toBeGreaterThanOrEqual(SLOW_MS - CLOCK_SLACK_MS,);
            expect(timings.totalMs,).toBe(timings.toQuorumMs + timings.inGraceMs,);
          },
        },),

        it({
          name: 'REJECTS WITH THE CALLER\'S OWN ABORT REASON before quorum when two asks rethrow failures of their own '
            + 'after the abort and the later-listed ask ends first',
          fn: async () => {
            /**
             The caller's steering, aborted once every ask is in flight.
             */
            const steering = new AbortController();
            /**
             Why the caller stopped the round.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The two failures the asks end in, the later-listed one first.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            /**
             The round, started and left waiting on the first two seats.
             */
            const round = runGatherRound({
              client: clientFailingAfterTheStop({
                failFirst: async function timedOut(): Promise<never> {
                  return await refuseAfterThat(new Error('the first seat timed out as the stop arrived',),);
                },
                failSecond: async function disconnected(): Promise<never> {
                  return await refuseAtOnce(new Error('the second seat lost its connection as the stop arrived',),);
                },
              },),
              modelIds: ROSTER,
              messages: [{ role: 'user', content: 'meow', },],
              signal: steering.signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: [], },),
              heardNeeded: ROSTER.length,
              graceMs: HANG_STOP_MS,
            },);
            steering.abort(reason,);
            expect(await rejectionOf(async function stopped(): Promise<unknown> {
              return await round;
            },),).toBe(reason,);
          },
        },),

        it({
          name: 'LOGS EACH FAILURE THE ASKS END IN AFTER THE CALLER\'S ABORT, the one that ends after the round '
            + 'rejected as well, on a line naming its seat, and rejects with the caller\'s own reason in their place',
          fn: async () => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             Why the caller stopped the round.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The two failures the asks end in, the later-listed one first.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            expect(await stoppedRound({
              failFirst: async function outOfTime(): Promise<never> {
                return await refuseAfterThat(new Error('the first seat ran out of time as the owner called',),);
              },
              failSecond: async function dropped(): Promise<never> {
                return await refuseAtOnce(new RangeError('the second seat dropped its line as the owner called',),);
              },
              stop: 'before quorum',
              reason,
              said,
            },),).toBe(reason,);
            expect(said,).toEqual([
              stoppedSeatLine({
                seat: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                failure: 'refused by RangeError',
              },),
              stoppedSeatLine({
                seat: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                failure: 'refused by Error',
              },),
            ],);
          },
        },),

        it({
          name: 'LOGS A PROVIDER REFUSAL AN ASK ENDS IN AFTER THE CALLER\'S ABORT by its seat, the status and the '
            + 'provider\'s words, the key its refusal echoed masked, once, where it ends before an ask ending in the '
            + 'caller\'s own reason',
          fn: async ctx => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             Why the caller stopped the round.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Refusal the real client raised over the real transport.
             */
            const failure = await providerRefusalOf({ sinon: ctx.sinon, },);
            /**
             The refusal first, then the caller's own reason.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            expect(await stoppedRound({
              failFirst: async function refused(): Promise<never> {
                return await refuseAtOnce(failure,);
              },
              failSecond: async function stopped(): Promise<never> {
                return await refuseAfterThat(reason,);
              },
              stop: 'before quorum',
              reason,
              said,
            },),).toBe(reason,);
            expect(said,).toEqual([
              stoppedSeatLine({
                seat: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                failure: statusFailureLogText({ provider: 'synthetic', status: 401, },),
              },),
            ],);
          },
        },),

        it({
          name: 'LOGS A PROVIDER REFUSAL AN ASK ENDS IN AFTER ANOTHER ASK ENDED IN THE CALLER\'S OWN REASON, by its '
            + 'seat, the status and the provider\'s words, where the round rejecting on that reason wrote no line for '
            + 'the refusal',
          fn: async ctx => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             Why the caller stopped the round.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Refusal the real client raised over the real transport.
             */
            const failure = await providerRefusalOf({ sinon: ctx.sinon, },);
            /**
             The caller's own reason first, then the refusal.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            expect(await stoppedRound({
              failFirst: async function refused(): Promise<never> {
                return await refuseAfterThat(failure,);
              },
              failSecond: async function stopped(): Promise<never> {
                return await refuseAtOnce(reason,);
              },
              stop: 'before quorum',
              reason,
              said,
            },),).toBe(reason,);
            expect(said,).toEqual([
              stoppedSeatLine({
                seat: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                failure: statusFailureLogText({ provider: 'synthetic', status: 401, },),
              },),
            ],);
          },
        },),

        it({
          name: 'LOGS A PROVIDER REFUSAL AN ASK ENDS IN WHEN THE CALLER STOPS THE ROUND AFTER QUORUM STOOD, by its '
            + 'seat, the status and the provider\'s words, where the wait on the grace window swallowed it unlogged',
          fn: async ctx => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             Why the caller stopped the round.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Refusal the real client raised over the real transport.
             */
            const failure = await providerRefusalOf({ sinon: ctx.sinon, },);
            /**
             The refusal first, then the caller's own reason.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            expect(await stoppedRound({
              failFirst: async function refused(): Promise<never> {
                return await refuseAtOnce(failure,);
              },
              failSecond: async function stopped(): Promise<never> {
                return await refuseAfterThat(reason,);
              },
              stop: 'after quorum',
              reason,
              said,
            },),).toBe(reason,);
            expect(said,).toEqual([
              stoppedSeatLine({
                seat: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                failure: statusFailureLogText({ provider: 'synthetic', status: 401, },),
              },),
            ],);
          },
        },),

        it({
          name: 'LOGS NOTHING when the asks end in the caller\'s own abort reason itself, which the round rejects with, '
            + 'so no failure is dropped',
          fn: async () => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            /**
             The caller's steering, aborted once every ask is in flight.
             */
            const steering = new AbortController();
            /**
             Why the caller stopped the round.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The round, started and left waiting on the first two seats.
             */
            const round = runGatherRound({
              client: clientFailingAfterTheStop({
                failFirst: withTheCallersReason,
                failSecond: withTheCallersReason,
              },),
              modelIds: ROSTER,
              messages: [{ role: 'user', content: 'meow', },],
              signal: steering.signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: said, },),
              heardNeeded: ROSTER.length,
              graceMs: HANG_STOP_MS,
            },);
            steering.abort(reason,);
            expect(await rejectionOf(async function stopped(): Promise<unknown> {
              return await round;
            },),).toBe(reason,);
            expect(said,).toEqual([],);
          },
        },),

        it({
          name: 'REJECTS WITH THE EVERY-PROVIDER-DRY REFUSAL, unchanged, although nobody aborted, instead of recording '
            + 'the seat as an abandoned straggler and returning a thin roster',
          fn: async () => {
            /**
             The refusal the router raised for one seat.
             */
            const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
            /**
             Client answering every seat at once.
             */
            const answering = scheduledClient({},);
            /**
             Every message the round logged.
             */
            const said: string[] = [];

            expect(await rejectionOf(async function dryRound(): Promise<unknown> {
              return await runGatherRound({
                client: {
                  ...answering,
                  chatJson: async function dryOne<ValueT,>(
                    request: Parameters<typeof answering.chatJson>[0],
                  ): Promise<Awaited<ReturnType<typeof answering.chatJson<ValueT>>>> {
                    if (request.modelId === SEAT_SYNTHETIC_VISION_WITHHELD)
                      throw dry;
                    return await answering.chatJson(request as Parameters<typeof answering.chatJson<ValueT>>[0],);
                  },
                },
                modelIds: ROSTER,
                messages: [{ role: 'user', content: 'meow', },],
                signal: new AbortController().signal,
                exchangeTimeoutMs: HANG_STOP_MS,
                responseFormat: MEOW_FORMAT,
                validate: isMeowReply,
                stage: 'cat-stage',
                l: capturingLogger({ messages: said, },),
                heardNeeded: ROSTER.length,
                graceMs: HANG_STOP_MS,
              },);
            },),).toBe(dry,);
            expect(said.filter(function lost(message,): boolean {
              return message.includes('voice lost',);
            },),).toEqual([],);
          },
        },),

        it({
          name: 'REJECTS WITH THE EVERY-PROVIDER-DRY REFUSAL that lands inside the grace window after quorum stood, '
            + 'although nobody aborted, instead of recording the seat as a silent position and returning a thin roster',
          fn: async () => {
            /**
             The refusal the router raised for one seat, after the others had made quorum.
             */
            const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
            /**
             Client answering every seat at once; the seat that meets the dry router does so a turn of the event loop
             later, after the seats that answered in microtasks have made quorum.
             */
            const answering = scheduledClient({},);
            /**
             Every message the round logged.
             */
            const said: string[] = [];

            expect(await rejectionOf(async function dryInTheWindow(): Promise<unknown> {
              return await runGatherRound({
                client: {
                  ...answering,
                  chatJson: async function dryLate<ValueT,>(
                    request: Parameters<typeof answering.chatJson>[0],
                  ): Promise<Awaited<ReturnType<typeof answering.chatJson<ValueT>>>> {
                    if (request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER) {
                      await turnOfTheLoop();
                      throw dry;
                    }
                    return await answering.chatJson(request as Parameters<typeof answering.chatJson<ValueT>>[0],);
                  },
                },
                modelIds: ROSTER,
                messages: [{ role: 'user', content: 'meow', },],
                signal: new AbortController().signal,
                exchangeTimeoutMs: HANG_STOP_MS,
                responseFormat: MEOW_FORMAT,
                validate: isMeowReply,
                stage: 'cat-stage',
                l: capturingLogger({ messages: said, },),
                heardNeeded: 1,
                graceMs: HANG_STOP_MS,
              },);
            },),).toBe(dry,);
          },
        },),

        it({
          name: 'CONTAINS A MODEL-DARK REFUSAL, in the first ask and in the grace window after quorum, as that seat\'s '
            + 'unreachable voice, and resolves with the others heard instead of rejecting as the dry refusal does',
          fn: async () => {
            /**
             Runs one round whose dark seat is refused at once, or a turn of the event loop later.

             @param late - whether the refusal lands after the seats that answered in microtasks have made quorum
             */
            async function darkRound({ late, }: { readonly late: boolean; },): Promise<void> {
            /**
             Client answering every seat at once, but for the dark one.
             */
              const answering = scheduledClient({},);
            /**
             Every message the round logged.
             */
              const said: string[] = [];
            /**
             What the round settled with: the dark seat refused at once, or a turn of the event loop later.
             */
              const outcomes = await runGatherRound({
                client: {
                  ...answering,
                  chatJson: async function darkOne<ValueT,>(
                    request: Parameters<typeof answering.chatJson>[0],
                  ): Promise<Awaited<ReturnType<typeof answering.chatJson<ValueT>>>> {
                    if (request.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER) {
                      if (late)
                        await turnOfTheLoop();
                      throw modelDarkRefusal({ modelId: request.modelId, },);
                    }
                    return await answering.chatJson(request as Parameters<typeof answering.chatJson<ValueT>>[0],);
                  },
                },
                modelIds: ROSTER,
                messages: [{ role: 'user', content: 'meow', },],
                signal: new AbortController().signal,
                exchangeTimeoutMs: HANG_STOP_MS,
                responseFormat: MEOW_FORMAT,
                validate: isMeowReply,
                stage: 'cat-stage',
                l: capturingLogger({ messages: said, },),
                heardNeeded: 1,
                graceMs: HANG_STOP_MS,
              },);
              expect(outcomes.find(function dark(outcome,): boolean {
                return outcome.modelId === SEAT_SYNTHETIC_VISION_NO_OPENROUTER;
              },)?.voice,).toMatchObject({ heard: false, unreachable: true, },);
              expect(outcomes.filter(function heard(outcome,): boolean {
                return outcome.voice.heard;
              },).length,).toBeGreaterThan(0,);
              expect(said.some(function seatUnreachable(message,): boolean {
                return message.includes('seat unreachable',);
              },),).toBe(true,);
            }
            await darkRound({ late: false, },);
            await darkRound({ late: true, },);
          },
        },),

        it({
          name: 'REJECTS WITH A LOGGER\'S OWN FAILURE, unchanged, when no abort is in play and a lost voice cannot be '
            + 'logged',
          fn: async () => {
            /**
             What the logger raises on every warning.
             */
            const logFailure = new Error('the cat sat on the log',);
            expect(await rejectionOf(async function unlogged(): Promise<unknown> {
              return await runGatherRound({
                client: clientFailingOne({ failingModelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),
                modelIds: ROSTER,
                messages: [{ role: 'user', content: 'meow', },],
                signal: new AbortController().signal,
                exchangeTimeoutMs: HANG_STOP_MS,
                responseFormat: MEOW_FORMAT,
                validate: isMeowReply,
                stage: 'cat-stage',
                l: {
                  ...capturingLogger({ messages: [], },),
                  warn: function refusesToWarn(): void {
                    throw logFailure;
                  },
                },
                heardNeeded: ROSTER.length,
                graceMs: HANG_STOP_MS,
              },);
            },),).toBe(logFailure,);
          },
        },),
      ],
    },),

    describe({
      name: 'the round line when quorum never stood (ledger P12)',
      children: [
        it({
          name: 'SAYS NO QUORUM STOOD rather than timing a quorum that never did: "select round: 3/5 heard, 18766ms '
            + 'total, 18766ms to quorum" was logged for a round that needed four voices',
          fn: async () => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            await runGatherRound({
              client: clientFailingOne({ failingModelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),
              modelIds: ROSTER,
              messages: [{ role: 'user', content: 'meow', },],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: said, },),
              heardNeeded: ROSTER.length,
              graceMs: HANG_STOP_MS,
            },);
            /**
             The round's own line.
             */
            const lines = said.filter(function isRound(message,): boolean {
              return message.includes(' round: ',);
            },);
            expect({
              lines: lines.length,
              noQuorum: lines.some(function says(line,): boolean {
                return line.includes(`no quorum (${String(ROSTER.length - 1,)} of ${String(ROSTER.length,)} needed)`,);
              },),
              claimsQuorum: lines.some(function claims(line,): boolean {
                return line.includes('to quorum',);
              },),
            },).toEqual({
              lines: 1,
              noQuorum: true,
              claimsQuorum: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'the round line on a clock the system time cannot move (ledger B78)',
      children: [
        it({
          name: 'TIMES THE ROUND\'S OWN MILLISECONDS when the system clock is set forward two hours as the round '
            + 'starts, where the line said the round took two hours',
          fn: async ctx => {
            /**
             Every message the round logged.
             */
            const said: string[] = [];
            const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);

            /**
             The round, its start already read: nothing in it waits before that.
             */
            const round = runGatherRound({
              client: scheduledClient({},),
              modelIds: ROSTER,
              messages: [{ role: 'user', content: 'meow', },],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: said, },),
              heardNeeded: ROSTER.length,
              graceMs: HANG_STOP_MS,
            },);
            wall.step({ byMs: 2 * HOUR_MS, },);
            await round;

            /**
             What the round said about itself.
             */
            const timings = readRoundLine({ said, },);
            expect(timings.totalMs,).toBeLessThan(HOUR_MS,);
            expect(timings.totalMs,).toBe(timings.toQuorumMs + timings.inGraceMs,);
          },
        },),
      ],
    },),
  ],
},);
