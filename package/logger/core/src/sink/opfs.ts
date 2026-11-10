import { reportLoggerInternalError, } from '../error-format.ts';
import { createRecordBuffer, } from './record-buffer.ts';

import type {
  Level,
  Sink,
} from '../types.ts';

/**
 Suffix of the probe file this sink's verification owns.
 The file never carries log records,
 so a crash between its creation and removal leaves sweepable residue,
 never a fake first record in a log a reader parses line by line.
 */
const PROBE_FILE_SUFFIX = '.probe.tmp';

/**
 Radix of the probe file's random component,
 so `Number.prototype.toString` yields compact alphanumerics.
 */
const PROBE_NONCE_RADIX = 36;

/**
 Length of the probe file's random component, which keeps two verifications
 started in the same millisecond on different probe files.
 */
const PROBE_NONCE_LENGTH = 4;

/**
 Builds an OPFS sink that buffers serialized records through the shared
 {@link createRecordBuffer} policy and appends each newline-joined JSONL
 batch to a per-session file in the Origin Private File System with one
 stream write per batch. The kept-open writable stream lives in this
 instance's closure (no module-global state), so independent loggers and
 tests never share a handle or need a reset hook.
 
 Flush triggers (32 KiB in-write cap, `warn`-or-worse severity, 250 ms
 quiet-period deadline, page lifecycle, and the `flush` hook) are the
 buffer's; see {@link createRecordBuffer}. Batch writes queue on the stream
 in issue order, so ordering holds at the batch boundary, and the sink's
 `flush` hook awaits every issued batch before resolving.
 
 @returns Sink backed by OPFS.
 
 @example
 ```ts
 const { logger } = createLogger({ sinks: [createOpfsSink()] });
 logger.warn('quota nearing limit');
 ```
 */
export function createOpfsSink(): Sink {
  /**
   Instance-local kept-open OPFS stream, opened by `verify` and reused by
   every batch write. Absent until a successful verification.
   */
  const state: { writable?: FileSystemWritableFileStream; } = {};

  /**
   Batch writes issued to the stream and not yet settled; the `flush` hook
   drains this so logger-level `flush()` observes every issued batch.
   */
  const pendingBatchWrites = new Set<Promise<void>>();

  /**
   Verifies OPFS is available by round-tripping a probe file it removes, then
   opens the stream reused by subsequent writes. The probe never touches this
   run's log file, so that file holds nothing but JSONL records. The logger
   calls this once and owns the resulting availability.
   
   @returns Whether OPFS logging is available.
   */
  async function verify(): Promise<boolean> {
    try {
      /**
       Origin Private File System directory handle that hosts every monochromatic log file.
       */
      const opfsRoot = await navigator.storage
        .getDirectory();
      /**
       ISO timestamp with colons replaced by dashes so it can be embedded in a cross-platform file name.
       */
      const timestamp = new Date().toISOString()
        .replaceAll(
        ':',
        '-',
        );
      /**
       Random component keeping two verifications started in the same millisecond on different probe files.
       */
      const probeNonce = Math.random()
        .toString(PROBE_NONCE_RADIX,)
        .slice(
        2,
        2 + PROBE_NONCE_LENGTH,
        )
        .padEnd(
        PROBE_NONCE_LENGTH,
        '0',
        );
      /**
       Probe file this verification owns, named outside the log family so it is never read as a record.
       */
      const probeFileName = `monochromatic-${timestamp}.${probeNonce}${PROBE_FILE_SUFFIX}`;
      /**
       OPFS handle for that probe file, created for this verification alone.
       */
      const probeHandle = await opfsRoot.getFileHandle(
        probeFileName,
        { create: true, },
      );
      /**
       Removes the probe file on every exit path, including a mismatch throw or a close failure.
       A cleanup failure is reported rather than thrown, because residue in the
       origin private file system says nothing about whether logging works.
       */
      await using _probeCleanup = {
        async [Symbol.asyncDispose](): Promise<void> {
          try {
            await opfsRoot.removeEntry(probeFileName,);
          }
          catch (error: unknown) {
            reportLoggerInternalError({
              context: 'OPFS sink probe file cleanup failed',
              error,
            },);
          }
        },
      };
      // Write probe data and close to flush; getFile() reads stale content
      // while a FileSystemWritableFileStream is still open.
      /**
       Throwaway writable used only to flush the probe so the next `getFile` returns persisted content.
       */
      const probeWritable = await probeHandle.createWritable();
      /**
       Probe record written and read back to confirm OPFS round-trips writes.
       */
      const testData = `{"test":true,"timestamp":${Date.now()}}\n`;
      await probeWritable.write(testData,);
      await probeWritable.close();

      /**
       File snapshot of the probe, taken after closing `probeWritable` so its bytes are flushed.
       */
      const probeFile = await probeHandle.getFile();
      /**
       Probe contents read back; matching the literal `"test":true` proves OPFS persisted the data.
       */
      const content = await probeFile.text();
      if (!content.includes('"test":true',))
        return false;

      /**
       OPFS handle for the per-run log file, created once verification proved writes persist.
       */
      const fileHandle = await opfsRoot.getFileHandle(
        `monochromatic-${timestamp}.log.jsonl`,
        { create: true, },
      );
      state.writable = await fileHandle.createWritable({ keepExistingData: true, },);
      return true;
    }
    catch (error: unknown) {
      /**
       OPFS storage object, present only when the current platform exposes the backend this sink verifies.
       */
      const opfsStorage = globalThis.navigator
        ?.storage;
      if ((opfsStorage !== undefined) && ('getDirectory' in opfsStorage))
        reportLoggerInternalError({
          context: 'OPFS sink verification failed',
          error,
        },);
      return false;
    }
  }

  /**
   Writes one newline-terminated batch to the OPFS stream, swallowing and
   reporting failures so the pending-write set always settles.
   
   @param batch - Newline-joined JSONL batch from the buffer.
   */
  async function writeBatch(batch: string,): Promise<void> {
    if (!state.writable)
      return;

    try {
      await state.writable
        .write(`${batch}\n`,);
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'OPFS sink record write failed',
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
   Backend handoff for the buffer: issues the batch write without awaiting
   (stream writes queue in issue order) and tracks it for the `flush` hook.
   
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
   Shared buffering stage; every flush trigger issues one queued stream
   write per joined batch.
   */
  const buffer = createRecordBuffer({ onFlush: handOffBatch, },);

  /**
   Buffers a log record through the shared policy; see
   {@link createRecordBuffer} for the flush triggers.
   
   @param record - Log record to buffer and eventually append.
   
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
   Drains the buffer onto the stream and resolves once every issued batch
   write has settled.
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
    name: 'OPFS',
    verify,
    write,
  };
}
