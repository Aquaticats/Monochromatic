/**
 Tests for counting an abandoned call that delivered nothing as a failed ask of its seat.

 In CuspariaKLSY22, 33 of `inception/mercury-2.5`'s 69 streams ended in the round's grace cut with `0 content chars, 0
 reasoning chars`, 33 rounds carried 100 s or more of grace for them (3,960 s summed), and `zheermao22` had 11. The
 tally counted every abandoned call in no column of asks (ledger B421, B687), so the seat could not be withheld
 however often its streams were dead, and each round waited it out again.

 The rule these cases pin: a call the round's own grace window cut, that had delivered no answer characters and no
 reasoning characters, is a failed ask of the SEAT, counted in `asked` and in its own `silent` column, and the withholding
 rule that already stood (every one of at least `minAsks` asks failed) applies to it. A call cut with content on its
 way, a call the caller or a deadline aborted, and a call that was not cut stay where they were counted.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { setImmediate as turnOfTheLoop, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { cutStreamError, } from '@monochromatic-dev/translation-repair-provider-stream/ts/cut-stream.test-fixture.ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import {
  createSeatTally,
  runGatherRound,
  seatReportLines,
  seatTallyClient,
  StragglerCutError,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  isMeowReply,
  MEOW_FORMAT,
  untilAborted,
} from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

//region Fixtures

/**
 Client whose every call throws what it is handed.

 @param failure - what each call throws

 @returns Client with the full surface

 @example
 ```ts
 const inner = throwingClient({ failure: cutStreamError({ firstByte: 'arrived', }), },);
 ```
 */
function throwingClient({ failure, }: { readonly failure: unknown; },): ModelClient {
  return {
    chatText: async () => {
      throw failure;
    },
    chatJson: async () => {
      throw failure;
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Signal the round's grace cut aborted, as `runGatherRound` aborts it.

 @returns Signal already aborted with the straggler cut's reason

 @example
 ```ts
 const signal = signalCutByTheRound();
 ```
 */
function signalCutByTheRound(): AbortSignal {
  /**
   Controller the cut aborts.
   */
  const controller = new AbortController();
  controller.abort(new StragglerCutError(),);
  return controller.signal;
}

/**
 Asks the seat once over a client that throws, swallowing the throw.

 @param inner - client the tally wraps

 @param tally - where the counts go

 @param signal - signal the call carries

 @returns Nothing; the call's throw is the thing under test and is read off the tally

 @example
 ```ts
 await askOnce({ inner, tally, signal, },);
 ```
 */
async function askOnce(
  {
    inner,
    tally,
    signal,
  }: {
    readonly inner: ModelClient;
    readonly tally: ReturnType<typeof createSeatTally>;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  await rejectionOf(async function ask(): Promise<void> {
    await seatTallyClient({
      inner,
      tally,
      minAsks: 4,
    },).chatText({
      modelId: SEAT_HYPER_VISION,
      messages: [{ role: 'user', content: 'meow', },],
      signal,
    },);
  },);
}

//endregion Fixtures

await describe({
  name: 'seat tally: abandoned calls that delivered nothing',
  children: [
    describe({
      name: 'the tally',
      children: [
        it({
          name: 'COUNTS A SILENT CALL AS AN ASK THAT FAILED, in asked and in silent and in no other column',
          fn: async () => {
            /** Tally under test. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            expect(tally.counts(),).toStrictEqual([{
              modelId: SEAT_HYPER_VISION,
              asked: 1,
              usable: 0,
              unusable: 0,
              threw: 0,
              silent: 1,
              abandoned: 0,
              unreachable: 0,
            },],);
          },
        },),

        it({
          name: 'WITHHOLDS A SEAT WHOSE EVERY ASK FAILED, whether it threw or ended with nothing delivered, once it was '
            + 'asked the given number of times',
          fn: async () => {
            /** Tally holding four silent asks of one seat and two thrown, two silent asks of another. */
            const tally = createSeatTally();
            for (let ask = 0; ask < 4; ask += 1)
              tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'threw', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'silent', },);
            tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'silent', },);
            expect(tally.withheld({ minAsks: 4, },).map(function idOf(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],);
          },
        },),

        it({
          name: 'DOES NOT WITHHOLD A SEAT THAT ANSWERED ONCE, however many silent asks it has, and not before the given '
            + 'number of asks: the rule stays "every ask failed"',
          fn: async () => {
            /** Tally holding a seat that answered once among many silent asks, and a seat with three silent asks. */
            const tally = createSeatTally();
            for (let ask = 0; ask < 9; ask += 1)
              tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'usable', },);
            for (let ask = 0; ask < 3; ask += 1)
              tally.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'silent', },);
            expect(tally.withheld({ minAsks: 4, },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'KEEPS A CALL CUT WITH CONTENT OUT OF THE COUNT: four abandoned calls withhold nothing, as before',
          fn: async () => {
            /** Tally holding four abandoned calls of one seat. */
            const tally = createSeatTally();
            for (let ask = 0; ask < 4; ask += 1)
              tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'abandoned', },);
            expect(tally.withheld({ minAsks: 4, },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'NAMES A SEAT DARK WHOSE ONLY ASKS WERE SILENT, since it was asked and never produced anything usable',
          fn: async () => {
            /** Tally holding two silent asks. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            expect(tally.dark().map(function idOf(count,): string {
              return count.modelId;
            },),).toStrictEqual([SEAT_HYPER_VISION,],);
          },
        },),

        it({
          name: 'WRITES THE SILENT COUNT ON THE SEAT LINE between threw and abandoned',
          fn: async () => {
            /** Tally holding two silent asks and one abandoned call. */
            const tally = createSeatTally();
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'silent', },);
            tally.record({ modelId: SEAT_HYPER_VISION, outcome: 'abandoned', },);
            expect(seatReportLines({ tally, },)[0],).toBe(
              `SEAT ${SEAT_HYPER_VISION} asked=2 usable=0 unusable=0 threw=0 silent=2 abandoned=1 unreachable=0`,
            );
          },
        },),
      ],
    },),

    describe({
      name: 'the wrapped client',
      children: [
        it({
          name: 'COUNTS A CALL THE ROUND\'S GRACE CUT, THAT HAD DELIVERED NO ANSWER AND NO REASONING, as silent',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            await askOnce({
              inner: throwingClient({ failure: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 0, }, },), },),
              tally,
              signal: signalCutByTheRound(),
            },);
            expect(tally.counts()[0]?.silent,).toBe(1,);
            expect(tally.counts()[0]?.asked,).toBe(1,);
            expect(tally.counts()[0]?.abandoned,).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS A CALL THE ROUND\'S GRACE CUT AFTER IT HAD REASONING as abandoned, the slow reasoner that was '
            + 'still working (Qwen, 43,372 reasoning characters, no answer yet)',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            await askOnce({
              inner: throwingClient({ failure: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 43_372, }, },), },),
              tally,
              signal: signalCutByTheRound(),
            },);
            expect(tally.counts()[0]?.silent,).toBe(0,);
            expect(tally.counts()[0]?.abandoned,).toBe(1,);
            expect(tally.counts()[0]?.asked,).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS AN EMPTY CUT THE CALLER OR A DEADLINE ABORTED as abandoned, since only the round\'s own grace '
            + 'window says the seat had its chance: an entry stop must not make every in-flight seat look dead',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Controller the caller aborts with its own reason. */
            const callerStop = new AbortController();
            callerStop.abort(new Error('the entry hit its hard cap',),);
            await askOnce({
              inner: throwingClient({ failure: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 0, }, },), },),
              tally,
              signal: callerStop.signal,
            },);
            expect(tally.counts()[0]?.silent,).toBe(0,);
            expect(tally.counts()[0]?.abandoned,).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS AN EMPTY CUT WITH NO COUNT OF CHARACTERS as silent only when no byte ever arrived, and as '
            + 'abandoned when one did, since the wire length is framing',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            await askOnce({
              inner: throwingClient({ failure: cutStreamError({ firstByte: 'never', },), },),
              tally,
              signal: signalCutByTheRound(),
            },);
            await askOnce({
              inner: throwingClient({ failure: cutStreamError({ firstByte: 'arrived', },), },),
              tally,
              signal: signalCutByTheRound(),
            },);
            expect(tally.counts()[0]?.silent,).toBe(1,);
            expect(tally.counts()[0]?.abandoned,).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS AN ERROR THAT IS NOT A CUT STREAM, aborted by the round, as abandoned',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            await askOnce({
              inner: throwingClient({ failure: new Error('cut by the round',), }),
              tally,
              signal: signalCutByTheRound(),
            },);
            expect(tally.counts()[0]?.silent,).toBe(0,);
            expect(tally.counts()[0]?.abandoned,).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES THE SEAT AFTER FOUR SILENT ASKS and says why in words that cover both ways an ask can fail',
          fn: async () => {
            /** Tally the wrapper counts into. */
            const tally = createSeatTally();
            /** Client over a seat whose every stream is cut empty. */
            const client = seatTallyClient({
              inner: throwingClient({ failure: cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 0, }, },), },),
              tally,
              minAsks: 4,
            },);
            /**
             Asks the seat once with the round's cut on its signal and lets the throw pass.

             @returns Nothing
             */
            async function askDeadStream(): Promise<void> {
              await rejectionOf(async function ask(): Promise<void> {
                await client.chatText({
                  modelId: SEAT_HYPER_VISION,
                  messages: [{ role: 'user', content: 'meow', },],
                  signal: signalCutByTheRound(),
                },);
              },);
            }
            await askDeadStream();
            await askDeadStream();
            await askDeadStream();
            await askDeadStream();
            /** What the fifth ask was refused with. */
            const refusal = await rejectionOf(async function fifth(): Promise<void> {
              await client.chatText({
                modelId: SEAT_HYPER_VISION,
                messages: [{ role: 'user', content: 'meow', },],
                signal: new AbortController().signal,
              },);
            },);
            expect(refusal,).toBeInstanceOf(NoProviderForModelError,);
            expect((refusal as Error).message,).toContain('all 4 asks failed',);
            expect((refusal as Error).message,).toContain('4 ended with nothing delivered',);
          },
        },),
      ],
    },),

    describe({
      name: runGatherRound.name,
      children: [
        it({
          name: 'CUTS ITS STRAGGLERS WITH THE STRAGGLER CUT AS THE ABORT REASON, so the tally can tell the round\'s grace '
            + 'window from the caller leaving',
          fn: async () => {
            /** Reasons the hanging call's signal carried when it was aborted. */
            const reasons: unknown[] = [];
            /** Client whose first seat answers at once and whose second waits for the cut. */
            const client: ModelClient = {
              chatText: async () => {
                throw new Error('chatText unused',);
              },
              chatJson: async (request) => {
                if (request.modelId === SEAT_SYNTHETIC_TEXT_EVERYWHERE) {
                  await untilAborted({ signal: request.signal, },);
                  reasons.push(request.signal.reason,);
                  throw new Error('cut by the round',);
                }
                /** Scripted payload for the answering call. */
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
            await runGatherRound({
              client,
              modelIds: [SEAT_HYPER_VISION, SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
              messages: [{ role: 'user', content: 'meow', },],
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              responseFormat: MEOW_FORMAT,
              validate: isMeowReply,
              stage: 'cat-stage',
              l: capturingLogger({ messages: [], },),
              heardNeeded: 1,
              graceMs: 30,
            },);
            // The round returns without waiting for the call it cut; the call ends at the next turns of the loop.
            await turnOfTheLoop();
            await turnOfTheLoop();
            expect(reasons.length,).toBe(1,);
            expect(reasons[0],).toBeInstanceOf(StragglerCutError,);
          },
        },),
      ],
    },),
  ],
},);
