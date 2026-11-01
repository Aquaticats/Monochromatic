/**
 Tests for what a stop signal shares with the entry in flight (B1247): the signal the entries run under, the flight an
 entry registers, and the promise that settles when the entry has shipped or has left.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createPassStop,
  SECOND_SIGNAL_MIN_GAP_MS,
  STOP_SHIP_WINDOW_MS,
} from '../../dist/final/node/index.mjs';

/**
 Whether a promise has settled by the time the microtasks queued so far have run.

 @param promise - promise to look at

 @returns True when it had settled

 @example
 ```ts
 const settled = await settledNow({ promise, },);
 ```
 */
async function settledNow({ promise, }: { readonly promise: Promise<void>; },): Promise<boolean> {
  /**
   What the look found.
   */
  const seen: { settled: boolean; } = { settled: false, };
  void noteSettled({
    promise,
    seen,
  },);
  await Promise.resolve();
  await Promise.resolve();
  return seen.settled;
}

/**
 Notes in a holder that a promise settled.

 @param promise - promise to wait on

 @param seen - holder the note is made in

 @example
 ```ts
 void noteSettled({ promise, seen, },);
 ```
 */
async function noteSettled(
  {
    promise,
    seen,
  }: {
    readonly promise: Promise<void>;
    readonly seen: { settled: boolean; };
  },
): Promise<void> {
  await promise;
  seen.settled = true;
}

await describe({
  name: createPassStop.name,
  concurrency: 1,
  children: [
    it({
      name: 'STARTS UNREQUESTED WITH NO ENTRY IN FLIGHT, and a wait for the entry in flight settles at once',
      fn: async () => {
        const stop = createPassStop();
        expect([stop.signal.aborted, stop.inFlight(),],).toEqual([false, false,],);
        expect(await settledNow({ promise: stop.inFlightShipped(), },),).toBe(true,);
      },
    },),

    it({
      name: 'ABORTS ITS SIGNAL WHEN A STOP IS REQUESTED, with an AbortError for a reason, so every stage under it ends '
        + 'under the class AbortError',
      fn: async () => {
        const stop = createPassStop();
        stop.request();
        expect(stop.signal.aborted,).toBe(true,);
        expect(Error.isError(stop.signal.reason,) ? stop.signal.reason.name : 'none',).toBe('AbortError',);
      },
    },),

    it({
      name: 'HOLDS THE WAIT UNTIL THE ENTRY SAYS IT SHIPPED, and lets it go then',
      fn: async () => {
        const stop = createPassStop();
        using flight = stop.begin();
        expect(stop.inFlight(),).toBe(true,);
        expect(await settledNow({ promise: stop.inFlightShipped(), },),).toBe(false,);
        flight.shipped();
        expect(await settledNow({ promise: stop.inFlightShipped(), },),).toBe(true,);
      },
    },),

    it({
      name: 'LETS THE WAIT GO WHEN THE ENTRY LEAVES WITHOUT SAYING IT SHIPPED, and holds no entry in flight afterwards',
      fn: async () => {
        const stop = createPassStop();
        {
          using _flight = stop.begin();
          expect(stop.inFlight(),).toBe(true,);
        }
        expect(stop.inFlight(),).toBe(false,);
        expect(await settledNow({ promise: stop.inFlightShipped(), },),).toBe(true,);
      },
    },),

    it({
      name: 'SETTLES ITS EXIT PROMISE WHEN THE HANDLER MARKS THE EXIT, and not before',
      fn: async () => {
        const stop = createPassStop();
        expect(await settledNow({ promise: stop.exited, },),).toBe(false,);
        stop.markExited();
        expect(await settledNow({ promise: stop.exited, },),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS THE WINDOW UNDER SYSTEMD\'S DEFAULT STOP TIMEOUT of 90 seconds with room for the flush that follows',
      fn: async () => {
        expect(STOP_SHIP_WINDOW_MS,).toBeLessThan(60_000,);
        expect(STOP_SHIP_WINDOW_MS,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'KEEPS THE INTERVAL OF A DUPLICATE DELIVERY (B1295) above nothing and inside the window, so a second signal '
        + 'can still end a wait that the window has not',
      fn: async () => {
        expect(SECOND_SIGNAL_MIN_GAP_MS,).toBeGreaterThan(0,);
        expect(SECOND_SIGNAL_MIN_GAP_MS,).toBeLessThan(STOP_SHIP_WINDOW_MS,);
      },
    },),

    it({
      name: 'HOLDS AN ENTRY IN FLIGHT FROM ITS RESERVATION (B1298), before the entry begins, so a signal between the '
        + 'queue\'s decision to start it and its start finds it, and lets the wait go when the reservation is let go',
      fn: async () => {
        const stop = createPassStop();
        {
          using _reservation = stop.reserve();
          expect(stop.inFlight(),).toBe(true,);
          expect(await settledNow({ promise: stop.inFlightShipped(), },),).toBe(false,);
        }
        expect(stop.inFlight(),).toBe(false,);
        expect(await settledNow({ promise: stop.inFlightShipped(), },),).toBe(true,);
      },
    },),

    it({
      name: 'LETS THE ENTRY THAT BEGINS TAKE OVER ITS RESERVATION (B1298): a wait started at the reservation settles when '
        + 'the entry says it shipped, and releasing the reservation afterwards removes nothing',
      fn: async () => {
        const stop = createPassStop();
        using reservation = stop.reserve();
        /**
         The wait a signal handler would have started at the reservation.
         */
        const waiting = stop.inFlightShipped();
        {
          using flight = stop.begin();
          expect(await settledNow({ promise: waiting, },),).toBe(false,);
          flight.shipped();
          expect(await settledNow({ promise: waiting, },),).toBe(true,);
        }
        expect(stop.inFlight(),).toBe(false,);
        reservation[Symbol.dispose]();
        expect(stop.inFlight(),).toBe(false,);
      },
    },),
  ],
},);
