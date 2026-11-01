import { reportLoggerInternalError, } from '../error-format.ts';
import {
  awaitRequest,
  awaitTransaction,
} from './indexed-db-util.ts';
import { createRecordBuffer, } from './record-buffer.ts';

import type {
  Level,
  Sink,
} from '../types.ts';

/**
 Database holding this logger's batches, one per origin, shared by every tab.
 */
const DATABASE_NAME = 'monochromatic.log';

/**
 Schema version; bump only with an upgrade path in `onupgradeneeded`.
 */
const DATABASE_VERSION = 1;

/**
 Object store holding one newline-joined JSONL batch string per
 auto-incremented key, so key order is arrival order across every tab and
 retention can trim oldest-first without any run bookkeeping.
 */
const BATCH_STORE = 'batch';

/**
 Retention cap on stored batches, trimmed oldest-first inside each persist
 transaction. At the buffer's 32 KiB flush cap this bounds the store near
 64 MiB, well under the multi-gigabyte origin quota
 (`navigator.storage.estimate()` reported 10 GiB on the measuring machine)
 while months of sessions still fit. A count cap instead of a byte tally
 because severity-flushed batches vary in size and an exact byte budget
 would need a cross-session tally re-summed at startup; the bound is
 approximate by design.
 */
const MAX_STORED_BATCHES = 2_048;

/**
 Opens (creating on first use) the logger's IndexedDB database with the
 batch store ready.
 
 @returns Open database connection.
 
 @throws DOMException - When the backend refuses to open, for example in a
 storage-partitioned context that denies IndexedDB.
 */
async function openLogDatabase(): Promise<IDBDatabase> {
  /**
   Open request; the upgrade handler runs only when the database is new or
   below {@link DATABASE_VERSION}.
   */
  const request = globalThis.indexedDB
    .open(
    DATABASE_NAME,
    DATABASE_VERSION,
    );
  request.onupgradeneeded = function createBatchStore(): void {
    request.result
      .createObjectStore(
      BATCH_STORE,
      { autoIncrement: true, },
      );
  };
  return await awaitRequest(request,);
}

/**
 Persists one batch and trims the store back under the retention cap, all
 inside one readwrite transaction so a crash between the steps cannot leave
 the trim half-applied.
 
 @param database - Open connection from {@link openLogDatabase}.
 
 @param batch - Newline-joined JSONL batch string to persist.
 
 @throws DOMException - When the transaction errors or aborts, for example
 under an origin-quota overflow.
 
 @mutates database - `database.transaction` opens a readwrite transaction,
 registering live state on the host-owned connection, and the queued add
 and trim change the store that connection controls.
 */
async function persistBatch(
  {
    database,
    batch,
  }: {
    readonly database: IDBDatabase;
    readonly batch: string;
  },
): Promise<void> {
  /**
   Single transaction carrying the add, the count, and any trim.
   */
  const transaction = database.transaction(
    BATCH_STORE,
    'readwrite',
  );
  /**
   Batch store within this transaction.
   */
  const store = transaction.objectStore(BATCH_STORE,);
  store.add(batch,);
  /**
   Stored batch count including the add queued in this transaction.
   */
  const count = await awaitRequest(store.count(),);
  if (count > MAX_STORED_BATCHES) {
    /**
     Oldest keys past the cap; `getAllKeys` returns keys in ascending order,
     which for an auto-incremented store is arrival order.
     */
    const staleKeys = await awaitRequest(store.getAllKeys(
      null,
      count - MAX_STORED_BATCHES,
    ),);
    /**
     Newest key still to be trimmed; everything at or below it goes.
     */
    const newestStale = staleKeys.at(-1,);
    if (newestStale !== undefined)
      store.delete(IDBKeyRange.upperBound(newestStale,),);
  }
  await awaitTransaction(transaction,);
}

/**
 Builds an IndexedDB sink that buffers serialized records through the shared
 {@link createRecordBuffer} policy and persists each newline-joined JSONL
 batch as one string value per transaction, measured at 0.15 µs of
 main-thread enqueue per record on headless Chromium 149 (one `add` per
 32 KiB batch). The connection lives in this instance's closure (no
 module-global state), so independent loggers and tests never share a
 handle or need a reset hook.
 
 Records are readable the moment their transaction settles (DevTools
 Application tab included), survive tab close and browser restart, and
 auto-incremented keys serialize across tabs, so no run-scoped naming is
 needed. Retention trims oldest-first past {@link MAX_STORED_BATCHES}.
 Transactions use the default relaxed durability: relaxed commits reach the
 browser's storage backend promptly and survive renderer crashes, and the
 OS-crash window `durability: 'strict'` would close is the rarest failure
 class, not worth an fsync per batch.
 
 Flush triggers (32 KiB in-write cap, `warn`-or-worse severity, 250 ms
 quiet-period deadline, page lifecycle, and the `flush` hook) are the
 buffer's; see {@link createRecordBuffer}. The sink's `flush` hook awaits
 every issued batch transaction before resolving.
 
 @returns Sink backed by IndexedDB.
 
 @example
 ```ts
 const { logger } = createLogger({ sinks: [createIndexedDbSink()] });
 logger.warn('quota nearing limit');
 ```
 */
export function createIndexedDbSink(): Sink {
  /**
   Instance-local open connection, set by `verify` and reused by every batch
   write. Absent until a successful verification.
   */
  const state: { database?: IDBDatabase; } = {};

  /**
   Batch transactions issued and not yet settled; the `flush` hook drains
   this so logger-level `flush()` observes every issued batch.
   */
  const pendingBatchWrites = new Set<Promise<void>>();

  /**
   Verifies IndexedDB is available and round-trips a probe value, keeping
   the opened connection for subsequent writes. The logger calls this once
   and owns the resulting availability.
   
   @returns Whether IndexedDB logging is available.
   */
  async function verify(): Promise<boolean> {
    try {
      if ((typeof globalThis.indexedDB) === 'undefined')
        return false;
      /**
       Connection kept for the sink's lifetime once the probe passes.
       */
      const database = await openLogDatabase();
      /**
       Probe transaction: add, read back, and remove one sentinel value.
       */
      const transaction = database.transaction(
        BATCH_STORE,
        'readwrite',
      );
      /**
       Batch store within the probe transaction.
       */
      const store = transaction.objectStore(BATCH_STORE,);
      /**
       Timestamp-based probe value so concurrent verifications never read each other's writes.
       */
      const probeValue = `probe-${Date.now()}`;
      /**
       Key the store assigned to the probe, used to read it back and remove it.
       */
      const probeKey = await awaitRequest(store.add(probeValue,),);
      /**
       Probe value read back; equality proves the backend round-trips writes.
       */
      const readBack = await awaitRequest(store.get(probeKey,) as IDBRequest<unknown>,);
      store.delete(probeKey,);
      await awaitTransaction(transaction,);

      if (readBack !== probeValue)
        return false;
      state.database = database;
      return true;
    }
    catch (error: unknown) {
      if ('indexedDB' in globalThis)
        reportLoggerInternalError({
          context: 'IndexedDB sink verification failed',
          error,
        },);
      return false;
    }
  }

  /**
   Writes one batch through {@link persistBatch}, swallowing and reporting
   failures so the pending-write set always settles.
   
   @param batch - Newline-joined JSONL batch from the buffer.
   */
  async function writeBatch(batch: string,): Promise<void> {
    if (!state.database)
      return;

    try {
      await persistBatch({
        database: state.database,
        batch,
      },);
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'IndexedDB sink record write failed',
        error,
      },);
    }
  }

  /**
   Removes a tracked batch write from {@link pendingBatchWrites} once it
   settles.
   
   @param pending - Promise returned by {@link writeBatch}.
   */
  async function removePendingWhenSettled(pending: Promise<void>,): Promise<void> {
    await pending;
    pendingBatchWrites.delete(pending,);
  }

  /**
   Backend handoff for the buffer: issues the batch transaction without
   awaiting and tracks it for the `flush` hook.
   
   @param batch - Newline-joined JSONL batch from the buffer.
   */
  function handOffBatch(batch: string,): void {
    /**
     In-flight batch write; never rejects, because {@link writeBatch} reports internally.
     */
    const pending = writeBatch(batch,);
    pendingBatchWrites.add(pending,);
    void removePendingWhenSettled(pending,);
  }

  /**
   Shared buffering stage; every flush trigger issues one batch transaction.
   */
  const buffer = createRecordBuffer({ onFlush: handOffBatch, },);

  /**
   Buffers a log record through the shared policy; see
   {@link createRecordBuffer} for the flush triggers.
   
   @param record - Log record to buffer and eventually persist.
   
   @mutates record - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
   */
  function write(record: {
    level: Level;
    message: string;
    timestamp: number;
  },): Promise<void> {
    buffer.add({
      level: record.level,
      serialized: JSON.stringify(record,),
    },);
    return Promise.resolve();
  }

  /**
   Drains the buffer into the store and resolves once every issued batch
   transaction has settled.
   */
  async function flush(): Promise<void> {
    buffer.drain();
    /**
     Snapshot of in-flight batch writes at drain time.
     */
    const writes = [...pendingBatchWrites,];
    await Promise.all(writes,);
  }

  return {
    flush,
    verify,
    write,
  };
}
