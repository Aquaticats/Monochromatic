/**
 Plain-words reports for sink verification verdicts. @module

 The logger cannot report through its own sinks without recursing, so these
 reports go straight to the host console. Each one names the sink, what happened
 to the records logged for it, and the one setting that changes the outcome, so
 a reader who only sees a log line can tell whether anything was lost.
 */

import {
  reportLoggerCondition,
  reportLoggerInternalError,
} from './error-format.ts';

import type { Sink, } from './types.ts';

//region A reader has to learn which sink, what happened, and what to do next.
/**
 Names one sink inside a diagnostic: its own name when it ships one, plus the
 startup-order index that identifies it even when it does not.

 @param entryIndex - Sink entry index from the logger's startup order.

 @param sink - Sink to name; its own `name` is used when it ships one.

 @returns Sink label quoted at the start of a verification report.

 @example
 ```ts
 sinkLabel({ entryIndex: 3, sink: createFileSink() }); // 'file sink (entry 3)'
 ```
 */
export function sinkLabel({
  entryIndex,
  sink,
}: {
  readonly entryIndex: number;
  readonly sink: Sink;
},): string {
  /**
   Diagnostic name the sink supplies; absent for third-party sinks that ship none.
   */
  const { name, } = sink;
  return (name === undefined)
    ? `sink (entry ${entryIndex})`
    : `${name} sink (entry ${entryIndex})`;
}

/**
 Clause naming what happens to the records held for a sink that failed.

 @param kept - Records held for the sink when its verdict arrived.

 @returns Clause completing a verification failure sentence.
 */
function lostRecordsClause({ kept, }: { readonly kept: number; },): string {
  return (kept === 0)
    ? 'it receives no records from here on'
    : `it receives no records, the ${kept} record${(kept === 1) ? '' : 's'} logged before it answered ${(kept === 1) ? 'is' : 'are'} dropped`;
}

/**
 Reports a sink whose verifier threw or rejected, naming the records kept for
 it and the sinks that keep logging.

 @param entryIndex - Sink entry index whose verifier failed.

 @param sink - Sink whose verifier failed, named in the report.

 @param error - Caught value the verifier threw or rejected with.

 @param kept - Records held for the sink when its verdict arrived.

 @example
 ```ts
 reportVerificationFailure({ entryIndex: 0, sink, error, kept: 2 });
 ```
 */
export function reportVerificationFailure({
  entryIndex,
  sink,
  error,
  kept,
}: {
  readonly entryIndex: number;
  readonly sink: Sink;
  readonly error: unknown;
  readonly kept: number;
},): void {
  reportLoggerInternalError({
    context: `${sinkLabel({
      entryIndex,
      sink,
    },)} verification failed; ${lostRecordsClause({ kept, },)}, and logging continues through the remaining sinks`,
    error,
  },);
}

/**
 Reports a probe that had not answered inside its limit, stating that the sink
 keeps its records and can still attach, plus the setting that changes the wait.

 @param entryIndex - Sink entry index whose probe is still running.

 @param sink - Sink whose probe is still running, named in the report.

 @param timeoutMs - Limit the probe ran past, in milliseconds.

 @example
 ```ts
 reportVerificationDeadline({ entryIndex: 3, sink, timeoutMs: 5_000 });
 ```
 */
export function reportVerificationDeadline({
  entryIndex,
  sink,
  timeoutMs,
}: {
  readonly entryIndex: number;
  readonly sink: Sink;
  readonly timeoutMs: number;
},): void {
  reportLoggerCondition({
    message: `${sinkLabel({
      entryIndex,
      sink,
    },)} verification had not answered within ${String(timeoutMs,)}ms; the probe is still running, so this sink simply receives nothing yet and the records logged meanwhile are kept for it. Logging continues through the remaining sinks. Pass a larger verifyTimeoutMs to createLogger if a slow but working probe is expected.`,
  },);
}
//endregion A reader has to learn which sink, what happened, and what to do next.
