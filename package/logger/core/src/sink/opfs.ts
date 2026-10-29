import { reportLoggerInternalError, } from '../error-format.ts';
import { createRecordBuffer, } from './record-buffer.ts';

import type {
  Level,
  Sink,
} from '../types.ts';

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
   Verifies OPFS is available and round-trips a probe write, then opens the
   stream reused by subsequent writes. The logger calls this once and owns
   the resulting availability.
   
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
       OPFS handle for the per-run log file, created on first verification and reused for subsequent writes.
       */
      const fileHandle = await opfsRoot.getFileHandle(
        `monochromatic-${timestamp}.log.jsonl`,
        { create: true, },
      );
      // Write test data and close to flush; getFile() reads stale content
      // while a FileSystemWritableFileStream is still open.
      /**
       Throwaway writable used only to flush the probe so the next `getFile` returns persisted content.
       */
      const probeWritable = await fileHandle.createWritable({ keepExistingData: true, },);
      /**
       Probe record written and read back to confirm OPFS round-trips writes.
       */
      const testData = `{"test":true,"timestamp":${Date.now()}}\n`;
      await probeWritable.write(testData,);
      await probeWritable.close();

      /**
       File snapshot of the probe, taken after closing `probeWritable` so its bytes are flushed.
       */
      const file = await fileHandle.getFile();
      /**
       Probe contents read back; matching the literal `"test":true` proves OPFS persisted the data.
       */
      const content = await file.text();
      /**
       Whether the probe round-tripped; only then is the reused stream opened.
       */
      const available = content.includes('"test":true',);

      if (available)
        // Reopen for subsequent log writes.
        state.writable = await fileHandle.createWritable({ keepExistingData: true, },);

      return available;
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
    verify,
    write,
  };
}
