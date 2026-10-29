import { reportLoggerInternalError, } from '../error-format.ts';
import { createLocalStorageStore, } from './local-storage-store.ts';
import { createRecordBuffer, } from './record-buffer.ts';
import { detectWebStorageRuntime, } from './web-storage-runtime.ts';

import type {
  Level,
  Sink,
} from '../types.ts';

/**
 Node CLI flag that backs `localStorage` with a file; without it Node 26
 leaves `globalThis.localStorage` undefined and prints an
 `ExperimentalWarning` on stderr the moment the getter is touched.
 */
const NODE_LOCALSTORAGE_FLAG = '--localstorage-file';

/**
 Reports whether this is a flagless plain-Node process, where
 `globalThis.localStorage` is undefined and merely touching the getter
 prints Node's ExperimentalWarning on stderr for every consumer, so `verify`
 skips the probe without any access. The flag is honored both on the command
 line (`process.execArgv`) and through `NODE_OPTIONS` (verified on Node 26;
 `execArgv` does not echo `NODE_OPTIONS` flags, so both are checked). A DOM
 host (such as an Electron renderer, which carries `process.versions.node`
 alongside a real `localStorage`) is exempt and probes normally.
 
 @returns Whether the probe must be skipped because Node would only warn.
 */
function nodeWithoutLocalStorageFile(): boolean {
  if (detectWebStorageRuntime() !== 'node')
    return false;
  if ('document' in globalThis)
    return false;
  /**
   Whether the backing-file flag reached this process on the command line.
   */
  const flaggedInExecArgv = process.execArgv
    .some(function startsWithFlag(argument: string,) {
      return argument.startsWith(NODE_LOCALSTORAGE_FLAG,);
    },);
  /**
   Raw `NODE_OPTIONS` value, absent when the variable is unset; `execArgv`
   does not echo flags arriving through it, so it is scanned separately.
   */
  const nodeOptions = process.env
    .NODE_OPTIONS;
  /**
   Whether the backing-file flag reached this process through `NODE_OPTIONS`.
   */
  const flaggedInNodeOptions = (nodeOptions ?? '').includes(NODE_LOCALSTORAGE_FLAG,);
  return !(flaggedInExecArgv || flaggedInNodeOptions);
}

/**
 Verifies localStorage actually persists data. Stateless: the logger calls
 this once per sink at startup and owns the resulting availability, so no
 verified/available flag is kept here.
 
 Election is by probe alone: any runtime whose `localStorage` round-trips
 (browsers, Deno, Node launched with `--localstorage-file`) keeps the sink.
 The one short-circuit, {@link nodeWithoutLocalStorageFile}, returns the
 same `false` the probe would and exists only to keep Node's access warning
 off every consumer's stderr, not to gate by runtime brand.
 
 @returns Whether localStorage is available and round-trips a probe write.
 
 @example
 ```ts
 if (await verifyLocalStorage()) {
   // localStorage usable
 }
 ```
 */
function verifyLocalStorage(): Promise<boolean> {
  if (nodeWithoutLocalStorageFile())
    return Promise.resolve(false,);
  try {
    /**
     Sentinel key used only for the probe write/read; removed afterward to avoid polluting real log entries.
     */
    const testKey = '__monochromatic_verify__';
    /**
     Timestamp-based probe value so concurrent verifications never read each other's writes.
     */
    const testValue = `test-${Date.now()}`;
    globalThis.localStorage
      .setItem(
      testKey,
      testValue,
      );
    /**
     Probe value read back from storage; equality with `testValue` proves writes actually persist.
     */
    const readBack = globalThis.localStorage
      .getItem(testKey,);
    globalThis.localStorage
      .removeItem(testKey,);
    return Promise.resolve(readBack === testValue,);
  }
  catch (error: unknown) {
    if ('localStorage' in globalThis)
      reportLoggerInternalError({
        context: 'localStorage sink verification failed',
        error,
      },);
    return Promise.resolve(false,);
  }
}

/**
 Builds a localStorage sink that buffers serialized records through the
 shared {@link createRecordBuffer} policy and persists each newline-joined
 JSONL batch under a run-scoped counter-incremented key through
 {@link createLocalStorageStore}. One uniform write path runs on every
 runtime; no per-runtime mode exists. Flush triggers (32 KiB in-write cap,
 `warn`-or-worse severity, 250 ms quiet-period deadline, page lifecycle,
 and the `flush` hook) are the buffer's; see {@link createRecordBuffer}.
 
 Unlike the sessionStorage sink, whose store dies with the tab, this sink's
 batches survive tab close and browser restart, bounded by oldest-first
 eviction at half the localStorage quota; that makes it the web storage sink
 whose records remain inspectable after a full crash-and-restart.
 
 @returns Sink backed by web `localStorage`.
 
 @example
 ```ts
 const { logger } = createLogger({ sinks: [createLocalStorageSink()] });
 logger.info('user signed in'); // buffered
 logger.warn('quota near');     // flushes both records in one batch
 ```
 */
export function createLocalStorageSink(): Sink {
  /**
   Synchronous admission shares completion without changing buffer persistence or flush triggers.
   */
  const completedWrite = Promise.resolve();
  /**
   Persistence engine owning run identity, key allocation, prior-run
   adoption, footprint accounting, and quota eviction; the buffer decides
   when a batch is handed to it.
   */
  const store = createLocalStorageStore();

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
    verify: verifyLocalStorage,
    write,
  };
}
