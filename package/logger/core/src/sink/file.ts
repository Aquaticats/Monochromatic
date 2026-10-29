import { resolveDefaultLogDirectory, } from './file-destination.ts';
import {
  createLogFile,
  newLogFilePath,
  probeLogDirectory,
} from './file-open.ts';
import {
  logRetentionPolicyFromEnv,
  NO_LOG_RETENTION_SWEEP,
  type LogRetentionPolicy,
} from './file-retention-policy.ts';
import { sweepLogDirectory, } from './file-retention-sweep.ts';

import { reportLoggerInternalError, } from '../error-format.ts';

import type { Sink, } from '../types.ts';
import { createFileAppendQueue, } from './file-append.ts';

/**
 Object serialization completed without producing a JSON value suitable for a JSONL record.
 */
class FileRecordSerializationError extends Error {
  /**
   Identify the serializer boundary without retaining the rejected object.
   */
  constructor() {
    super('File sink record serialization produced no JSON value',);
    this.name = 'FileRecordSerializationError';
  }
}

//region File sink
// Append-only JSONL sink. Imports `node:fs/promises` statically: this module
// is reachable only from the Node artifact (the `./node` subpath and the
// `#default-sinks` `node` condition), so the `node` export condition already
// asserts a Node runtime and no `process.versions.node` probe is needed.

/**
 Completion shared by every unverified or rejected write,
 since a sink without a verified destination admits no records.
 */
const completedWrite: Promise<void> = Promise.resolve();

/**
 Create the sink-owned file's append queue,
 with the file itself created before its first append so unused processes
 leave no files.

 @param destination - verified destination directory

 @returns Append queue owning this sink's single log file.
 */
function openAppends(destination: string,): ReturnType<typeof createFileAppendQueue> {
  /**
   Sink-owned log file this queue creates before its first append.
   */
  const filePath = newLogFilePath(destination,);
  return createFileAppendQueue({
    filePath,
    ensureFile: async function createFileOnFirstAppend(): Promise<void> {
      await createLogFile(filePath,);
    },
  });
}

/**
 Builds a file sink that appends JSONL records to a caller-selected directory,
 `MONOCHROMATIC_LOG_DIRECTORY`, or the nearest ancestor
 `node_modules/.monochromatic`, with user-state fallback (resolved once during
 verification). Verification probes the destination with a temp file it
 removes; the sink's own `{YYYY-MM-DD}/{timestamp}.{uuid}.log.jsonl` file is
 created on the first record, so a zero-record process leaves zero files.
 `MONOCHROMATIC_LOG_FILE=off` disables the sink entirely,
 even with an explicit destination.
 At verification the sink also sweeps its destination down to the retention
 window (14 days, 512 MiB, 50,000 files by default;
 `MONOCHROMATIC_LOG_MAX_AGE_DAYS`,
 `MONOCHROMATIC_LOG_MAX_BYTES`,
 and `MONOCHROMATIC_LOG_MAX_FILES` override),
 so unbounded accumulation stays bounded without a cleanup tool.
 The resolved directory and the verification memo live in this
 instance's closure (no module-global state), so independent loggers and
 tests retain their own append ordering. Each `write` returns its batch's
 complete append, so the logger's pending-write tracking also
 covers queued records; `flush` additionally settles the retention sweep.

 @param environment - runtime environment read for the kill switch and destination

 @param directory - explicit destination keeps caller-owned diagnostics outside workspace artifacts

 @param retention - retention bounds for this sink, or {@link NO_LOG_RETENTION_SWEEP} to keep the directory unswept

 @returns Sink backed by `node:fs/promises`.

 @example
 ```ts
 const { logger } = createLogger({ sinks: [createFileSink()] });
 logger.error('unhandled rejection');
 await logger.flush();
 ```
 */
export function createFileSink({
  environment = process.env,
  directory: configuredDirectory = environment.MONOCHROMATIC_LOG_DIRECTORY,
  retention,
}: Readonly<{
  environment?: Readonly<NodeJS.ProcessEnv>;
  directory?: string;
  retention?: LogRetentionPolicy | typeof NO_LOG_RETENTION_SWEEP;
}> = {}): Sink {
  /**
   Empty destinations preserve ancestor discovery; other supplied paths remain literal.
   */
  const explicitDirectory = configuredDirectory === '' ? undefined : configuredDirectory;
  /**
   The documented off switch,
   checked first so a disabled sink never touches the filesystem.
   */
  const disabled = environment.MONOCHROMATIC_LOG_FILE === 'off';
  /**
   Retention bounds for this sink;
   {@link NO_LOG_RETENTION_SWEEP} keeps caller-owned directories unswept.
   */
  const policy = retention ?? logRetentionPolicyFromEnv({ environment, },);
  /**
   Instance-local file-sink resources. `opened` is populated with the first
   record; `verifyPromise` memoizes concurrent verification so a caller
   arriving mid-flight shares the same async work rather than starting a
   second probe.
   */
  const state: {
    directory?: string;
    opened?: ReturnType<typeof openAppends>;
    verifyPromise?: Promise<boolean>;
    sweep?: Promise<void>;
  } = {};

  /**
   Append queue of this sink's lazily created file,
   shared by every write so one process keeps one file.

   @param destination - verified destination directory

   @returns Append queue, created on first use.
   */
  function appendsQueue(destination: string,): ReturnType<typeof openAppends> {
    /**
     Queue already created for this sink, shared by every write.
     */
    const { opened, } = state;
    if (opened !== undefined)
      return opened;
    /**
     Fresh queue, stored immediately so every write joins one append order.
     */
    const created = openAppends(destination,);
    state.opened = created;
    return created;
  }

  /**
   Verified destination of this sink,
   probed strictly for explicit destinations
   and resolved through the default discovery otherwise.

   @returns Destination directory proven writable.
   */
  async function resolveDestination(): Promise<string> {
    if (explicitDirectory === undefined)
      return await resolveDefaultLogDirectory();
    await probeLogDirectory(explicitDirectory,);
    return explicitDirectory;
  }

  /**
   Actual verification work, invoked exactly once via the memoized
   `verifyPromise`. Probes the destination directory, falling back from an
   unavailable project to user state. Explicit destinations remain strict.
   The logger owns resulting availability.

   @returns Whether file system logging is available.
   */
  async function runVerify(): Promise<boolean> {
    if (disabled)
      return false;
    try {
      /**
       Verified destination owns all subsequent queued appends for this instance.
       */
      const resolvedDirectory = await resolveDestination();
      state.directory = resolvedDirectory;
      /**
       Housekeeping runs off the logging path and reports its own failures;
       `flush` awaits it so exits and assertions can settle the sweep.
       */
      if (policy !== NO_LOG_RETENTION_SWEEP)
        state.sweep = sweepLogDirectory({
          directory: resolvedDirectory,
          policy,
        },);
      return true;
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'file sink verification failed',
        error,
      },);
      return false;
    }
  }

  /**
   Verifies file system availability via {@link runVerify}, memoizing
   concurrent calls so a second caller never starts a duplicate probe.

   @returns Whether file system logging is available.
   */
  function verify(): Promise<boolean> {
    /**
     Memoized verification promise of this instance.
     */
    const { verifyPromise, } = state;
    if (verifyPromise !== undefined)
      return verifyPromise;

    /**
     Verification work started for this instance.
     */
    const started = runVerify();
    state.verifyPromise = started;
    return started;
  }

  /**
   Writes a single record as a JSONL line to this sink's log file,
   creating that file with this first record.
   The returned completion is this batch's complete append,
   shared with same-batch records so pending-write tracking covers queued records;
   unverified and rejected writes share one completed promise.

   @param record - Log record to write.

   @mutates record - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
   */
  function write(record: object,): Promise<void> {
    /**
     Destination resolved during verification; unverified writes retain their existing no-op behavior.
     */
    const { directory, } = state;
    if (directory === undefined)
      return completedWrite;

    try {
      /**
       Serialization finishes before queue admission, including reentrant writes from user serialization hooks.
       */
      const serialized = JSON.stringify(record,);
      if (serialized === undefined)
        throw new FileRecordSerializationError();
      return appendsQueue(directory,)
        .append(`${serialized}\n`,);
    }
    catch (error: unknown) {
      reportLoggerInternalError({
        context: 'file sink record append failed',
        error,
      },);
      return completedWrite;
    }
  }

  /**
   Settles the retention sweep started during verification.
   Record appends settle through their `write` completions,
   so nothing else is buffered here.
   */
  async function flush(): Promise<void> {
    /**
     Retention sweep started during verification, if any.
     */
    const { sweep, } = state;
    if (sweep === undefined)
      return;
    await sweep;
  }

  return {
    flush,
    verify,
    write,
  };
}
//endregion File sink
