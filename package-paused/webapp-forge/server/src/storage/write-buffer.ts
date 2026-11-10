/**
 * Coalescing write buffer in front of a `Storage` adapter.
 *
 * The dispatcher fans many fragment renders out in parallel; without a
 * buffer in front of the adapter every render would issue its own
 * `put` against the (potentially network-bound) backend. The buffer
 * groups items into batches of up to `flushAtItems`, flushed every
 * `flushAtMs`, and dispatches them with a configurable concurrency
 * ceiling.
 *
 * Per-key ordering is preserved: when the same key is enqueued twice,
 * the second body wins; the first put is dropped before flush so the
 * adapter sees a single durable write per coalescing window.
 *
 * The buffer exposes an explicit `flush()` for tests and a `close()`
 * that flushes once and refuses further writes.
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core';

import type {
  Storage,
  StoragePutItem,
} from './adapter.ts';

/**
Tagged logger for batches the adapter could not write.
*/
const writeBufferLog = tagged({
  tag: 'write-buffer',
  l: logger,
},);

/**
Lists a batch's storage keys, so a failure report names what did not land.

@param items - batch the adapter rejected

@returns comma-separated keys in batch order
*/
function describeKeys(items: readonly StoragePutItem[],): string {
  return items
    .map(function keyOf(item,) {
      return item.key;
    },)
    .join(', ',);
}

/**
 * Default item count that triggers a flush.
 *
 * @example
 * ```ts
 * createWriteBuffer({ storage, flushAtItems: DEFAULT_FLUSH_AT_ITEMS });
 * ```
 */
const DEFAULT_FLUSH_AT_ITEMS = 256;

/**
 * Default ms-based flush trigger.
 *
 * @example
 * ```ts
 * createWriteBuffer({ storage, flushAtMs: DEFAULT_FLUSH_AT_MS });
 * ```
 */
const DEFAULT_FLUSH_AT_MS = 50;

/**
 * Default outstanding-batch concurrency ceiling for the adapter.
 *
 * @example
 * ```ts
 * createWriteBuffer({ storage, concurrency: DEFAULT_CONCURRENCY });
 * ```
 */
const DEFAULT_CONCURRENCY = 64;

/**
 * Tunable behaviour for {@link createWriteBuffer}.
 *
 * @example
 * ```ts
 * const buffer = createWriteBuffer({ storage, flushAtItems: 64, flushAtMs: 25 });
 * ```
 */
export type WriteBufferOptions = {
  /**
   * Flush when this many items are queued. Default 256.
   */
  readonly flushAtItems?: number;

  /**
   * Flush this many ms after the first queued item. Default 50.
   */
  readonly flushAtMs?: number;

  /**
   * Maximum simultaneous in-flight `putBatch` calls. Default 64.
   */
  readonly concurrency?: number;
};

/**
 * Named parameters for {@link createWriteBuffer}.
 *
 * @example
 * ```ts
 * const buffer = createWriteBuffer({ storage, flushAtItems: 64 });
 * ```
 */
export type CreateWriteBufferParams = {
  /**
   * Downstream storage adapter to buffer writes against.
   */
  readonly storage: Storage;

  /**
   * Flush when this many items are queued. Default 256.
   */
  readonly flushAtItems?: number;

  /**
   * Flush this many ms after the first queued item. Default 50.
   */
  readonly flushAtMs?: number;

  /**
   * Maximum simultaneous in-flight `putBatch` calls. Default 64.
   */
  readonly concurrency?: number;
};

/**
 * Public interface exposed by {@link createWriteBuffer}.
 *
 * @example
 * ```ts
 * const buffer: WriteBuffer = createWriteBuffer(storage);
 * buffer.enqueue({ key: 'a', body: bytes });
 * await buffer.close();
 * ```
 */
export type WriteBuffer = {
  /**
   * Enqueues a put. Coalesces against any pending item with the same key.
   *
   * @param item - put operation
   */
  enqueue(item: StoragePutItem,): void;

  /**
   * Forces a flush. Resolves once every batch issued by this call has
   * completed. Does not flush items added after the call begins.
   *
   * @throws Error when a batch could not be written: the key-naming error from
   * this call's own batch, or the adapter's error from another in-flight batch.
   * Failed items stay queued for the next `flush()`, `close()` or `enqueue`
   */
  flush(): Promise<void>;

  /**
   * Flushes one final time and refuses further `enqueue` calls.
   *
   * @throws Error when the final batch could not be written; those items stay
   * queued, so a later `flush()` retries them even though `enqueue` is closed
   */
  close(): Promise<void>;

  /**
   * Number of items currently queued (before the next flush).
   */
  readonly pending: number;
};

/**
 * Fire-and-forget helper that adapts a promise to the void-returning
 * call sites. A rejection is logged here because no caller awaits it; the
 * failed batch's items stay queued, so an explicit `flush()` retries them.
 *
 * @param promise - promise to detach from the current call
 *
 * @example
 * ```ts
 * detach(storage.putBatch(items));
 * ```
 */
function detach(promise: Promise<unknown>,): void {
  void (async function logRejection(): Promise<void> {
    try {
      await promise;
    }
    catch (error) {
      writeBufferLog.error(`detached write buffer flush failed: ${caughtValueText(error,)}`,);
    }
  }());
}

/**
 * Creates a write buffer in front of a storage adapter.
 *
 * @param storage - downstream storage adapter (memory, S3-compatible, etc.)
 *
 * @param flushAtItems - item-count threshold that triggers a flush (default 256)
 *
 * @param flushAtMs - ms-based flush trigger (default 50)
 *
 * @param concurrency - maximum simultaneous in-flight `putBatch` calls (default 64)
 *
 * @returns write buffer instance
 *
 * @example
 * ```ts
 * const buffer = createWriteBuffer({
 *   storage,
 *   flushAtItems: 64,
 *   flushAtMs: 25,
 * });
 * buffer.enqueue({ key: 'fragments/a', body: bytes });
 * await buffer.close();
 * ```
 */
export function createWriteBuffer({
  storage,
  flushAtItems = DEFAULT_FLUSH_AT_ITEMS,
  flushAtMs = DEFAULT_FLUSH_AT_MS,
  concurrency = DEFAULT_CONCURRENCY,
}: CreateWriteBufferParams,): WriteBuffer {
  /**
   * Pending items keyed by storage key (later wins).
   */
  const queue = new Map<string, StoragePutItem>();

  /**
   * Enqueue generation per key, bumped by every `enqueue`. A failed batch
   * restores an item only while its key still carries the generation that batch
   * captured, so a body enqueued and snapshotted again while the older batch was
   * in flight never comes back. Entries are dropped once their body is
   * delivered, which keeps the map bounded by the keys in flight or queued.
   */
  const generations = new Map<string, number>();

  /**
   * In-flight flush promises bounded by `concurrency`.
   */
  const inFlight = new Set<Promise<void>>();

  /**
   * Mutable per-instance state captured by the closures below.
   */
  const state: {
    closed: boolean;
    timerId: ReturnType<typeof setTimeout> | null;
  } = {
    closed: false,
    timerId: null,
  };

  /**
   * Puts a rejected batch's items back into the queue so the next flush
   * retries them. A key enqueued again while the batch was in flight keeps its
   * newer body, whether that newer body is still queued or already delivered by
   * another batch, because the later write is the one the caller asked for.
   *
   * @param row - rejected items and the generations captured with them
   *
   * @example
   * ```ts
   * requeueFailedBatch({ items, capturedGenerations });
   * ```
   */
  function requeueFailedBatch(row: {
    readonly items: readonly StoragePutItem[];
    readonly capturedGenerations: ReadonlyMap<string, number>;
  },): void {
    /**
     * Generations this batch captured, read through a local so the lookups stay
     * one member access deep.
     */
    const { capturedGenerations, } = row;
    for (const item of row.items) {
      /**
       * A key whose generation moved on was enqueued again after this batch
       * captured it, so its newer body wins and the older one is dropped.
       */
      if (capturedGenerations.get(item.key,)
        !== generations.get(item.key,))
        continue;
      if (!queue.has(item.key,))
        queue.set(
          item.key,
          item,
        );
    }
  }

  /**
   * Drops the generation entries of a delivered batch, so the map holds only
   * keys whose body is still queued or in flight.
   *
   * @param row - delivered items and the generations captured with them
   *
   * @example
   * ```ts
   * forgetDeliveredGenerations({ items, capturedGenerations });
   * ```
   */
  function forgetDeliveredGenerations(row: {
    readonly items: readonly StoragePutItem[];
    readonly capturedGenerations: ReadonlyMap<string, number>;
  },): void {
    /**
     * Generations this batch captured, read through a local so the lookups stay
     * one member access deep.
     */
    const { capturedGenerations, } = row;
    for (const item of row.items) {
      if (capturedGenerations.get(item.key,)
        === generations.get(item.key,))
        generations.delete(item.key,);
    }
  }

  /**
   * Cancels any pending time-based flush trigger.
   *
   * @example
   * ```ts
   * clearTimer();
   * ```
   */
  function clearTimer(): void {
    if (state.timerId
      !== null) {
      clearTimeout(state.timerId,);
      state.timerId = null;
    }
  }

  /**
   * Drains the queue into a single `putBatch` call against the
   * adapter. Awaits the put before resolving so the next caller sees
   * a settled state. A rejected batch goes back into the queue, because the
   * snapshot taken here is the only remaining handle on those items, and
   * nothing re-arms the timer for it: the retry happens on the next `enqueue`,
   * `flush()` or `close()` rather than on a timer of its own, so a backend that
   * stays down is not hammered.
   *
   * @throws Error naming the batch's keys when the adapter rejected it
   *
   * @example
   * ```ts
   * await flushOnce();
   * ```
   */
  async function flushOnce(): Promise<void> {
    clearTimer();
    if (queue.size
      === 0)
      return;
    /**
     * Snapshot of pending items and their generations; re-entry from `enqueue`
     * repopulates `queue` with newer bodies.
     */
    const snapshot = [...queue.entries(),];
    /**
     * Put items to send, in snapshot order.
     */
    const items: StoragePutItem[] = snapshot.map(function itemOf(entry,) {
      return entry[1];
    },);
    /**
     * Enqueue generation captured for each key, so the requeue below can tell
     * whether the key moved on while this batch was in flight.
     */
    const capturedGenerations = new Map(snapshot.map(function generationOf(entry,) {
      /**
       * Generation the key carried when this batch snapshotted it.
       */
      const generation = generations.get(entry[0],)
        ?? 0;
      return [
        entry[0],
        generation,
      ] as const;
    },),);
    queue.clear();
    // Bound concurrency: when we have already saturated the in-flight
    // batches, await one before kicking off the next.
    while (inFlight.size
      >= concurrency) {
      // oxlint-disable-next-line no-await-in-loop -- explicit serialisation under saturation
      await Promise.race(inFlight,);
    }
    /**
     * In-flight putBatch promise tracked so close() can await every flush.
     */
    const promise = storage.putBatch(items,);
    inFlight.add(promise,);
    detach(async function trackInFlight(): Promise<void> {
      /**
       * RAII disposer removes `promise` from the in-flight set on settle.
       */
      using _disposeOnSettle = {
        [Symbol.dispose](): void {
          inFlight.delete(promise,);
        },
      };
      void _disposeOnSettle;
      await promise;
    }(),);
    try {
      await promise;
      forgetDeliveredGenerations({
        items,
        capturedGenerations,
      },);
    }
    catch (error) {
      requeueFailedBatch({
        items,
        capturedGenerations,
      },);
      writeBufferLog.error(
        `putBatch failed for ${describeKeys(items,)}; re-queued for the next flush: ${
          caughtValueText(error,)
        }`,
      );
      throw new Error(
        `write buffer: putBatch failed for ${describeKeys(items,)}: ${caughtValueText(error,)}`,
        { cause: error, },
      );
    }
  }

  /**
   * Schedules a time-based flush trigger if one is not already active.
   *
   * @example
   * ```ts
   * scheduleTimer();
   * ```
   */
  function scheduleTimer(): void {
    if (state.timerId
      !== null)
      return;
    state.timerId = setTimeout(
      function timerFlush() {
        state.timerId = null;
        // Fire-and-forget: any consumer that needs to await the flush
        // should call `flush()` explicitly.
        detach(flushOnce(),);
      },
      flushAtMs,
    );
  }

  return {
    enqueue(item: StoragePutItem,): void {
      if (state.closed)
        throw new Error('write buffer is closed',);
      queue.set(
        item.key,
        item,
      );
      generations.set(
        item.key,
        (generations.get(item.key,)
          ?? 0) + 1,
      );
      if (queue.size
        >= flushAtItems) {
        // Trigger the size-based flush; `flushOnce` clears the timer.
        detach(flushOnce(),);
        return;
      }
      scheduleTimer();
    },
    async flush(): Promise<void> {
      await flushOnce();
      await Promise.all(inFlight,);
    },
    async close(): Promise<void> {
      state.closed = true;
      clearTimer();
      await flushOnce();
      await Promise.all(inFlight,);
    },
    /**
     * Number of items currently queued (before the next flush).
     */
    get pending(): number {
      return queue.size;
    },
  };
}
