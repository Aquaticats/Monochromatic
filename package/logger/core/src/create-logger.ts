import {
  reportLoggerCondition,
  reportLoggerInternalError,
} from './error-format.ts';
import { createPendingWrites, } from './pending-writes.ts';
import { watchSinkProbes, } from './sink-probe-watch.ts';
import { settleWithinHostTimeout, } from './with-host-timeout.ts';

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
 but working backend probe (a network filesystem, a busy IndexedDB).
 The limit bounds how long startup waits for a probe before logging proceeds;
 it never decides whether a sink's records are kept, because a probe that
 answers after the limit still attaches its sink and receives the records kept
 for it. A probe that never answers (a hung mount, an IndexedDB open blocked by
 another tab) therefore costs one bounded wait and no records.
 Override per logger through the `verifyTimeoutMs` option of
 {@link createLogger}.
 */
export const DEFAULT_VERIFY_TIMEOUT_MS = 5_000;

/**
 Most records the logger buffers for sinks whose verification has not answered
 yet. Startup waits at most {@link DEFAULT_VERIFY_TIMEOUT_MS} per sink, so this
 bounds the memory a burst during that window can claim; a probe that answers
 later keeps that bound for the wait until its records are replayed or dropped.
 On overflow the oldest buffered record
 is dropped so the newest (usually most diagnostic) context survives, and
 one synthetic `warn` record naming the dropped count is written to every
 available sink once every sink has a verdict.
 */
export const STARTUP_BUFFER_CAP = 10_000;

/**
 Sink paired with its current availability inside one logger instance.
 Availability starts `false` (not-yet-verified reads as unavailable,
 identical to a verified failure) and flips `true` once the sink's own
 `verify` confirms its backend. `decided` records whether that verdict has
 arrived at all, so the logger knows whether a record is still owed to the
 sink.
 */
type SinkEntry = {
  available: boolean;
  decided: boolean;
  readonly sink: Sink;
};

/**
 Builds a multi-sink logger over the supplied sink adapters. All
 orchestration (per-sink availability, startup buffering and replay,
 in-flight write tracking, and flush) lives here; the exported default
 `logger` is just this factory applied to the default sink set, and tests
 apply it to fake sinks to exercise the orchestration directly.
 
 Verification runs eagerly at construction and never blocks callers:
 records emitted while an async sink is still verifying are kept for it and
 replay the moment it answers. Every sink verifies
 concurrently under its own time limit (`verifyTimeoutMs`, default
 {@link DEFAULT_VERIFY_TIMEOUT_MS}) when the host exposes timers, so one
 backend that never answers cannot starve the others in those runtimes. A sink
 whose `verify` resolves `false` or throws is dropped and receives no records.
 Running past the limit drops nothing: the logger stops waiting for that probe
 so startup can continue, and applies its answer whenever it arrives, so a
 backend that a busy host merely delayed still receives the records logged
 meanwhile. A rejected `write` is the sink's own concern and does not disable
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
 
 @param verifyTimeoutMs - Milliseconds startup waits for one sink's `verify()`
 before logging proceeds without it; raise it for a slow but working probe.
 The limit never decides whether a sink receives the records kept for it.
 
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
      decided: false,
      sink,
    };
  },);

  /**
   Log records emitted while at least one sink's verification has not
   answered. Each such sink receives a replay when it answers, while
   already-available sinks still receive writes immediately; the buffer is
   released once every sink has a verdict.
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
   `undecidedEntries` counts sinks whose verification has not answered yet, so
   records for them stay buffered; `droppedStartupRecords` counts startup-buffer
   overflow drops for the release marker record.
   */
  const state: {
    droppedStartupRecords: number;
    hasAvailableSink: boolean;
    initialized: boolean;
    startupRecordsReleased: boolean;
    undecidedEntries: number;
  } = {
    droppedStartupRecords: 0,
    hasAvailableSink: false,
    initialized: false,
    startupRecordsReleased: false,
    undecidedEntries: entries.length,
  };

  /**
   Buffers a record logged while at least one sink has no verdict yet, under
   {@link STARTUP_BUFFER_CAP}, dropping the oldest buffered record (and counting
   the drop) when the cap is reached.
   
   @param record - Record logged while a sink's verification is outstanding.
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
   Marks a sink unavailable after its verification answered negative, threw, or
   a flush hook failed. Write failures never reach here: a rejected write is the
   sink's own concern and leaves the backend available, so one transient hiccup
   does not retire it.
   
   @param entryIndex - Sink entry index whose backend is no longer usable.
   */
  function markEntryUnavailable({ entryIndex, }: { readonly entryIndex: number; },): void {
    /**
     Mutable sink entry being disabled.
     */
    const entry = getSinkEntry({ entryIndex, },);
    entry.available = false;
    recomputeAvailability();
    if (!entry.decided)
      concludeEntry({
        available: false,
        entryIndex,
      },);
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
   Replays records kept for a sink that just became available.
   
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
   Records one sink's verdict: its availability, the fact that it no longer
   needs records kept, the replay of those records when it answered available,
   and, once every sink has a verdict, the release of the kept records.
   
   @param entryIndex - Sink entry index whose verdict arrived.
   
   @param available - Whether the backend reported itself usable.
   */
  function concludeEntry(
    {
      entryIndex,
      available,
    }: {
      readonly entryIndex: number;
      readonly available: boolean;
    },
  ): void {
    /**
     Sink entry whose verdict is arriving.
     */
    const entry = getSinkEntry({ entryIndex, },);
    if (entry.decided)
      return;
    entry.decided = true;
    entry.available = available;
    state.undecidedEntries -= 1;
    recomputeAvailability();
    if (available)
      replayStartupRecordsToEntry({ entryIndex, },);
    releaseStartupRecords();
  }

  /**
   Frees the kept records and reports their overflow once every sink has a
   verdict. While one verdict is outstanding the records stay, so a probe that
   answers late still receives everything logged meanwhile.
   */
  function releaseStartupRecords(): void {
    if ((state.undecidedEntries > 0) || state.startupRecordsReleased)
      return;
    state.startupRecordsReleased = true;
    startupRecords.length = 0;
    emitDroppedStartupMarker();
  }

  /**
   Counts the records held for sinks that have no verdict yet, which a failing
   sink takes with it.

   @returns Number of records still kept.
   */
  function keptRecordCount(): number {
    return startupRecords.length;
  }

  /**
   Writes one synthetic `warn` record to every available sink when the
   startup buffer overflowed, so the loss is never silent. Runs once, when
   every sink has a verdict, because only then is the dropped count final.
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
   keep `initialized` from flipping. A verifier that runs past its limit keeps
   running and reports later; `initialized` only means startup stopped waiting.
   Completion order does not affect correctness: a record's immediate-write set
   (sinks already available when it was logged) and its replay set (sinks that
   become available later) are disjoint, so each available sink receives each
   record exactly once whichever verify settles first.
   */
  async function initialize(): Promise<void> {
    if (state.initialized)
      return;

    await watchSinkProbes({
      ms: verifyTimeoutMs,
      ports: {
        conclude: concludeEntry,
        keptCount: keptRecordCount,
        markUnavailable: markEntryUnavailable,
      },
      sinks: entries.map(function toSink(entry,) {
        return entry.sink;
      },),
    },);

    state.initialized = true;
    releaseStartupRecords();
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
      if (state.initialized
        && (state.undecidedEntries === 0)
        && (!state.hasAvailableSink))
        throw new Error('No logging backends available',);

      /**
       Shared LogRecord forwarded to every available sink; built once per call so all sinks see the same timestamp.
       */
      const record: LogRecord = {
        level,
        message,
        timestamp: Date.now(),
      };

      if (state.undecidedEntries > 0)
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

   The deadline arrives as the wait's outcome, so no rejection text decides how
   the failure is reported: a rejection here is the work failing on its own.
   */
  async function flushAll(): Promise<void> {
    try {
      /**
       Which side of the wait finished first: the work, or the deadline.
       */
      const outcome = await settleWithinHostTimeout({
        ms: flushDeadlineMs,
        promise: drainEverything(),
      },);

      if (outcome === 'settled')
        return;

      reportLoggerCondition({ message: `flush deadline of ${flushDeadlineMs}ms elapsed; abandoning in-flight sink work`, },);
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'flush failed before completion; abandoning in-flight sink work',
        error,
      },);
    }

    abandonPendingWrites();
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
