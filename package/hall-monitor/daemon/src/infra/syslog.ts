/**
 Journal routing for the hall-monitor daemon.

 The daemon reports through `logger(1)`, which files each line in the system
 journal, so `journalctl -t hall-monitor -f` follows a run.

 This module used to keep its own `log` object: four functions over a
 fire-and-forget spawn whose failures were discarded. The same routing is now a
 sink the shared logger owns, which gains level filtering, tag composition, and a
 flush that waits for the journal writes before the process exits.

 @module
 */

import spawn from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  createLogger,
  tagged,
  type Level,
  type Logger,
  type LogRecord,
  type Sink,
} from '@monochromatic-dev/logger-core/ts';

//region Journal arguments

/**
 Journal identifier every record of this daemon is filed under.
 */
const JOURNAL_TAG = 'hall-monitor';

/**
 syslog priority each log level is filed at.
 `trace` is filed as `debug` because syslog has no finer level, and `fatal` as
 `crit` because it outranks the `err` an ordinary failure uses.
 */
const PRIORITY_BY_LEVEL: Record<Level, string> = {
  debug: 'debug',
  error: 'err',
  fatal: 'crit',
  info: 'info',
  trace: 'debug',
  warn: 'warning',
};

/**
 Deadline for one `logger(1)` invocation.
 The utility measured under 10 ms per invocation on this daemon's host, so this
 bound only fires once the journal socket has stopped answering; without it a
 blocked write would hold the daemon's logging forever.
 */
const JOURNAL_WRITE_TIMEOUT_MS = 5_000;

/**
 Builds the argv one record is filed with.

 The message is placed after `--` so a message starting with a dash is never read
 as an option, and the journal tag comes from {@link JOURNAL_TAG} so every line of
 this daemon carries the same identifier.

 @param record - record to file

 @returns argv for the journal utility

 @example
 ```ts
 journalArgumentsFor({ level: 'warn', message: 'capture failed', timestamp: 0 });
 // ['-t', 'hall-monitor', '-p', 'user.warning', '--', 'capture failed']
 ```
 */
function journalArgumentsFor(record: LogRecord,): readonly string[] {
  return [
    '-t',
    JOURNAL_TAG,
    '-p',
    `user.${PRIORITY_BY_LEVEL[record.level]}`,
    '--',
    record.message,
  ];
}

//endregion Journal arguments

//region Journal sink

/**
 Files one journal line.

 @param args - argv handed to the journal utility
 */
type JournalRunner = (args: readonly string[]) => Promise<void>;

/**
 Files one journal line through `logger(1)`.

 @param args - argv handed to the journal utility
 */
async function runLoggerUtility(args: readonly string[],): Promise<void> {
  await spawn(
    'logger',
    [...args,],
    {
      stderr: 'ignore',
      stdin: 'ignore',
      stdout: 'ignore',
      timeout: JOURNAL_WRITE_TIMEOUT_MS,
    },
  );
}

/**
 Writes one record to the journal.

 A journal utility that is missing or rejects the message falls back to stderr
 rather than losing the record, because a gap in the record of a monitoring
 daemon is worse than a line routed elsewhere, and a service manager collects
 stderr anyway.

 @param record - record the logger handed this sink

 @param run - journal utility invocation, injected so a test observes the argv
 without writing to the host journal
 */
async function writeRecordToJournal({
  record,
  run,
}: Readonly<{
  record: LogRecord;
  run: JournalRunner;
}>): Promise<void> {
  try {
    await run(journalArgumentsFor(record,),);
  }
  catch (error: unknown) {
    process.stderr
      .write(`${JOURNAL_TAG}: ${record.message} (journal write failed: ${caughtValueText(error,)})\n`,);
  }
}

/**
 Always-available verification for the journal sink.

 Kept outside {@link createJournalSink} so the sink construction does not
 recreate it, and documented here because the reason it never probes is the
 sink's fallback rather than an omission.

 @returns whether the sink can receive records, always true
 */
function verifyJournal(): Promise<boolean> {
  return Promise.resolve(true,);
}

/**
 Builds the journal sink described by this module's doc.

 `verify` resolves true without probing the utility, because
 {@link writeRecordToJournal} already delivers to stderr when the utility is
 unusable. Reporting the sink unavailable instead would leave the logger with no
 backend at all, which it reports by throwing on the next record and would take
 the daemon down.

 @param run - journal utility invocation; defaults to `logger(1)`

 @returns sink filing every record once, on stderr when the journal refuses it

 @example
 ```ts
 const written: (readonly string[])[] = [];
 const sink = createJournalSink({ run: async (args) => { written.push(args); } });
 await sink.write({ level: 'info', message: 'capture complete', timestamp: 0 });
 ```
 */
function createJournalSink({
  run = runLoggerUtility,
}: Readonly<{
  run?: JournalRunner;
}> = {},): Sink {
  return {
    name: 'journal',
    verify: verifyJournal,
    write: function writeToJournal(record: LogRecord,): Promise<void> {
      return writeRecordToJournal({
        record,
        run,
      },);
    },
  };
}

//endregion Journal sink

//region Logger

/**
 Logger over the journal sink alone, so an event is filed once rather than once
 locally and once in the journal.
 */
const { logger: journalLogger, } = createLogger({ sinks: [createJournalSink(),], },);

/**
 Daemon logger: tagged with the journal identifier so a line still names the
 daemon when it reaches stderr instead, and the only logging entry point the
 daemon modules use.
 */
const log: Logger = tagged({
  l: journalLogger,
  tag: JOURNAL_TAG,
},);

//endregion Logger

//region Expected failures

/**
 Reports a caught value a call site already expected.

 LG2 asks every `catch (error)` to use its binding, even when the failure is
 expected: a silently swallowed `pkill` that matched nothing, or a `/proc` read
 that hit a permission wall, is exactly what an operator needs when the daemon
 misbehaves. The report is filed at `debug` so a routine, expected condition
 never crowds the journal at a level an operator watches.

 @param error - caught value

 @param l - logger the report is filed through

 @param operation - plain description of what failed

 @throws whatever `error` carries when it is not an `Error`, so a throw nobody planned stays fatal

 @example
 ```ts
 try {
   await spawn('pkill', ['-f', 'llama-server'],);
 }
 catch (error: unknown) {
   reportExpectedFailure({ error, l, operation: 'pkill', });
 }
 ```
 */
function reportExpectedFailure({
  error,
  l,
  operation,
}: Readonly<{
  error: unknown;
  l: Logger;
  operation: string;
}>): void {
  if (!Error.isError(error,))
    throw error;
  l.debug(`${operation} failed: ${caughtValueText(error,)}`,);
}

//endregion Expected failures

export {
  createJournalSink,
  journalArgumentsFor,
  log,
  reportExpectedFailure,
};
export type { JournalRunner, };
