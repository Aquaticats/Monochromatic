//region Fake clock for a hold wait
// TEST SUPPORT, NOT PACKAGE SOURCE. Puts a fake clock under the wait the budget layer takes on a refusal hold
// (`waitOutHold`, `budget-hold-wait.ts`), so no case sleeps for a hold and a stalled machine changes nothing. The wait
// sleeps on the global `setTimeout`, in slices of its poll interval, and the clock here fakes that and `clearTimeout`
// only, with the native timers cleared. Two entries: `fakeClockStarted` hands the clock to a case that ticks it itself,
// and `waitedOutOnFakeClock` runs a reading to its end on the clock and reports how long the reading waited.
//
// THE RUN PACKAGE KEEPS ITS OWN (`run-seats-fake-clock.test-fixture.ts`), which this package cannot import; the two do
// the same job for different readings and differ in what they hand a reading and what they report.

import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';

/**
 Fake clock that the production wait sleeps on, for a case that ticks it itself.

 The clock is installed on the calling case's own sandbox with the timers the wait arms, and removed when the returned
 handle is disposed, so the case sleeps no real time and a stalled machine changes nothing. A native timer made before
 the install and cleared under it, such as the module logger's start check, is cleared by the real `clearTimeout`: the
 fake one would refuse it, write a `FakeTimers` message to stderr and leave the timer holding the process for its full
 length.

 @param sinon - case's own sandbox, which restores the clock when the case ends

 @returns The clock, starting at one second so no reading sits at instant zero, and the disposal that removes it

 @example
 ```ts
 using fake = fakeClockStarted({ sinon: ctx.sinon, },);
 ```
 */
export function fakeClockStarted(
  { sinon, }: { readonly sinon: DisposableSandbox; },
): { readonly clock: ReturnType<DisposableSandbox['useFakeTimers']>; } & Disposable {
  /**
   The fake clock the wait sleeps on.
   */
  const clock = sinon.useFakeTimers({
    now: 1_000,
    toFake: [
      'setTimeout',
      'clearTimeout',
    ],
    shouldClearNativeTimers: true,
  },);
  return {
    clock,
    [Symbol.dispose]: function removeClock(): void {
      clock.restore();
    },
  };
}

/**
 Runs a reading while a fake clock passes the end of its holds, and returns what the reading returned with the fake
 time that had passed when it returned.

 THE CLOCK MOVES PAST THE LONGEST WAIT THE READING CAN TAKE, the holds it waits out and one poll, and the reading is
 then aborted: a reading that waited for its holds finishes inside that tick and reads again, one that did not has
 finished before it, and a reading that is still waiting afterwards becomes a failure with the abort reason instead of a
 hang. A reading that rejects while the clock ticks is reported by the final await and not as an unhandled rejection.
 The fake clock is removed before the call returns, so one case can take several readings.

 @param sinon - case's own sandbox, which fakes the timers

 @param holdMs - total of the waits the reading takes: the holds it is told of, one after another

 @param pollMs - how often the reading's wait checks for abort

 @param reading - starts the reading, handing it the abort signal it must honour

 @returns Whatever the reading returned, and the fake milliseconds from the start of the call to the instant the
 reading returned, which are exactly the milliseconds the reading slept

 @throws What the reading throws, among them the abort reason of a reading that was still waiting after the tick

 @example
 ```ts
 const { value, elapsedMs, } = await waitedOutOnFakeClock({
   sinon: ctx.sinon,
   holdMs: 5,
   pollMs: 1,
   reading: async ({ signal, }) => await readBudgetsPastHolds({ budgets, modelId, signal, refused, pollMs: 1, },),
 },);
 ```
 */
export async function waitedOutOnFakeClock<ResultT,>(
  {
    sinon,
    holdMs,
    pollMs,
    reading,
  }: {
    readonly sinon: DisposableSandbox;
    readonly holdMs: number;
    readonly pollMs: number;
    readonly reading: (inputs: { readonly signal: AbortSignal; },) => Promise<ResultT>;
  },
): Promise<{
  readonly value: ResultT;
  readonly elapsedMs: number;
}> {
  /**
   The fake clock the reading's wait sleeps on, removed when the call ends however it ends.
   */
  using fake = fakeClockStarted({ sinon, },);
  /**
   The clock the call ticks and reads.
   */
  const { clock, } = fake;
  /**
   Fake instant the call began at.
   */
  const began = clock.now;
  /**
   Abort the reading carries, raised once the clock has passed the holds.
   */
  const abort = new AbortController();
  /**
   Runs the reading and notes the fake instant it returned at, which is before the tick that carries it moves on.

   @returns The reading's result with the fake time it took
   */
  async function timedReading(): Promise<{
    readonly value: ResultT;
    readonly elapsedMs: number;
  }> {
    /**
     What the reading returned.
     */
    const value = await reading({ signal: abort.signal, },);
    return {
      value,
      elapsedMs: clock.now - began,
    };
  }
  /**
   The reading, started on the fake clock.
   */
  const running = timedReading();
  // A reading that rejects while the clock ticks is reported by the final await, not as an unhandled rejection.
  void Promise.allSettled([running,],);
  await clock.tickAsync(holdMs + pollMs,);
  abort.abort();
  await clock.tickAsync(pollMs,);
  return await running;
}

//endregion Fake clock for a hold wait
