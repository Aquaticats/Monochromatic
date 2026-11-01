import { appendFile, } from 'node:fs/promises';
import { reportLoggerInternalError, } from '../error-format.ts';

//region One completion covers each bounded batch of complete JSONL records

/**
 Aggregation stays below one large engine string while an individual record remains indivisible.
 */
const MAX_BATCH_BYTES = 65_536;

/**
 Queue nodes own captured strings and one completion, rather than a promise chain per record.
 */
type AppendBatch = {
  readonly lines: string[];
  bytes: number;
  readonly completion: Promise<void>;
  readonly resolve: () => void;
  next?: AppendBatch;
};

/**
 Captured records share this queue's append order and each batch's complete append.
 */
type FileAppendQueue = {
  readonly append: (line: string,) => Promise<void>;
};

/**
 Batch synchronous producer frames without dropping records or overlapping complete append operations.

 @param filePath - verified destination owned by one sink

 @param ensureFile - one-time file creation ahead of first append

 @returns Queue preserving captured record order until append completion

 @example
 ```ts
 const queue = createFileAppendQueue({ filePath, ensureFile });
 await queue.append(`${JSON.stringify(record)}\n`);
 ```
 */
export function createFileAppendQueue({
  filePath,
  ensureFile,
}: {
  readonly filePath: string;
  readonly ensureFile: () => Promise<void>;
},): FileAppendQueue {
  /**
   Writing removes a batch from the queue before awaiting, so new records cannot enter an issued append.
   */
  const state: {
    opening?: Promise<void>;
    running: boolean;
    queue?: {
      head: AppendBatch;
      tail: AppendBatch
    };
  } = { running: false, };

  /**
   Append completion includes every filesystem chunk, preserving the existing write-error reporting behavior.

   @param batch - captured records detached from further producer admission

   @returns Completed batch whose callers may now be released
   */
  async function appendBatch(batch: AppendBatch,): Promise<AppendBatch> {
    try {
      /**
       One-time file creation, shared by every batch of this queue.
       */
      state.opening ??= ensureFile();
      await state.opening;
      await appendFile(
        filePath,
        batch.lines
          .join('',),
      );
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'file sink batch append failed',
        error,
      },);
    }
    return batch;
  }

  /**
   Async iteration starts each append only after the prior yielded append has completed.

   @returns Sequential append results until the current queue becomes empty
   */
  async function* queuedAppends(): AsyncGenerator<AppendBatch> {
    while (state.queue !== undefined) {
      /**
       Detached batch remains owned by this drain until its complete append settles.
       */
      const batch = state.queue
        .head;
      if (batch.next === undefined)
        delete state.queue;
      else
        state.queue
          .head = batch.next;
      yield appendBatch(batch,);
    }
    // Release admission before the final async iterator step can yield to a new producer.
    state.running = false;
  }

  /**
   One consumer resolves each batch only after its complete serial append.
   */
  async function drain(): Promise<void> {
    for await (const batch of queuedAppends())
      batch.resolve();
  }

  /**
   Microtask admission lets one synchronous producer frame fill batches before filesystem work starts.
   */
  function beginDrain(): void {
    void drain();
  }

  /**
   Capture one record into the pending tail or a new bounded batch, returning that batch's completion.

   @param line - serialized record captured before queue admission
   */
  function append(line: string,): Promise<void> {
    /**
     Byte bound applies to emitted UTF-8 rather than JavaScript code units.
     */
    const bytes = Buffer.byteLength(line,);
    /**
     Only queued batches remain writable; an issued batch was detached before its filesystem await.
     */
    const tail = state.queue
      ?.tail;
    if ((tail !== undefined) && ((tail.bytes + bytes) <= MAX_BATCH_BYTES)) {
      tail.lines
        .push(line,);
      tail.bytes += bytes;
      return tail.completion;
    }
    /**
     One deferred completion serves every record admitted to this batch.
     */
    const completion = Promise.withResolvers<void>();
    /**
     An oversized individual record stays whole and is never combined with another record.
     */
    const batch: AppendBatch = {
      lines: [line,],
      bytes,
      completion: completion.promise,
      resolve: completion.resolve,
    };
    if (state.queue === undefined)
      state.queue = {
        head: batch,
        tail: batch,
      };
    else {
      state.queue
        .tail
        .next = batch;
      state.queue
        .tail = batch;
    }
    if (!state.running) {
      state.running = true;
      queueMicrotask(beginDrain,);
    }
    return batch.completion;
  }

  return { append, };
}

//endregion One completion covers each bounded batch of complete JSONL records
