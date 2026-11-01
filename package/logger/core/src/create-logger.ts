import { reportLoggerInternalError, } from './error-format.ts';
import { createPendingWrites, } from './pending-writes.ts';
import { withHostTimeout, } from './with-host-timeout.ts';

import type {
  Level,
  Logger,
  LogRecord,
  Sink,
} from './types.ts';

/**
 Default `flush()` deadline in milliseconds. Measured on 2026-09-06: a
 default logger flushing 100 records through the console and file sinks
 settles in about 2 ms locally, so this leaves three orders of magnitude for
 a slow but working backend while still bounding shutdown on a wedged one.
 Override per logger through the `flushDeadlineMs` option of
 {@link createLogger}. Hosts without both timer primitives cannot enforce
 this deadline.
 */
export const DEFAULT_FLUSH_DEADLINE_MS = 5_000;

/**
 Default per-sink `verify()` time limit in milliseconds. Measured on
 2026-09-06: the default logger's five shipped verifies complete together in
 about 2.4 ms locally, so this leaves three orders of magnitude for a slow
 but working backend probe (a network filesystem, a busy IndexedDB) while a
 verify that never answers (a hung mount, an IndexedDB open blocked by
 another tab) cannot stall startup when the host exposes timers.
 Override per logger through the `verifyTimeoutMs` option of
 {@link createLogger}.
 */
export const DEFAULT_VERIFY_TIMEOUT_MS = 5_000;

/**
 Most records the logger buffers before its sinks have verified. Startup
 lasts at most {@link DEFAULT_VERIFY_TIMEOUT_MS} when the host exposes timers,
 so this bounds the memory a burst during that window can claim; on overflow
 the oldest buffered record
 is dropped so the newest (usually most diagnostic) context survives, and
 one synthetic `warn` record naming the dropped count is written to every
 available sink once initialization completes.
 */
export const STARTUP_BUFFER_CAP = 10_000;

/**
 Sink paired with its current availability inside one logger instance.
 Availability starts `false` (not-yet-verified reads as unavailable,
 identical to a verified failure) and flips `true` once the sink's own
 `verify` confirms its backend.
 */
type SinkEntry = {
  available: boolean;
  readonly sink: Sink;
};

/**
 Builds a multi-sink logger over the supplied sink adapters. All
 orchestration (per-sink availability, startup buffering and replay,
 in-flight write tracking, and flush) lives here; the exported default
 `logger` is just this factory applied to the default sink set, and tests
 apply it to fake sinks to exercise the orchestration directly.
 
 Verification runs eagerly at construction and never blocks callers:
 records emitted while an async sink is still verifying buffer internally
 and replay to that sink the moment it verifies. Every sink verifies
 concurrently under its own time limit (`verifyTimeoutMs`, default
 {@link DEFAULT_VERIFY_TIMEOUT_MS}) when the host exposes timers, so one
 backend that never answers cannot starve the others in those runtimes. A sink
 whose `verify` resolves `false`, throws, or runs past the limit is dropped
 and receives no records; an answer that arrives after the limit is
 ignored. A rejected `write` is the sink's own concern and does not disable
 the backend.
 
 With host timers, `flush()` always resolves: one deadline (`flushDeadlineMs`,
 default {@link DEFAULT_FLUSH_DEADLINE_MS}) wraps startup verification, the
 in-flight write drain, and every sink flush hook together. When it elapses
 the logger reports one breadcrumb, abandons the tracked writes from its
 view (the sinks expose no cancellation, so the underlying work continues),
 and resolves, so a wedged backend cannot hang a shutdown. Without both
 `setTimeout` and `clearTimeout`, a timerless host such as QuickJS-ng awaits
 operations without deadlines; a custom sink that never settles can then
 prevent initialization or flush from completing.
 
 @param sinks - Sink adapters to fan each record out to, in priority order.
 
 @param flushDeadlineMs - Milliseconds one `flush()` may take before it
 resolves anyway; raise it for slow but working backends such as network
 filesystems.
 
 @param verifyTimeoutMs - Milliseconds one sink's `verify()` may take before
 the sink counts as unavailable; raise it for a slow but working probe.
 
 @returns Logger plus its eager `initPromise`; callers need not await
 `initPromise` before logging, since startup records replay on verify.
 
 @example
 ```ts
 const { logger } = createLogger({ sinks: [createConsoleSink()] });
 logger.info('ready');
 await logger.flush();
 ```
 
 @example
 ```ts
 const { logger } = createLogger({
   sinks: [createFileSink()],
   flushDeadlineMs: 30_000,
 });
 ```
 */
export function createLogger(
  {
    sinks,
    flushDeadlineMs = DEFAULT_FLUSH_DEADLINE_MS,
    verifyTimeoutMs = DEFAULT_VERIFY_TIMEOUT_MS,
  }: {
    readonly sinks: readonly Sink[];
    readonly flushDeadlineMs?: number;
    readonly verifyTimeoutMs?: number;
  },
): {
  readonly initPromise: Promise<void>;
  readonly logger: Logger;
} {
  /**
   Per-sink availability for this logger instance, indexed by startup order.
   */
  const entries: SinkEntry[] = sinks.map(function toEntry(sink,): SinkEntry {
    return {
      available: false,
      sink,
    };
  },);

  /**
   Log records emitted before every sink has completed startup verification.
   Records stay here only during initialization; each sink that later
   verifies as available receives a replay, while already-available sinks
   still receive writes immediately.
   */
  const startupRecords: LogRecord[] = [];

  /**
   Sink writes currently in flight. Logger-level `flush()` drains these so
   sinks without their own flush hook, such as file writes, still settle
   before the flush resolves.
   */
  const pendingWrites = createPendingWrites();

  /**
   Instance-local aggregate flags. `initialized` flips true once the eager
   `initialize()` settles; `hasAvailableSink` reflects whether any sink
   survives verification and is recomputed by `recomputeAvailability`;
   `droppedStartupRecords` counts startup-buffer overflow drops for the
   post-initialization marker record.
   */
  const state: {
    droppedStartupRecords: number;
    hasAvailableSink: boolean;
    initialized: boolean;
  } = {
    droppedStartupRecords: 0,
    hasAvailableSink: false,
    initialized: false,
  };

  /**
   Buffers a pre-initialization record under {@link STARTUP_BUFFER_CAP},
   dropping the oldest buffered record (and counting the drop) when the cap
   is reached.
   
   @param record - Record logged before every sink has verified.
   */
  function bufferStartupRecord({ record, }: { readonly record: LogRecord; },): void {
    if (startupRecords.length >= STARTUP_BUFFER_CAP) {
      startupRecords.shift();
      state.droppedStartupRecords += 1;
    }
    startupRecords.push(record,);
  }

  /**
   Recomputes aggregate availability after a sink entry's flag flips.
   */
  function recomputeAvailability(): void {
    state.hasAvailableSink = entries.some(function isAvailable(entry,) {
      return entry.available;
    },);
  }

  /**
   Reads a sink entry by startup-order index.
   
   @param entryIndex - Sink entry index from the construction-time order.
   
   @returns Sink entry at that index.
   
   @throws Error when index no longer maps to a sink entry.
   */
  function getSinkEntry({ entryIndex, }: { readonly entryIndex: number; },): SinkEntry {
    /**
     Sink entry read from startup-order storage; undefined means the caller supplied an invalid index.
     */
    const entry = entries[entryIndex];
    if (entry === undefined)
      throw new Error(`Missing logger sink entry at index ${entryIndex}.`,);

    return entry;
  }

  /**
   Marks a sink unavailable after its verification fails or throws. Write
   failures never reach here: a rejected write is the sink's own concern and
   leaves the backend available, so one transient hiccup does not retire it.
   
   @param entryIndex - Sink entry index whose backend failed verification.
   */
  function markEntryUnavailable({ entryIndex, }: { readonly entryIndex: number; },): void {
    /**
     Mutable sink entry being disabled.
     */
    const entry = getSinkEntry({ entryIndex, },);
    entry.available = false;
    recomputeAvailability();
  }

  /**
   Sends a record to one sink, recording the write so `flush()` can await it.
   
   @param entryIndex - Available sink entry index to receive the record.
   
   @param record - Log record to deliver.
   */
  function writeRecordToEntry(
    {
      entryIndex,
      record,
    }: {
      readonly entryIndex: number;
      readonly record: LogRecord;
    },
  ): void {
    try {
      /**
       Sink entry receiving the record.
       */
      const entry = getSinkEntry({ entryIndex, },);
      pendingWrites.add(entry.sink
        .write(record,),);
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'sink write threw synchronously while dispatching record',
        error,
      },);
    }
  }

  /**
   Replays buffered startup records to a sink that just became available.
   
   @param entryIndex - Newly available sink entry index.
   */
  function replayStartupRecordsToEntry({ entryIndex, }: { readonly entryIndex: number; },): void {
    startupRecords.forEach(function replayStartupRecord(record,) {
      writeRecordToEntry({
        entryIndex,
        record,
      },);
    },);
  }

  /**
   Applies a verification result to a sink and replays startup records on
   success.
   
   @param entryIndex - Sink entry index whose verification completed.
   
   @param available - Whether the backend verified successfully.
   */
  function setEntryAvailability(
    {
      entryIndex,
      available,
    }: {
      readonly entryIndex: number;
      readonly available: boolean;
    },
  ): void {
    /**
     Sink entry whose availability is changing.
     */
    const entry = getSinkEntry({ entryIndex, },);
    entry.available = available;
    recomputeAvailability();
    if (available)
      replayStartupRecordsToEntry({ entryIndex, },);
  }

  /**
   Runs one sink's verification under the verify time limit when host timers
   exist, and records the result, replaying buffered startup records on
   success. A rejected verification, a synchronous throw from the verifier,
   or a timed verify that runs past `verifyTimeoutMs` drops the sink;
   a late answer after the limit
   is never observed, so it cannot flip availability afterwards.
   
   @param entryIndex - Sink entry index to verify.
   */
  async function verifyAndApply({ entryIndex, }: { readonly entryIndex: number; },): Promise<void> {
    try {
      /**
       Sink entry whose verifier is about to run.
       */
      const entry = getSinkEntry({ entryIndex, },);
      setEntryAvailability({
        available: await withHostTimeout({
          label: `sink ${entryIndex} verify`,
          ms: verifyTimeoutMs,
          promise: entry.sink
            .verify(),
        },),
        entryIndex,
      },);
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: `sink verification failed for entry ${entryIndex}`,
        error,
      },);
      markEntryUnavailable({ entryIndex, },);
    }
  }

  /**
   Writes one synthetic `warn` record to every available sink when the
   startup buffer overflowed, so the loss is never silent. Runs once, after
   initialization, when the dropped count is known and final.
   */
  function emitDroppedStartupMarker(): void {
    /**
     Records dropped from the startup buffer; zero means nothing to report.
     */
    const dropped = state.droppedStartupRecords;
    if (dropped === 0)
      return;
    /**
     Marker record naming the loss; the noun agrees with the count.
     */
    const marker: LogRecord = {
      level: 'warn',
      message: `${dropped} startup record${(dropped === 1) ? '' : 's'} dropped before a backend verified (buffer cap ${STARTUP_BUFFER_CAP})`,
      timestamp: Date.now(),
    };
    entries.forEach(function writeMarker(
      entry,
      entryIndex,
    ) {
      if (entry.available) {
        writeRecordToEntry({
          entryIndex,
          record: marker,
        },);
      }
    },);
  }

  /**
   Initializes all sink backends by verifying their availability. Runs once
   at construction. Every verifier runs concurrently, each under the verify
   time limit, so one backend that never answers cannot starve the rest or
   keep `initialized` from flipping. Completion order does not affect
   correctness: a record's immediate-write set (sinks already available when
   it was logged) and its replay set (sinks that become available later) are
   disjoint, so each available sink receives each record exactly once
   whichever verify settles first.
   */
  async function initialize(): Promise<void> {
    if (state.initialized)
      return;

    await Promise.all(
      entries.map(function verifyEntry(
        _entry,
        entryIndex,
      ) {
        return verifyAndApply({ entryIndex, },);
      },),
    );

    state.initialized = true;
    startupRecords.length = 0;
    emitDroppedStartupMarker();
  }

  /**
   Eager readiness promise. Consumers do not need to await this before
   logging; `flush()` awaits it internally, and startup records replay to
   async sinks as they become available.
   */
  const initPromise: Promise<void> = initialize();

  /**
   Creates a logging method for the specified severity level.
   
   @param level - Log severity level for messages from this method.
   
   @returns Logging function for the given level.
   */
  function createMethod(level: Level,): (message: string,) => void {
    return function logAtLevel(message: string,): void {
      if ((!state.hasAvailableSink) && state
        .initialized)
        throw new Error('No logging backends available',);

      /**
       Shared LogRecord forwarded to every available sink; built once per call so all sinks see the same timestamp.
       */
      const record: LogRecord = {
        level,
        message,
        timestamp: Date.now(),
      };

      if (!state.initialized)
        bufferStartupRecord({ record, },);

      /**
       Indices of sinks that survived verification; recomputed per call so a sink dropped at verify time is excluded next time.
       */
      const availableIndices = entries
        .map(function indexEntry(
          _entry,
          entryIndex,
        ) {
          return entryIndex;
        },)
        .filter(function isAvailable(entryIndex,) {
          return getSinkEntry({ entryIndex, },)
            .available;
        },);

      availableIndices.forEach(function writeToSink(entryIndex,) {
        writeRecordToEntry({
          entryIndex,
          record,
        },);
      },);
    };
  }

  /**
   Runs every available sink's own `flush` hook. A rejecting hook marks that
   sink unavailable and does not fail the aggregate.
   */
  async function runSinkFlushHooks(): Promise<void> {
    await Promise.all(
      entries.map(async function runFlush(
        entry,
        entryIndex,
      ) {
        /**
         Optional sink-supplied flush hook; absent when the sink writes synchronously and needs no draining.
         */
        const sinkFlush = entry.sink
          .flush;
        if ((!entry.available) || ((typeof sinkFlush) !== 'function'))
          return;
        try {
          await sinkFlush();
        }
        catch (error: unknown) {
          reportLoggerInternalError({
            context: `sink flush failed for entry ${entryIndex}`,
            error,
          },);
          markEntryUnavailable({ entryIndex, },);
        }
      },),
    );
  }

  /**
   Drains startup verification, observed sink completions, and every sink flush hook (via
   {@link runSinkFlushHooks}) in order. Never rejects on its own: every
   inner failure is already reported and swallowed, so the only way this
   stays pending is a verify, write, or hook that never settles.
   */
  async function drainEverything(): Promise<void> {
    await initPromise;
    await pendingWrites.drain();
    await runSinkFlushHooks();
  }

  /**
   Drops every tracked write from the logger's view after the flush deadline
   elapsed. The tracked wrappers keep their own rejection handling, so a
   late settlement can neither surface as an unhandled rejection nor stall
   the next `flush()`. The underlying sink work is not cancelled: sinks
   expose no cancellation signal.
   */
  function abandonPendingWrites(): void {
    pendingWrites.clear();
  }

  /**
   Runs {@link drainEverything} under the flush deadline when host timers
   exist. Resolves once all tracked writes and hooks settle, or once the
   deadline elapses; a deadline hit reports one breadcrumb and abandons the
   tracked writes. In timerless hosts it waits for all work to settle.
   */
  async function flushAll(): Promise<void> {
    try {
      await withHostTimeout({
        label: 'logger flush',
        ms: flushDeadlineMs,
        promise: drainEverything(),
      },);
    }
    catch (error: unknown) {
      /**
       Whether the timeout utility's own deadline caused this failure.
       */
      const deadlineElapsed = Error.isError(error)
        ? error.message === `Timed out after ${flushDeadlineMs}ms: logger flush`
        : false;
      reportLoggerInternalError({
        context: deadlineElapsed
          ? `flush deadline of ${flushDeadlineMs}ms elapsed; abandoning in-flight sink work`
          : 'flush failed before completion; abandoning in-flight sink work',
        error,
      },);
      abandonPendingWrites();
    }
  }

  /**
   Multi-sink logger that writes to all available backends.
   */
  const logger: Logger = {
    debug: createMethod('debug',),
    error: createMethod('error',),
    fatal: createMethod('fatal',),
    flush: flushAll,
    info: createMethod('info',),
    trace: createMethod('trace',),
    warn: createMethod('warn',),
  };

  return {
    initPromise,
    logger,
  };
}
