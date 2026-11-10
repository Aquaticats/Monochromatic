/**
 A logger that keeps its records in memory, for cases that assert on what a call
 site reported.

 The daemon files every record through `logger(1)`, so a case that observed a
 report by writing to the host journal would be both slow and intrusive. This
 logger records the level and message instead, which is what a case checks.

 @module
 */

import type {
  Level,
  Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 One record the logger captured.
 */
export type RecordedRecord = {
  /**
   Level the record was filed at.
   */
  readonly level: Level;
  /**
   Message the record carries.
   */
  readonly message: string;
};

/**
 A recording logger together with what it captured.
 */
export type RecordingLogger = {
  /**
   Logger a call site reports through.
   */
  readonly logger: Logger;
  /**
   Records it captured, in emission order.
   */
  readonly records: readonly RecordedRecord[];
  /**
   How many times a caller asked it to flush.
   */
  readonly flushCount: () => number;
};

/**
 Builds a logger that keeps every record instead of filing it.

 @returns logger, the records it captured, and a flush counter

 @example
 ```ts
 const { logger, records } = recordingLogger();
 reportExpectedFailure({ error, l: logger, operation: 'pkill' });
 expect(records).toHaveLength(1);
 ```
 */
export function recordingLogger(): RecordingLogger {
  /**
   Records captured so far.
   */
  const records: RecordedRecord[] = [];
  /**
   Flush counter, held in a holder so the closure needs no `let`.
   */
  const flushState: { count: number; } = { count: 0 };

  /**
   Builds the filing function for one level, so every method records the same
   shape.

   @param level - level the built method files at

   @returns method recording one message at that level
   */
  function capture(level: Level,): (message: string) => void {
    return function record(message: string,): void {
      records.push({
        level,
        message,
      },);
    };
  }

  return {
    flushCount: function flushCount(): number {
      return flushState.count;
    },
    logger: {
      debug: capture('debug',),
      error: capture('error',),
      fatal: capture('fatal',),
      flush: function flush(): Promise<void> {
        flushState.count += 1;
        return Promise.resolve();
      },
      info: capture('info',),
      trace: capture('trace',),
      warn: capture('warn',),
    },
    records,
  };
}
