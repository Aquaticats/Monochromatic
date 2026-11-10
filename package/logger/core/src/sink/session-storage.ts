import { reportLoggerInternalError, } from '../error-format.ts';
import { createRecordBuffer, } from './record-buffer.ts';
import { createSessionStorageStore, } from './session-storage-store.ts';
import { createWebStorageProbeKey, } from './web-storage-probe-key.ts';

import type {
  Level,
  Sink,
} from '../types.ts';

/**
 Verifies sessionStorage actually persists data. Stateless: the logger calls
 this once per sink at startup and owns the resulting availability, so no
 verified/available flag is kept here.
 
 Election is by probe alone: any runtime whose `sessionStorage` round-trips
 (browsers, Node 22+, Deno) keeps the sink, and the buffered write path
 keeps the per-record cost acceptable everywhere rather than a runtime brand
 check deciding who may log here.

 The probe names a key of its own through {@link createWebStorageProbeKey},
 the same shape the localStorage probe uses: a fixed key would let a second
 verification of the same store, in this tab or another document, overwrite or
 remove this one's value.

 @returns Whether sessionStorage is available and round-trips a probe write.

 @example
 ```ts
 if (await verifySessionStorage()) {
   // sessionStorage usable
 }
 ```
 */
function verifySessionStorage(): Promise<boolean> {
  try {
    /**
     Key this probe owns for its whole write-read-remove cycle; removed afterward to avoid polluting real log entries.
     */
    const testKey = createWebStorageProbeKey();
    /**
     Probe value unique to this write, so a store that echoes something else
     fails the equality check instead of electing the sink.
     */
    const testValue = `test-${Date.now()}`;
    globalThis.sessionStorage
      .setItem(
      testKey,
      testValue,
      );
    /**
     Probe value read back from storage; equality with `testValue` proves writes actually persist.
     */
    const readBack = globalThis.sessionStorage
      .getItem(testKey,);
    globalThis.sessionStorage
      .removeItem(testKey,);
    return Promise.resolve(readBack === testValue,);
  }
  catch (error: unknown) {
    if ('sessionStorage' in globalThis)
      reportLoggerInternalError({
        context: 'sessionStorage sink verification failed',
        error,
      },);
    return Promise.resolve(false,);
  }
}

/**
 Builds a sessionStorage sink that buffers serialized records through the
 shared {@link createRecordBuffer} policy and persists each newline-joined
 JSONL batch under a counter-incremented key through
 {@link createSessionStorageStore}. One uniform write path runs on every
 runtime; no per-runtime mode exists. Flush triggers (32 KiB in-write cap,
 `warn`-or-worse severity, 250 ms quiet-period deadline, page lifecycle,
 and the `flush` hook) are the buffer's; see {@link createRecordBuffer}.
 
 @returns Sink backed by web `sessionStorage`.
 
 @example
 ```ts
 const { logger } = createLogger({ sinks: [createSessionStorageSink()] });
 logger.info('user signed in'); // buffered
 logger.warn('quota near');     // flushes both records in one batch
 ```
 */
export function createSessionStorageSink(): Sink {
  /**
   Synchronous admission shares completion without changing buffer persistence or flush triggers.
   */
  const completedWrite = Promise.resolve();
  /**
   Persistence engine owning key allocation, footprint accounting, and quota
   eviction; the buffer decides when a batch is handed to it.
   */
  const store = createSessionStorageStore();

  /**
   Shared buffering stage; every flush trigger lands one joined batch in the
   persistence engine synchronously.
   */
  const buffer = createRecordBuffer({ onFlush: store.persist, },);

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
    return completedWrite;
  }

  /**
   Drains the buffer into the persistence engine; the drain is synchronous,
   so the batch has landed by the time the resolved promise is observed.
   */
  function flush(): Promise<void> {
    buffer.drain();
    return completedWrite;
  }

  return {
    flush,
    name: 'sessionStorage',
    verify: verifySessionStorage,
    write,
  };
}
