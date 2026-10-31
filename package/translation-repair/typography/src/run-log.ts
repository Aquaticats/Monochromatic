import { join, } from 'node:path';

import { settleWithin, } from '@monochromatic-dev/async-time-core/ts';
import {
  createLogger,
  logger as defaultLogger,
  type Logger,
  type Level,
  sinks,
} from '@monochromatic-dev/logger-core/ts';
import {
  createFileSink,
  NO_LOG_RETENTION_SWEEP,
} from '@monochromatic-dev/logger-core/ts/node.ts';

//region Run log
// WHERE A RUN'S RECORDS GO. A corpus pass writes the records of everything it does through the shared logger's file sink
// into its own run folder, so the run's evidence lives with the run and no reader depends on someone capturing the
// console (the owner's answer of 2026-10-31: a `*.log` capture means the logger was not used as designed).
//
// THE FILE SINK IS BUILT HERE, NEVER LEFT TO THE ENVIRONMENT. The environment route (`MONOCHROMATIC_LOG_DIRECTORY`)
// cannot hold what a run needs: its retention bounds accept only finite numbers (there is no way to turn the sweep
// off), a sweep writes a stamp file into the directory it sweeps, and a default sweep expires whole day directories
// older than 14 days, which are the run's evidence. A sink given `NO_LOG_RETENTION_SWEEP` and an explicit directory
// never sweeps, and it ignores `MONOCHROMATIC_LOG_FILE=off` here (the environment handed to it is empty), because a
// run that quietly records nothing is worse than one that records.
//
// THE WAIT FOR THE DESTINATION IS BOUNDED (`RUN_LOG_VERIFY_MS`), because the pass holds its lock while it asks and a
// destination that never answers would leave it holding the lock and saying nothing. A destination that has not
// answered is a run log that records nothing and says `unanswered`, and the pass refuses to start naming the folder.
//
// THE FOLDER A RUN KEEPS: `<run folder>/log/<UTC day>/<timestamp>.<uuid>.log.jsonl`, one file per process that opened
// the run log, never rotated, one JSON object per line holding exactly `level`, `message` and `timestamp`. The day
// directory is the sink's own and cannot be turned off.

/**
 Name of the directory under a run folder that holds the run's records.
 */
export const RUN_LOG_DIRECTORY = 'log';

/**
 Name ending every file the run log writes: the file sink's own, `{timestamp}.{uuid}.log.jsonl`.
 */
export const RUN_LOG_SUFFIX = '.log.jsonl';

/**
 Longest a run waits for the file sink to say whether its folder is writable, in milliseconds. A verification is a
 directory made and a file written, which takes the time of a disk answering; a destination that has not answered in
 this long is not going to, and a pass that waited on it would hold its lock and print nothing (ledger B1112).
 */
export const RUN_LOG_VERIFY_MS = 30_000;

/**
 What the run folder's destination answered: it is writable, it is not, or it did not answer within the bound.
 */
export type RunLogDestination = 'unanswered' | 'unwritable' | 'writable';

/**
 One run's open log: the logger that shows its lines to a person and records them, the logger that only records, and
 whether the records are really being kept.

 @example
 ```ts
 await using log = await openRunLog({ runDir, },);
 ```
 */
export type RunLog = AsyncDisposable & {
  /**
   Directory the sink writes its day directories into.
   */
  readonly directory: string;

  /**
   Whether the destination proved writable, so records are being kept; a run that needs its evidence refuses when not.
   */
  readonly recording: boolean;

  /**
   Why records are or are not being kept: the destination proved writable, proved not to be, or did not answer within
   the bound, which a refusal says in other words.
   */
  readonly destination: RunLogDestination;

  /**
   Logger that shows each line on the console and records it.
   */
  readonly logger: Logger;

  /**
   Logger that only records, for a line the process also prints itself.
   */
  readonly recordOnly: Logger;

  /**
   Settles every record written so far.
   */
  readonly flush: () => Promise<void>;
};

/**
 Run log installed for the process, which the context roots write through.
 */
const installed: { current?: RunLog; } = {};

/**
 Opens the log of one run.

 @param runDir - run folder the records live in

 @param fileSinkOf - builds the file sink, the logger's own unless a case plants one that does not answer

 @param verifyWithinMs - longest to wait for the sink to say whether the folder is writable; past it the log is open and
 records nothing, and says its destination did not answer

 @returns Open log, which flushes when disposed

 @example
 ```ts
 await using log = await openRunLog({ runDir: '/runs/corpus-pass-tabby', },);
 ```
 */
export async function openRunLog(
  {
    runDir,
    fileSinkOf = createFileSink,
    verifyWithinMs = RUN_LOG_VERIFY_MS,
  }: {
    readonly runDir: string;
    readonly fileSinkOf?: (options: Parameters<typeof createFileSink>[0],) => ReturnType<typeof createFileSink>;
    readonly verifyWithinMs?: number;
  },
): Promise<RunLog> {
  /**
   Directory under the run folder the sink writes into.
   */
  const directory = join(
    runDir,
    RUN_LOG_DIRECTORY,
  );

  /**
   The one sink both loggers write through, so every record of the process lands in one file in arrival order.
   */
  const fileSink = fileSinkOf({
    directory,
    retention: NO_LOG_RETENTION_SWEEP,
    environment: {},
  },);

  /**
   The destination's answer, asked before any record so a run can refuse to start without its evidence, and awaited
   only as long as the bound allows.
   */
  const verifying = fileSink.verify();

  /**
   What the destination answered within the bound.
   */
  const destination = await destinationOf({
    verifying,
    verifyWithinMs,
  },);

  /**
   Whether the destination proved writable.
   */
  const recording = destination === 'writable';

  /**
   Logger that shows and records. A sink that has not answered is left out of it, since the logger would wait for it on
   its first line.
   */
  const { logger, } = createLogger({
    sinks: recording
      ? [
        sinks.createConsoleSink(),
        fileSink,
      ]
      : [sinks.createConsoleSink(),],
  },);

  /**
   Logger that records alone; absent a sink it would refuse every line, so an unwritable run log gets the console
   logger's silence instead.
   */
  const { logger: recordOnly, } = createLogger({
    sinks: [
      recording ? fileSink : sinks.createNoopSink(),
    ],
  },);

  /**
   Settles both loggers.
   */
  async function flush(): Promise<void> {
    await logger.flush();
    await recordOnly.flush();
  }

  return {
    directory,
    recording,
    destination,
    logger,
    recordOnly,
    flush,
    async [Symbol.asyncDispose](): Promise<void> {
      await flush();
    },
  };
}

/**
 What a file sink's verification answered, within a bound.

 @param verifying - sink's verification, started

 @param verifyWithinMs - longest to wait for it

 @returns Whether the destination is writable or not, or that it did not answer in time

 @example
 ```ts
 const destination = await destinationOf({ verifying: sink.verify(), verifyWithinMs: 30_000, },);
 ```
 */
async function destinationOf(
  {
    verifying,
    verifyWithinMs,
  }: {
    readonly verifying: Promise<boolean>;
    readonly verifyWithinMs: number;
  },
): Promise<RunLogDestination> {
  if (await settleWithin({
    promise: verifying,
    ms: verifyWithinMs,
  },) === 'expired')
    return 'unanswered';
  return (await verifying) ? 'writable' : 'unwritable';
}

/**
 Installs a run log for the process: every context root writes through it from now on, and the lines passes print are
 recorded in it.

 @param log - open run log

 @returns Disposable putting back whatever was installed before

 @example
 ```ts
 using _installed = installRunLog({ log, },);
 ```
 */
export function installRunLog({ log, }: { readonly log: RunLog; },): Disposable {
  /**
   Run log standing before this one.
   */
  const before = installed.current;
  installed.current = log;
  return {
    [Symbol.dispose](): void {
      if (before === undefined)
        delete installed.current;
      else
        installed.current = before;
    },
  };
}

/**
 Builds one level method that writes through whichever logger stands at the moment of the call.

 @param level - severity the method logs at

 @returns Forwarding level method

 @example
 ```ts
 const info = forwardingLevel({ level: 'info', },);
 ```
 */
function forwardingLevel({ level, }: { readonly level: Level; },): (message: string,) => void {
  return function logAtLevel(message: string,): void {
    (installed.current
      ?.logger
      ?? defaultLogger)[level](message,);
  };
}

/**
 Logger that forwards every call to the installed run log's logger, or to the default logger while none is installed.

 THE CHOICE IS MADE AT THE CALL, not where the root is built: roots are built when their module loads, long before a pass
 has a run folder to install.
 */
export const runLogger: Logger = {
  debug: forwardingLevel({ level: 'debug', },),
  error: forwardingLevel({ level: 'error', },),
  fatal: forwardingLevel({ level: 'fatal', },),
  info: forwardingLevel({ level: 'info', },),
  trace: forwardingLevel({ level: 'trace', },),
  warn: forwardingLevel({ level: 'warn', },),
  async flush(): Promise<void> {
    await (installed.current
      ?.logger
      ?? defaultLogger).flush();
  },
};

/**
 Prints a line the way a process prints it and records it in the installed run log, when one is installed.

 A PASS'S OWN LINES (`START`, `TALLY`, `DONE`) stay on the console exactly as they were; the record is made beside them
 and carries the line whole as its message, so a reader finds them in the run folder without anyone capturing the
 console.

 @param line - line to print and record

 @example
 ```ts
 printRunLine({ line: 'START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815', },);
 ```
 */
export function printRunLine({ line, }: { readonly line: string; },): void {
  console.log(line,);
  recordRunLine({
    level: 'info',
    line,
  },);
}

/**
 Records a line in the installed run log, when one is installed, and prints nothing.

 FOR A LINE THE CONSOLE GETS ANOTHER WAY: the refusal that ends a pass is printed once by the entry that states it, and
 a second print of it by the logger would change what the operator's console and the built command's stderr say.

 @param level - level the record is written at

 @param line - line to record whole as the message

 @example
 ```ts
 recordRunLine({ level: 'error', line: 'the pass stopped: required provider hyper is not ready', },);
 ```
 */
export function recordRunLine(
  {
    level,
    line,
  }: {
    readonly level: Level;
    readonly line: string;
  },
): void {
  if (installed.current
    ?.recording
    === true)
    installed.current
      .recordOnly[level](line,);
}

//endregion Run log
