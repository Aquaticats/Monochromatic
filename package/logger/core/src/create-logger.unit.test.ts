import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createLogger,
  DEFAULT_FLUSH_DEADLINE_MS,
  DEFAULT_VERIFY_TIMEOUT_MS,
  STARTUP_BUFFER_CAP,
  type LogRecord,
  type Sink,
  type SinkFlush,
  type Verify,
} from '@monochromatic-dev/logger-core';

/**
 Milliseconds a slow write parks before recording, long enough that the
 record is provably still pending when a synchronous assertion runs but the
 draining `flush()` must wait for it.
 */
const SLOW_WRITE_MS = 25;

/**
 Flush deadline the deadline tests inject: short enough to keep the suite
 fast, long enough that timer granularity cannot fire it early.
 */
const SHORT_DEADLINE_MS = 60;

/**
 Timer slack subtracted from the deadline when asserting a flush waited it
 out, covering setTimeout clamping and scheduler jitter.
 */
const DEADLINE_TOLERANCE_MS = 15;

/**
 Upper bound on a flush that must not wait out the deadline again; well
 under `SHORT_DEADLINE_MS` so a regression that re-waits is caught.
 */
const FAST_FLUSH_MS = 40;

/**
 Harness timeout for the deadline tests: a regression that hangs forever
 fails here instead of stalling the suite.
 */
const DEADLINE_TEST_TIMEOUT_MS = 2_000;

/**
 Promise that never settles, standing in for a wedged sink operation.
 
 @returns Pending promise whose resolver is unreachable.
 */
function neverSettles(): Promise<never> {
  return Promise.withResolvers<never>().promise;
}

/**
 Times one `flush()` call.
 
 @param flush - Flush function to time.
 
 @returns Elapsed milliseconds.
 */
async function timeFlush({ flush, }: { readonly flush: () => Promise<void>; },): Promise<number> {
  /**
   Start timestamp.
   */
  const start = performance.now();
  await flush();
  return performance.now() - start;
}

/**
 Structural view of a sinon stub: only the recorded calls matter here, and
 naming the shape keeps the test free of a direct sinon type import.
 */
type RecordedCalls = {
  readonly getCalls: () => readonly { readonly args: readonly unknown[]; }[];
};

/**
 Collects the console.warn messages containing `needle`. Sibling tests in
 this file run concurrently and emit their own internal-error reports
 through the same console, so a raw call count would be noise.
 
 @param warn - Stubbed console.warn.
 
 @param needle - Substring identifying the breadcrumb family.
 
 @returns Matching messages, in call order.
 */
function breadcrumbMessages(
  {
    warn,
    needle,
  }: {
    readonly warn: RecordedCalls;
    readonly needle: string;
  },
): string[] {
  return warn.getCalls()
    .map(function toMessage(call,) {
      return String(call.args[0],);
    },)
    .filter(function matchesNeedle(message,) {
      return message.includes(needle,);
    },);
}

/**
 Collects the flush-deadline breadcrumbs, see {@link breadcrumbMessages}.
 
 @param warn - Stubbed console.warn.
 
 @returns Flush-deadline breadcrumb messages observed, in call order.
 */
function deadlineBreadcrumbMessages({ warn, }: { readonly warn: RecordedCalls; },): string[] {
  return breadcrumbMessages({
    needle: 'flush deadline',
    warn,
  },);
}

/**
 Counts the sink-verification breadcrumbs (a verify that rejected, threw, or
 ran past the verify time limit).
 
 @param warn - Stubbed console.warn.
 
 @returns Number of verification breadcrumbs observed.
 */
function verifyBreadcrumbs({ warn, }: { readonly warn: RecordedCalls; },): number {
  return breadcrumbMessages({
    needle: 'sink verification failed',
    warn,
  },).length;
}

/**
 Builds a verifier that answers `true` after a delay.
 
 @param delayMs - Milliseconds before the verifier resolves.
 
 @returns Verify function resolving `true` after the delay.
 */
function verifyTrueAfter({ delayMs, }: { readonly delayMs: number; },): Verify {
  return async function verifyLater(): Promise<boolean> {
    await wait(delayMs,);
    return true;
  };
}

/**
 Counts the flush-deadline breadcrumbs, see {@link deadlineBreadcrumbMessages}.
 
 @param warn - Stubbed console.warn.
 
 @returns Number of flush-deadline breadcrumbs observed.
 */
function deadlineBreadcrumbs({ warn, }: { readonly warn: RecordedCalls; },): number {
  return deadlineBreadcrumbMessages({ warn, },).length;
}

/**
 Builds a verified sink whose every write never settles.
 
 @returns Sink standing in for a wedged backend.
 */
function wedgedWriteSink(): Sink {
  return {
    verify: function verifyAvailable(): Promise<boolean> {
      return Promise.resolve(true,);
    },
    write: function writeForever(): Promise<void> {
      return neverSettles();
    },
  };
}

/**
 Recording sink plus the array it appends every written record to, so a test
 can assert exactly which records crossed the seam.
 */
type RecordingSink = {
  readonly records: LogRecord[];
  readonly sink: Sink;
};

/**
 Builds a fake sink that records every record it receives. The seam under
 test is `Sink`, so the whole orchestration (verify, replay, fan-out, flush)
 is exercised through one self-contained adapter with no globals to reset.
 
 @param verify - Backend availability check; defaults to synchronously available.
 
 @param flush - Optional flush hook the logger should drain.
 
 @param writeDelayMs - Milliseconds each write parks before recording, to
 keep a record pending across a `flush()`.
 
 @returns Sink adapter paired with its recorded-record array.
 */
function recordingSink(
  {
    verify = function verifyAvailable(): Promise<boolean> {
      return Promise.resolve(true,);
    },
    flush,
    writeDelayMs = 0,
  }: {
    readonly flush?: SinkFlush;
    readonly verify?: Verify;
    readonly writeDelayMs?: number;
  } = {},
): RecordingSink {
  /**
   Records this sink has received, in arrival order.
   */
  const records: LogRecord[] = [];

  /**
   Records every received record after the optional delay.
   
   @param record - Record handed to the sink.
   */
  async function write(record: LogRecord,): Promise<void> {
    if (writeDelayMs > 0)
      await wait(writeDelayMs,);
    records.push(record,);
  }

  // `flush` is spread in only when present: under `exactOptionalPropertyTypes`
  // an optional property cannot be assigned an explicit `undefined`.
  const sink: Sink = {
    ...((flush === undefined) ? {} : { flush, }),
    verify,
    write,
  };
  return {
    records,
    sink,
  };
}

/**
 Maps recorded records down to their messages for concise assertions.
 
 @param recording - Recording sink whose messages to read.
 
 @returns Messages in arrival order.
 */
function messages({ recording, }: { readonly recording: RecordingSink; },): string[] {
  return recording.records
    .map(function toMessage(record,) {
      return record.message;
    },);
}

/**
 Recording sink whose first write parks on a promise the test settles, so a
 `flush()` can abandon that write at the deadline and the test can then
 settle it late, after the logger has stopped tracking it.
 */
type ParkedWriteSink = RecordingSink & {
  readonly settleParked: PromiseWithResolvers<void>;
};

/**
 Builds a verified sink whose first write waits on `settleParked` and whose
 later writes record immediately.
 
 @returns Sink, its record array, and the resolvers for the parked write.
 */
function parkedFirstWriteSink(): ParkedWriteSink {
  /**
   Records this sink has received, in arrival order.
   */
  const records: LogRecord[] = [];
  /**
   Resolvers the test uses to settle the parked write.
   */
  const settleParked = Promise.withResolvers<void>();
  /**
   Whether the parked write has been handed out; only the first write parks.
   */
  const handedOut = { parked: false, };

  /**
   Parks the first write on `settleParked`, records every later one.
   
   @param record - Record handed to the sink.
   */
  async function write(record: LogRecord,): Promise<void> {
    if (!handedOut.parked) {
      handedOut.parked = true;
      await settleParked.promise;
    }
    records.push(record,);
  }

  return {
    records,
    settleParked,
    sink: {
      verify: function verifyAvailable(): Promise<boolean> {
        return Promise.resolve(true,);
      },
      write,
    },
  };
}

/**
 Unhandled-rejection reasons observed while the capture is alive, plus the
 disposer that detaches the listener.
 */
type UnhandledCapture = {
  readonly reasons: readonly unknown[];
  readonly [Symbol.dispose]: () => void;
};

/**
 Listens for `unhandledRejection` on the process until disposed. A listener
 also stops Node from treating the rejection as fatal, so the assertion on
 `reasons` is what enforces the property.
 
 @returns Capture whose `reasons` grows with every unhandled rejection.
 */
function captureUnhandledRejections(): UnhandledCapture {
  /**
   Reasons observed so far.
   */
  const reasons: unknown[] = [];

  /**
   Listener appended to the process.
   
   @param reason - Rejection reason Node reports.
   */
  function record(reason: unknown,): void {
    reasons.push(reason,);
  }

  process.on('unhandledRejection', record,);
  return {
    reasons,
    [Symbol.dispose]: function detach(): void {
      process.off('unhandledRejection', record,);
    },
  };
}

await describe({
  name: 'createLogger orchestration',
  children: [
    it({
      name: 'fans each record out to every available sink',
      fn: async () => {
        const a = recordingSink();
        const b = recordingSink();
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [a.sink, b.sink,], },);
        await initPromise;

        logger.info('hello',);
        await logger.flush();

        expect(messages({ recording: a, },),)
          .toEqual(['hello',],);
        expect(messages({ recording: b, },),)
          .toEqual(['hello',],);
      },
    },),

    it({
      name: 'buffers a pre-verify record and replays it to a late-verifying sink exactly once',
      fn: async () => {
        const late = recordingSink({
          verify: function verifyLater(): Promise<boolean> {
            // Resolves on a microtask, after the synchronous log call, so the
            // record must buffer and replay rather than write immediately.
            return Promise.resolve(true,);
          },
        },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [late.sink,], },);

        // Logged synchronously, before the async verify resolves: the record
        // must buffer rather than drop, then replay once on verify.
        logger.info('early',);
        expect(late.records,)
          .toEqual([],);

        await initPromise;
        await logger.flush();
        expect(messages({ recording: late, },),)
          .toEqual(['early',],);
      },
    },),

    it({
      name: 'delivers a startup record once to both an immediately-available and a late sink',
      fn: async () => {
        const eager = recordingSink();
        const late = recordingSink({
          verify: function verifyLater(): Promise<boolean> {
            // Resolves on a microtask, after the synchronous log call, so the
            // record must buffer and replay rather than write immediately.
            return Promise.resolve(true,);
          },
        },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [eager.sink, late.sink,], },);

        logger.info('boot',);
        await initPromise;
        await logger.flush();

        // Exactly once each: the eager sink via the immediate write, the late
        // sink via replay. No double-delivery to the eager sink.
        expect(messages({ recording: eager, },),)
          .toEqual(['boot',],);
        expect(messages({ recording: late, },),)
          .toEqual(['boot',],);
      },
    },),

    it({
      name: 'drops a sink whose verify resolves false',
      fn: async () => {
        const off = recordingSink({
          verify: function verifyUnavailable(): Promise<boolean> {
            return Promise.resolve(false,);
          },
        },);
        const on = recordingSink();
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [off.sink, on.sink,], },);
        await initPromise;

        logger.info('x',);
        await logger.flush();
        expect(off.records,)
          .toEqual([],);
        expect(messages({ recording: on, },),)
          .toEqual(['x',],);
      },
    },),

    it({
      name: 'drops a sink whose verify throws or rejects',
      fn: async () => {
        const thrower = recordingSink({
          verify: function verifyThrows(): Promise<boolean> {
            // Throws synchronously, before returning a promise; the logger's
            // try around `await verify()` still catches it.
            throw new Error('sync verify failed',);
          },
        },);
        const rejecter = recordingSink({
          verify: async function verifyRejects(): Promise<boolean> {
            // Rejects after a microtask; awaiting it in the logger rejects and
            // is caught.
            await Promise.resolve();
            throw new Error('async verify failed',);
          },
        },);
        const on = recordingSink();
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [thrower.sink, rejecter.sink, on.sink,], },);
        await initPromise;

        logger.info('x',);
        await logger.flush();
        expect(thrower.records,)
          .toEqual([],);
        expect(rejecter.records,)
          .toEqual([],);
        expect(messages({ recording: on, },),)
          .toEqual(['x',],);
      },
    },),

    it({
      name: 'throws once initialized with no available backend',
      fn: async () => {
        const off = recordingSink({
          verify: function verifyUnavailable(): Promise<boolean> {
            return Promise.resolve(false,);
          },
        },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [off.sink,], },);
        await initPromise;

        expect(function logWithNoBackend() {
          logger.info('x',);
        },)
          .toThrow('No logging backends available',);
      },
    },),

    it({
      name: 'a rejecting write does not retire the sink',
      fn: async () => {
        /**
         Write-attempt counter; a retired sink would stop receiving writes,
         so a second attempt proves the rejection left the backend available.
         */
        const counters: { attempts: number; } = { attempts: 0, };
        const flaky: Sink = {
          verify: function verifyAvailable(): Promise<boolean> {
            return Promise.resolve(true,);
          },
          write: async function write(): Promise<void> {
            counters.attempts++;
            throw new Error('transient write failure',);
          },
        };
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [flaky,], },);
        await initPromise;

        logger.info('one',);
        await logger.flush();
        // Still available, so this neither throws nor is skipped.
        logger.info('two',);
        await logger.flush();

        expect(counters.attempts,)
          .toBe(2,);
      },
    },),

    it({
      name: 'flush drains a still-pending write before resolving',
      fn: async () => {
        const slow = recordingSink({ writeDelayMs: SLOW_WRITE_MS, },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [slow.sink,], },);
        await initPromise;

        logger.info('drains',);
        // The slow write is still parked, so nothing has been recorded yet.
        expect(slow.records,)
          .toEqual([],);

        await logger.flush();
        expect(messages({ recording: slow, },),)
          .toEqual(['drains',],);
      },
    },),

    it({
      name: 'flush runs every available sink flush hook',
      fn: async () => {
        /**
         Hook-invocation counter proving `flush()` reached the sink's own hook.
         */
        const counters: { flushes: number; } = { flushes: 0, };
        const hooked = recordingSink({
          flush: async function flushHook(): Promise<void> {
            counters.flushes++;
          },
        },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [hooked.sink,], },);
        await initPromise;

        await logger.flush();
        expect(counters.flushes,)
          .toBe(1,);
      },
    },),

    it({
      name: 'a rejecting flush hook disables that sink without failing the aggregate flush',
      fn: async () => {
        const bad = recordingSink({
          flush: async function flushHook(): Promise<void> {
            throw new Error('flush hook failed',);
          },
        },);
        const good = recordingSink();
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [bad.sink, good.sink,], },);
        await initPromise;

        await expect(logger.flush(),)
          .resolves
          .toBeUndefined();

        // The rejecting hook retired its sink; subsequent records skip it.
        logger.info('after',);
        await logger.flush();
        expect(bad.records,)
          .toEqual([],);
        expect(messages({ recording: good, },),)
          .toEqual(['after',],);
      },
    },),

    it({
      name: 'writes a mid-init record immediately to an already-available sink and replays it once to a still-verifying sink',
      fn: async () => {
        const eager = recordingSink();
        const late = recordingSink({
          verify: async function verifyLate(): Promise<boolean> {
            // Still parked when the record is logged, so the eager sink (whose
            // microtask verify already resolved) takes the immediate write
            // while this sink only receives the record via replay on verify.
            await wait(SLOW_WRITE_MS,);
            return true;
          },
        },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [eager.sink, late.sink,], },);

        // Halfway through the late sink's verify: eager has flipped available,
        // late has not, and `initialize()` has not yet completed.
        await wait(SLOW_WRITE_MS / 2,);
        logger.info('mid',);

        await initPromise;
        await logger.flush();

        // Eager via the immediate mid-init write, late via replay: each exactly
        // once. A regression that replayed startup records to already-available
        // sinks would make `eager` ['mid', 'mid'].
        expect(messages({ recording: eager, },),)
          .toEqual(['mid',],);
        expect(messages({ recording: late, },),)
          .toEqual(['mid',],);
      },
    },),

    it({
      name: 'a synchronously-throwing write does not retire the sink',
      fn: async () => {
        /**
         Write-attempt counter; a retired sink would stop receiving writes, so
         a second attempt proves the synchronous throw left it available.
         */
        const counters: { attempts: number; } = { attempts: 0, };
        const flaky: Sink = {
          verify: function verifyAvailable(): Promise<boolean> {
            return Promise.resolve(true,);
          },
          write: function write(): Promise<void> {
            counters.attempts++;
            // Throws synchronously, before returning a promise; the logger's
            // try around the `write()` call swallows it without retiring the
            // sink (distinct from a rejected promise, handled by `trackWrite`).
            throw new Error('synchronous write failure',);
          },
        };
        const on = recordingSink();
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [flaky, on.sink,], },);
        await initPromise;

        logger.info('one',);
        await logger.flush();
        // Still available, so this neither throws nor is skipped.
        logger.info('two',);
        await logger.flush();

        expect(counters.attempts,)
          .toBe(2,);
        // The healthy sibling keeps receiving every record.
        expect(messages({ recording: on, },),)
          .toEqual(['one', 'two',],);
      },
    },),

    it({
      name: 'does not run the flush hook of a sink that failed verification',
      fn: async () => {
        /**
         Flush-hook counter; stays zero because an unavailable sink's hook
         must be skipped by `flushAll`.
         */
        const counters: { flushes: number; } = { flushes: 0, };
        const off = recordingSink({
          verify: function verifyUnavailable(): Promise<boolean> {
            return Promise.resolve(false,);
          },
          flush: async function flushHook(): Promise<void> {
            counters.flushes++;
          },
        },);
        const on = recordingSink();
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [off.sink, on.sink,], },);
        await initPromise;

        await logger.flush();
        expect(counters.flushes,)
          .toBe(0,);
      },
    },),

    it({
      name: 'throws once initialized with an empty sink list',
      fn: async () => {
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [], },);
        await initPromise;

        expect(function logWithNoSinks() {
          logger.info('x',);
        },)
          .toThrow('No logging backends available',);
      },
    },),

    //region Breadcrumb suites (each attempt owns its console.warn replacement)

    describe({
      name: 'breadcrumb suites',
      children: [
        describe({
          name: 'flush deadline',
          children: [
            it({
              name: 'exports a positive default flush deadline',
              fn: async () => {
                expect(DEFAULT_FLUSH_DEADLINE_MS,)
                  .toBeGreaterThan(0,);
              },
            },),

            it({
              name: 'flush resolves once the deadline elapses when a write never settles',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [wedgedWriteSink(),],
                },);
                await initPromise;
                logger.info('stuck',);

                const elapsed = await timeFlush({ flush: logger.flush, },);
                expect(elapsed,)
                  .toBeGreaterThanOrEqual(SHORT_DEADLINE_MS - DEADLINE_TOLERANCE_MS,);
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(1,);
                expect(deadlineBreadcrumbMessages({ warn, },)[0],)
                  .toContain(`${SHORT_DEADLINE_MS}ms`,);
              },
            },),

            it({
              name: 'a second flush after an abandoned write does not wait out the deadline again',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                sinon.stub(
                  console,
                  'warn',
                );
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [wedgedWriteSink(),],
                },);
                await initPromise;
                logger.info('stuck',);
                await logger.flush();

                const elapsed = await timeFlush({ flush: logger.flush, },);
                expect(elapsed,)
                  .toBeLessThan(FAST_FLUSH_MS,);
              },
            },),

            it({
              name: 'flush resolves once the deadline elapses when a flush hook never settles',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                const hookWedged = recordingSink({
                  flush: function flushForever(): Promise<void> {
                    return neverSettles();
                  },
                },);
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [hookWedged.sink,],
                },);
                await initPromise;

                const elapsed = await timeFlush({ flush: logger.flush, },);
                expect(elapsed,)
                  .toBeGreaterThanOrEqual(SHORT_DEADLINE_MS - DEADLINE_TOLERANCE_MS,);
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(1,);
              },
            },),

            it({
              name: 'flush resolves once the deadline elapses when a verify never settles',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                const verifyWedged = recordingSink({
                  verify: function verifyForever(): Promise<boolean> {
                    return neverSettles();
                  },
                },);
                const { logger, } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [verifyWedged.sink,],
                },);

                const elapsed = await timeFlush({ flush: logger.flush, },);
                expect(elapsed,)
                  .toBeGreaterThanOrEqual(SHORT_DEADLINE_MS - DEADLINE_TOLERANCE_MS,);
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(1,);
              },
            },),

            it({
              name: 'a flush that settles inside the deadline reports no breadcrumb',
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                const quick = recordingSink({ writeDelayMs: 1, },);
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [quick.sink,],
                },);
                await initPromise;
                logger.info('fast',);
                await logger.flush();

                expect(messages({ recording: quick, },),)
                  .toEqual(['fast',],);
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(0,);
              },
            },),
          ],
        },),

        describe({
          name: 'verify liveness',
          children: [
            it({
              name: 'exports a positive default verify timeout',
              fn: async () => {
                expect(DEFAULT_VERIFY_TIMEOUT_MS,)
                  .toBeGreaterThan(0,);
              },
            },),

            it({
              name: 'a verify that never settles no longer starves the sinks after it',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                const wedged = recordingSink({
                  verify: function verifyForever(): Promise<boolean> {
                    return neverSettles();
                  },
                },);
                const later = recordingSink();
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  sinks: [
                    wedged.sink,
                    later.sink,
                  ],
                  verifyTimeoutMs: SHORT_DEADLINE_MS,
                },);
                await initPromise;
                logger.info('after init',);
                await logger.flush();

                expect(messages({ recording: later, },),)
                  .toEqual(['after init',],);
                expect(messages({ recording: wedged, },),)
                  .toEqual([],);
                expect(verifyBreadcrumbs({ warn, },),)
                  .toBe(1,);
              },
            },),

            it({
              name: 'a verify that answers after the time limit stays unavailable',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                sinon.stub(
                  console,
                  'warn',
                );
                const slow = recordingSink({
                  verify: verifyTrueAfter({ delayMs: SHORT_DEADLINE_MS * 3, },),
                },);
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  sinks: [slow.sink,],
                  verifyTimeoutMs: SHORT_DEADLINE_MS,
                },);
                await initPromise;
            // Let the late answer arrive, then log and drain.
                await wait(SHORT_DEADLINE_MS * 4,);
                expect(function logAfterLateAnswer() {
                  logger.info('late',);
                },)
                  .toThrow('No logging backends available',);
                await logger.flush();

                expect(messages({ recording: slow, },),)
                  .toEqual([],);
              },
            },),

            it({
              name: 'sinks verify concurrently rather than one after another',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async () => {
                const firstStarted = Promise.withResolvers<void>();
                const secondStarted = Promise.withResolvers<void>();
                const release = Promise.withResolvers<void>();
                const first = recordingSink({ verify: async function firstVerification() {
                  firstStarted.resolve();
                  await release.promise;
                  return true;
                }, });
                const second = recordingSink({ verify: async function secondVerification() {
                  secondStarted.resolve();
                  await release.promise;
                  return true;
                }, });
                const { initPromise, } = createLogger({
                  sinks: [
                    first.sink,
                    second.sink,
                  ],
                },);
            // Both verifications must start before either can complete.
                await Promise.all([firstStarted.promise, secondStarted.promise]);
                release.resolve();
                await initPromise;
              },
            },),

            it({
              name: 'a record logged while sinks verify at different speeds reaches each exactly once',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async () => {
                const quick = recordingSink({ verify: verifyTrueAfter({ delayMs: 1, },), },);
                const slow = recordingSink({ verify: verifyTrueAfter({ delayMs: SLOW_WRITE_MS, },), },);
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  sinks: [
                    quick.sink,
                    slow.sink,
                  ],
                },);
                logger.info('early',);
                await initPromise;
                logger.info('late',);
                await logger.flush();

                expect(messages({ recording: quick, },),)
                  .toEqual([
                    'early',
                    'late',
                  ],);
                expect(messages({ recording: slow, },),)
                  .toEqual([
                    'early',
                    'late',
                  ],);
              },
            },),
          ],
        },),

        describe({
          name: 'abandoned writes',
          children: [
            it({
              name: 'a write rejecting after the deadline abandoned it is reported once, never left unhandled, and does not retire the sink',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                using unhandled = captureUnhandledRejections();
                const parked = parkedFirstWriteSink();
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [parked.sink,],
                },);
                await initPromise;
                logger.info('parked',);
                await logger.flush();
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(1,);

                parked.settleParked.reject(new Error('late failure',),);
                await wait(1,);
                logger.info('after',);

                const elapsed = await timeFlush({ flush: logger.flush, },);
                expect(elapsed,)
                  .toBeLessThan(FAST_FLUSH_MS,);
                expect(messages({ recording: parked, },),)
                  .toEqual(['after',],);
                expect(unhandled.reasons,)
                  .toHaveLength(0,);
                expect(
                  breadcrumbMessages({
                    warn,
                    needle: 'sink write promise rejected while being tracked',
                  },),
                )
                  .toHaveLength(1,);
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(1,);
              },
            },),

            it({
              name: 'a write resolving after the deadline abandoned it still lands and the sink keeps working',
              timeout: DEADLINE_TEST_TIMEOUT_MS,
              fn: async ({ sinon, },) => {
                const warn = sinon.stub(
                  console,
                  'warn',
                );
                using unhandled = captureUnhandledRejections();
                const parked = parkedFirstWriteSink();
                const {
                  logger,
                  initPromise,
                } = createLogger({
                  flushDeadlineMs: SHORT_DEADLINE_MS,
                  sinks: [parked.sink,],
                },);
                await initPromise;
                logger.info('parked',);
                await logger.flush();

                parked.settleParked.resolve();
                await wait(1,);
                logger.info('after',);
                await logger.flush();

                expect(messages({ recording: parked, },),)
                  .toEqual([
                    'parked',
                    'after',
                  ],);
                expect(unhandled.reasons,)
                  .toHaveLength(0,);
                expect(
                  breadcrumbMessages({
                    warn,
                    needle: 'sink write promise rejected',
                  },),
                )
                  .toHaveLength(0,);
                expect(deadlineBreadcrumbs({ warn, },),)
                  .toBe(1,);
              },
            },),
          ],
        },),
      ],
    },),

    //endregion Breadcrumb suites

    //region Startup buffer bound

    it({
      name: 'exports a positive startup buffer cap',
      fn: async () => {
        expect(STARTUP_BUFFER_CAP,)
          .toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'a startup burst beyond the cap keeps the newest records and reports the loss once',
      timeout: DEADLINE_TEST_TIMEOUT_MS,
      fn: async () => {
        /**
         Records logged before the sink verifies: the cap plus a few extra
         that must push the oldest ones out.
         */
        const extra = 3;
        /**
         Total records in the burst; the last one is index `burstSize - 1`.
         */
        const burstSize = STARTUP_BUFFER_CAP + extra;
        const late = recordingSink({ verify: verifyTrueAfter({ delayMs: SLOW_WRITE_MS, },), },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [late.sink,], },);
        for (let index = 0; index < burstSize; index += 1)
          logger.info(`burst ${index}`,);
        await initPromise;
        await logger.flush();

        const received = messages({ recording: late, },);
        expect(received,)
          .toHaveLength(STARTUP_BUFFER_CAP + 1,);
        expect(received[0],)
          .toBe(`burst ${extra}`,);
        expect(received[STARTUP_BUFFER_CAP - 1],)
          .toBe(`burst ${burstSize - 1}`,);
        expect(received[STARTUP_BUFFER_CAP],)
          .toBe(`${extra} startup records dropped before a backend verified (buffer cap ${STARTUP_BUFFER_CAP})`,);
        expect(late.records[STARTUP_BUFFER_CAP]?.level,)
          .toBe('warn',);
      },
    },),

    it({
      name: 'no marker record is written when the startup buffer never overflowed',
      fn: async () => {
        const late = recordingSink({ verify: verifyTrueAfter({ delayMs: 1, },), },);
        const {
          logger,
          initPromise,
        } = createLogger({ sinks: [late.sink,], },);
        logger.info('one',);
        await initPromise;
        await logger.flush();

        expect(messages({ recording: late, },),)
          .toEqual(['one',],);
      },
    },),

    //endregion Startup buffer bound
  ],
},);
