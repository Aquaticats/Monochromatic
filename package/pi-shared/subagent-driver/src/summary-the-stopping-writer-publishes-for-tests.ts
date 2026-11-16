/**
 The summary a writer of a summary that stops between its steps publishes, which is the one both the writer and the test that kills
 it name.

 The last message is the test's own marker, so a reader of the bytes says whose they are: the writer that took the summary's name, or
 the one that was killed before it could.

 @module
 */

import type { RunSummary, } from './driver-types.ts';

/**
 Reads the summary the stopping writer publishes.

 @param marker - the marker this writer's last message carries

 @returns the summary

 @example
 ```ts
 const summary = summaryTheStoppingWriterPublishes({ marker: 'KILLED-WRITER', });
 ```
 */
export function summaryTheStoppingWriterPublishes({ marker, }: { readonly marker: string },): RunSummary {
  return {
    endReason: 'idle',
    totalMilliseconds: 1,
    turns: 1,
    settled: 1,
    toolHistogram: {},
    toolErrors: [],
    dialogs: [],
    sessionStats: {},
    lastAssistantText: marker,
    retries: 0,
    rateLimitErrors: 0,
    piStartAttempts: 1,
  };
}
