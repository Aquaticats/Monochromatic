import { syncBuiltinESMExports, } from 'node:module';
import timersPromises, { setTimeout as pause, } from 'node:timers/promises';

import type { DisposableSandbox, } from '@monochromatic-dev/module-test/ts';
import { HANG_STOP_MS, } from './hang-stop.test-fixture.ts';

//region Fake clock walk
// RUNS A WALK THAT LASTS MINUTES ON A FAKE CLOCK, for the cases that ask whether a call outlives a deadline without
// waiting for it. `setTimeout` and `clearTimeout` are faked on the calling case's own sandbox, and the signal factory
// `AbortSignal.timeout` is replaced by one built on the fake `setTimeout`, so a signal made from a timer, from
// `AbortSignal.timeout` or from `AbortSignal.any` over either aborts at its figure on the fake clock and the cut
// shows. The clock stands still until the walk has made its first call, since the walk reads its inputs over real I/O
// first and ticking through that would spend the scripted minutes before anything had been asked. The clock is then
// ticked by the steps the case hands, and the case reads how many exchanges the transport had cut after each.
//
// `node:timers/promises` IS ON THE CLOCK TOO. A cut written with `import {
// setTimeout } from 'node:timers/promises'` was on the real clock: the sinon clock reaches that built-in only where
// its own module can `require` it, which the bundled test harness cannot (measured: the walk of a two-minute wait of
// that form did not end on a tick past it), and a named import is bound once besides. The walk stubs the built-in's
// `setTimeout` with a wait on the fake global one, and `syncBuiltinESMExports` copies the module's current properties
// into its named exports, once after the stub and once when the walk ends, so a cut on either form is on the fake
// clock for the walk and on the real one again after it. FAKED RATHER THAN REFUSED BY A SOURCE SCAN: a scan has to
// list every spelling of the import and can be written around, while this puts the one wait every spelling reaches on
// the clock the case owns. `setImmediate`, `setInterval` and `scheduler.wait` of that module stay real; no cut in the
// family is written with them. `fake-clock-walk.unit.test.ts` walks both forms.
//
// THE STUB IS PROCESS-WIDE, so the retry ladder's sleeps (`provider/src/transient-retry.ts`) and the request pace
// (`provider/src/request-pace.ts`), which import that built-in, are on the fake clock during a walk too, and so
// would the harness's own fixtures be if they ran in a walking process (they do not: only the two bench probe
// suites walk). A sleep the release of the transport provokes is therefore a wait on a clock nobody ticks again,
// and the walk stops waiting for the walk's end at the hang stop after the release as it does before the first call.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The delayed transport this drives is `delayed-transport.test-fixture.ts` of the
// provider package; this file knows it only through the promise and the release it is handed.

/**
 The real wait, taken when this module is evaluated, before any walk has put the fake one in the named export. The hang
 stop races it, so it must stay real while a walk ticks.
 */
const realWait = pause;

/**
 Waits on the fake global clock the way `node:timers/promises` `setTimeout` waits on the real one.

 @param ms - fake milliseconds to wait

 @param value - what the wait settles with

 @param signal - abort signal that ends the wait early, rejecting it with an `AbortError`

 @returns Settles with the value once the fake clock has advanced by the figure

 @example
 ```ts
 await waitOnFakeClock({ ms: 120_000, value: undefined, signal, },);
 ```
 */
function waitOnFakeClock(
  {
    ms,
    value,
    signal,
  }: {
    readonly ms: number;
    readonly value: unknown;
    readonly signal: AbortSignal;
  },
): Promise<unknown> {
  /**
   Promise the wait settles, and the two ends that settle it.
   */
  const {
    promise,
    resolve,
    reject,
  } = Promise.withResolvers<unknown>();
  if (signal.aborted) {
    reject(new DOMException(
      'The operation was aborted',
      'AbortError',
    ),);
    return promise;
  }

  /**
   Ends the wait early by rejecting it and dropping the fake timer.
   */
  function abort(): void {
    clearTimeout(timer,);
    reject(new DOMException(
      'The operation was aborted',
      'AbortError',
    ),);
  }

  /**
   Fake timer that settles the wait at its figure.
   */
  const timer = setTimeout(
    function elapsed(): void {
      signal.removeEventListener(
        'abort',
        abort,
      );
      resolve(value,);
    },
    ms,
  );
  signal.addEventListener(
    'abort',
    abort,
    { once: true, },
  );
  return promise;
}

/**
 What the walk did on the fake clock.
 */
export type FakeClockWalk = {
  /**
   Whether the walk had ended on its own when the ticking stopped, so its outcome came from the clock and not from the
   release.
   */
  readonly endedByItself: boolean;

  /**
   How many exchanges the transport had cut after each tick, in the order of the ticks.
   */
  readonly cutsAfterEachTick: readonly number[];
};

/**
 Runs a walk on a fake clock, ticks it by the steps handed, then releases the transport so the walk always ends.

 @param sinon - case's own sandbox, which restores the clock and the signal factory when the case ends

 @param walkOnce - starts the walk and settles when it ends

 @param firstCall - settles when the walk makes its first call

 @param release - ends every exchange still waiting and every later one

 @param ticks - fake milliseconds to advance, one step after another

 @param cutCount - reads how many exchanges the transport has cut so far, called after each tick

 @param waitForEnd - whether to wait, up to the hang stop, for the walk to end once the ticking is done; a case whose
 walk is meant to outlast the ticking passes false and reads `endedByItself` as false

 @param hangStopMs - real milliseconds every wait for the walk gives up after, the one after the release included;
 `HANG_STOP_MS` unless a case of the rig itself needs the stop to be reached

 @returns Whether the walk ended by itself, and the cuts counted after each tick

 @throws Error when the walk makes no call in the real time the case allows

 @throws Error when the walk has not ended in the real time the case allows after the transport is released

 @throws Whatever the walk threw, once the transport is released

 @example
 ```ts
 const { endedByItself, } = await walkOnFakeClock({
   sinon,
   walkOnce,
   firstCall,
   release,
   ticks: [520_000,],
   cutCount,
   waitForEnd: true,
 },);
 ```
 */
export async function walkOnFakeClock(
  {
    sinon,
    walkOnce,
    firstCall,
    release,
    ticks,
    cutCount,
    waitForEnd,
    hangStopMs = HANG_STOP_MS,
  }: {
    readonly sinon: DisposableSandbox;
    readonly walkOnce: () => Promise<void>;
    readonly firstCall: Promise<void>;
    readonly release: () => void;
    readonly ticks: readonly number[];
    readonly cutCount: () => number;
    readonly waitForEnd: boolean;
    readonly hangStopMs?: number;
  },
): Promise<FakeClockWalk> {
  /**
   The fake clock.
   */
  const clock = sinon.useFakeTimers({ toFake: [
    'setTimeout',
    'clearTimeout',
  ], },);

  /**
   The built-in's `setTimeout`, replaced by a wait on the fake clock.
   */
  const fakedWait = sinon.stub(
    timersPromises,
    'setTimeout',
  )
    .callsFake(function sleepOnFakeClock(
      ms,
      value,
      options,
    ) {
      // The built-in's own defaults: one millisecond, and no signal to end the wait early.
      return waitOnFakeClock({
        ms: ms ?? 1,
        value,
        signal: options?.signal ?? new AbortController().signal,
      },);
    },);

  // The module's property is replaced; its named exports follow only when they are told to.
  syncBuiltinESMExports();

  /**
   Puts the real wait back in the named export when the walk ends, however it ends, since the sandbox restores the
   module's property only when the case ends, after which nothing would copy it into the export.
   */
  using _restored = {
    [Symbol.dispose](): void {
      fakedWait.restore();
      syncBuiltinESMExports();
    },
  };
  sinon.stub(
    AbortSignal,
    'timeout',
  )
    .callsFake(function timeoutOnFakeClock(ms,): AbortSignal {
    /**
     Controller the fake timer aborts.
     */
      const controller = new AbortController();
      setTimeout(
        function expire(): void {
          controller.abort(new DOMException(
            'The operation was aborted due to timeout',
            'TimeoutError',
          ),);
        },
        ms,
      );
      return controller.signal;
    },);

  /**
   Whether the walk has ended, either way, and what it threw, in a list of none or one.
   */
  const ended = {
    done: false,
    thrown: [] as unknown[],
  };

  /**
   Starts the walk and notes how it ends, so a failure of the walk is not left unhandled while the clock ticks.
   */
  async function tracking(): Promise<void> {
    try {
      await walkOnce();
    }
    catch (error) {
      ended.thrown
        .push(error,);
    }
    ended.done = true;
  }

  /**
   The walk's end, whichever way.
   */
  const settled = tracking();

  /**
   Cuts counted after each tick so far.
   */
  const cutsAfterEachTick: number[] = [];

  /**
   Waits for the walk's first call, ticks the clock by the steps, and reports whether the walk ended before the
   release, which is what tells a cut or a reply from the case giving up. The transport is released when this scope
   ends, however it ends.

   @returns Whether the walk ended by itself
   */
  async function ticked(): Promise<boolean> {
    /**
     Releases the transport when the scope ends.
     */
    using _released = { [Symbol.dispose]: release, };
    await Promise.race([
      firstCall,
      settled,
      realWait(hangStopMs,)
        .then(function giveUp(): never {
          throw new Error('the walk made no call in the real time the case allows',);
        },),
    ],);
    await ticks.reduce<Promise<void>>(
      async function afterTick(
        previous,
        ms,
      ): Promise<void> {
        await previous;
        await clock.tickAsync(ms,);
        cutsAfterEachTick.push(cutCount(),);
      },
      Promise.resolve(),
    );
    if (waitForEnd)
      await Promise.race([
        settled,
        realWait(hangStopMs,),
      ],);
    return ended.done;
  }

  /**
   Whether the walk ended before the release.
   */
  const endedByItself = await ticked();

  // THE TRANSPORT IS RELEASED NOW, and a walk that sleeps again because of it sleeps on the fake clock, which nobody
  // ticks any more: waiting for it without a stop would hang the case where, before the clock was faked, it
  // ended in real time.
  await Promise.race([
    settled,
    realWait(hangStopMs,)
      .then(function giveUpAfterRelease(): never {
        throw new Error('the walk did not end after the release in the real time the case allows',);
      },),
  ],);
  if (ended.thrown
    .length
    > 0)
    throw ended.thrown[0];
  return {
    endedByItself,
    cutsAfterEachTick,
  };
}

//endregion Fake clock walk
