//region Fake clock for a seat reading
// TEST SUPPORT, NOT PACKAGE SOURCE. Runs a seat reading whose scripted holds are waited out on a fake clock, so no
// case sleeps for a hold and a stalled machine changes nothing. Shared by the cases that script a short hold (40 ms)
// and let the reader wait it out in slices of a few milliseconds.

import type { DisposableSandbox, } from '@monochromatic-dev/module-test/ts';

/**
 Runs a reading while a fake clock passes the end of its holds, and returns what the reading returned.

 THE CLOCK MOVES PAST THE LONGEST WAIT THE READING CAN TAKE, one hold and one poll, and the reading is then aborted: a
 reader that waited for a hold finishes its wait inside that tick and reads again, one that did not has finished before
 it, and a reading that is still waiting afterwards becomes a failure instead of a hang. The fake clock is removed
 before the call returns, so one case can take several readings.

 @param sinon - case's own sandbox, which fakes the timers

 @param holdMs - longest hold the reading's scripted holds state

 @param pollMs - how often the reading's wait checks for abort

 @param reading - starts the reading, handing it the abort signal it must honour

 @returns Whatever the reading returned

 @throws What the reading throws, among them the abort reason of a reading that was still waiting after the tick

 @example
 ```ts
 const reads = await readOnFakeClock({
   sinon: ctx.sinon,
   holdMs: 40,
   pollMs: 5,
   reading: async ({ signal, }) => await readJudgeSeats({ client, phase, signal, l, pollMs: 5, },),
 },);
 ```
 */
export async function readOnFakeClock<ResultT,>(
  {
    sinon,
    holdMs,
    pollMs,
    reading,
  }: {
    readonly sinon: DisposableSandbox;
    readonly holdMs: number;
    readonly pollMs: number;
    readonly reading: (inputs: { readonly signal: AbortSignal; }) => Promise<ResultT>;
  },
): Promise<ResultT> {
  /**
   The fake clock the reading's wait sleeps on.
   */
  const clock = sinon.useFakeTimers({
    toFake: [
      'setTimeout',
      'clearTimeout',
    ],
  },);
  /**
   Puts the real timers back when the call ends, however it ends.
   */
  using _installed = {
    [Symbol.dispose]: function removeClock(): void {
      clock.restore();
    },
  };
  /**
   Abort the reading carries, raised once the clock has passed the hold.
   */
  const abort = new AbortController();
  /**
   The reading, started on the fake clock.
   */
  const running = reading({ signal: abort.signal, },);
  await clock.tickAsync(holdMs + pollMs,);
  abort.abort();
  await clock.tickAsync(pollMs,);
  return await running;
}

//endregion Fake clock for a seat reading
