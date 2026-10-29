/**
 Tests that the seat reader does not wait for a provider refused for payment, whose stated hold ends and whose meter
 reads wet.

 THE SHAPE: a refusal for payment (HTTP 402) keeps a provider dry until its meter moves, however wet the meter reads,
 and the same refusal may state a wait of its own, which is a hold. When that hold ends the provider is still dry, so a
 reader that counts it as returning waits for nothing. Here the wide bench stands one seat short of its quorum with
 NeuralWatt and Bedrock wet, and Hyper, whose return would reach the quorum, is refused for payment while its meter
 reads wet. The budgets are the real ones over stub meters, as the router's own are, so the states and the holds reach
 the reader the way a run's do. The hold is a minute and a half on a fake clock that both cases tick past, so no case
 sleeps and a stalled machine changes nothing: the hold is read running before the reading starts, and the clock
 passes its end while the reading runs, so a reader that waited for it would read twice in either case.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
  type DisposableSandbox,
} from '@monochromatic-dev/test-harness-core/ts';

import { createProviderBudgets, HOLD_POLL_MS, } from '@monochromatic-dev/translation-repair-provider';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { readJudgeSeats, } from '../../dist/final/node/index.mjs';
import { shortWarningOf, } from './run-seats-short-warning.test-fixture.ts';

/**
 The wait the refusals here state, longer than the rate-limit backoff of a minute so the control case's hold is the
 stated wait, and read on the fake clock, which only the cases move.
 */
const STATED_HOLD_MS = 90_000;

/**
 Budgets over stub meters with NeuralWatt, Bedrock and Hyper wet and every other provider unconfigured, so dry.

 @returns Real budgets, reading on whichever clock is global when they are asked, which the cases fake

 @example
 ```ts
 const budgets = budgetsWithHyperWet();
 ```
 */
function budgetsWithHyperWet(): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    neuralwatt: {
      energy: async () => ({
        dayStart: '2026-10-23T19:15:08.000Z',
        dailyWh: 550,
        spentWh: 0,
        reckonedWh: 0,
        reservedWh: 0,
        remainingWh: 550,
        calls: 0,
      }),
    },
    bedrock: {
      credits: async () => ({
        creditUsd: 100,
        spentUsd: 10,
        reckonedUsd: 0,
        remainingUsd: 90,
        calls: 12,
      }),
    },
    hyper: { credits: async () => ({ balance: 243, }), },
  },);
}

/**
 Refuses Hyper once, reads the lane contest's seats while a fake clock passes the end of the hold, and counts the
 dryness reads the reading took.

 THE HOLD IS READ RUNNING FIRST, then the reading starts and the clock moves past the hold's end and one poll: a reader
 that waits for the hold finishes its wait inside that tick and reads twice, one that does not has finished before it.
 An abort at the end turns a reading that is still waiting into a failure, never a hang.

 @param paymentRequired - whether the refusal said the balance cannot pay for the call

 @param lines - receives what the reading logs

 @param sinon - case's own sandbox, which restores the clock when the case ends

 @returns Dryness reads taken: 1 without a wait, 2 with one

 @example
 ```ts
 const reads = await readsAfterRefusal({ paymentRequired: true, lines: [], sinon: ctx.sinon, },);
 ```
 */
async function readsAfterRefusal(
  {
    paymentRequired,
    lines,
    sinon,
  }: {
    readonly paymentRequired: boolean;
    readonly lines: string[];
    readonly sinon: DisposableSandbox;
  },
): Promise<number> {
  /**
   The fake clock the budgets count the hold down on and the wait sleeps on.
   */
  const clock = sinon.useFakeTimers({
    toFake: ['setTimeout', 'clearTimeout', 'performance',],
    shouldClearNativeTimers: true,
  },);
  /**
   Abort every read here carries, raised once the clock has passed the hold.
   */
  const abort = new AbortController();
  /**
   The real budgets whose Hyper meter reads wet.
   */
  const budgets = budgetsWithHyperWet();
  await budgets.markRefused({
    provider: 'hyper',
    signal: abort.signal,
    statedWaitMs: STATED_HOLD_MS,
    paymentRequired,
  },);
  // THE HOLD IS OBSERVED RUNNING, not assumed: the clock has not moved since the refusal, so it stands at the stated
  // wait.
  expect(budgets.holds().hyper,).toBe(STATED_HOLD_MS,);
  /**
   Dryness reads taken so far.
   */
  const counted = { reads: 0, };
  /**
   The reading, started on the fake clock.
   */
  const reading = readJudgeSeats({
    client: {
      providerDryness: async function providerDryness() {
        counted.reads += 1;
        return await budgets.read({ signal: abort.signal, },);
      },
      providerMeterStates: async function providerMeterStates() {
        return await budgets.readStates({ signal: abort.signal, },);
      },
      providerHolds: budgets.holds,
    },
    phase: 'lane contest',
    signal: abort.signal,
    l: levelCapturingLogger({ lines, },),
  },);
  await clock.tickAsync(STATED_HOLD_MS + HOLD_POLL_MS,);
  abort.abort();
  await clock.tickAsync(HOLD_POLL_MS,);
  await reading;
  return counted.reads;
}

// THE LOGGER STARTS ON ITS FIRST LINE, and its start sleeps on a timer: begun under the fake clock it writes an
// internal error to stderr. One reading on the real clock starts it before any case fakes the timers.
await budgetsWithHyperWet().read({ signal: new AbortController().signal, },);

await describe({
  name: 'the seat reader and a provider refused for payment',
  concurrency: 1,
  children: [
    it({
      name: 'WAITS OUT THE STATED HOLD OF A PROVIDER REFUSED FOR A REASON OTHER THAN PAYMENT (the control): Hyper back '
        + 'makes the wide bench 5 of 7 against a quorum of 4, and its meter reads wet, so the wait buys the quorum',
      fn: async (ctx) => {
        expect(await readsAfterRefusal({ paymentRequired: false, lines: [], sinon: ctx.sinon, },),).toBe(2,);
      },
    },),

    it({
      name: 'DOES NOT WAIT FOR A PROVIDER REFUSED FOR PAYMENT whose stated hold ends and whose meter reads wet: it '
        + 'stays dry until the meter moves, so the hold ending brings nothing back and the reader seats the bench '
        + 'short at once',
      fn: async (ctx) => {
        /**
         What the reading logged.
         */
        const lines: string[] = [];
        expect(await readsAfterRefusal({ paymentRequired: true, lines, sinon: ctx.sinon, },),).toBe(1,);
        // THE WARNING NAMES THE REFUSAL AS A CAUSE OF DRYNESS: Hyper's meter reads wet, so a clause saying only that it
        // reads dry by its meter would contradict the reading it follows.
        expect(shortWarningOf({ lines, phase: 'lane contest', },),).toContain(
          'hyper reads dry by its meter or by a payment refusal that stands until the meter moves, which a hold ending '
            + 'does not change',
        );
        expect(lines.some(function waitLine(line,): boolean {
          return line.includes('waiting ',);
        },),).toBe(false,);
      },
    },),
  ],
},);
