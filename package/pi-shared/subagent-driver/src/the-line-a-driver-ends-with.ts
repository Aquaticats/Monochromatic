/**
 The line the driver prints as a run ends: why it ended, how many attempts it made, how many starts Pi needed, and what Pi
 printed when none of them took the prompt.

 A batch prints the last line of what a start printed, and the run folder's `stderr.log` holds Pi's own words; this line is what a
 reader is given instead, so nobody has to open a log to learn why a run did nothing.

 @module
 */

import { join, } from 'node:path';
import type {
  PiStartFailure,
  RunSummary,
} from './driver-types.ts';

/**
 Says how many attempts a run made at the model error it ended on.

 @param summary - what the caller reads

 @returns the attempts, in words, or nothing when the run ended on no model error
 */
function attemptsOf(summary: RunSummary,): string {
  if (summary.modelErrorAttempts === undefined) {
    return '';
  }
  /**
   How many attempts the run made, as text.
   */
  const attempts = String(summary.modelErrorAttempts,);
  return (summary.modelErrorAttempts === 1)
    ? ' after 1 attempt'
    : ` after ${attempts} attempts`;
}

/**
 Says how Pi ended, in its own terms.

 @param start - the start whose Pi ended, when there was one

 @returns the status or the signal, in words
 */
function exitWordsOf(start?: PiStartFailure,): string {
  if (start === undefined) {
    return 'without a status';
  }
  /**
   How that Pi ended.
   */
  const { exit, } = start;
  if (exit.signal !== undefined) {
    return `by ${exit.signal}`;
  }
  if (exit.code !== undefined) {
    return `with status ${String(exit.code,)}`;
  }
  return 'without a status';
}

/**
 Says what Pi's standard error ended with, which is why a reader of the line needs no log.

 @param reason - the reason Pi's own words name, when they name one

 @param lines - the last lines Pi printed

 @returns what the line says about them
 */
function standardErrorWordsOf({
  reason,
  lines,
}: {
  readonly reason?: string;
  readonly lines: readonly string[]
},): string {
  if (reason !== undefined) {
    return `, and its standard error ended with: ${reason}`;
  }
  /**
   The last line Pi printed, when the tail names no reason of its own.
   */
  const lastLine = lines.at(-1,);
  return (lastLine === undefined)
    ? ' and printed nothing to its standard error'
    : `, and its standard error ended with: ${lastLine}`;
}

/**
 Says how many starts Pi needed, and what the last of them printed when every start ended before the prompt was taken.

 @param summary - what the caller reads

 @returns the note, in words, or nothing when no start ended before it took the prompt
 */
function startsNoteOf(summary: RunSummary,): string {
  /**
   The starts that ended before their Pi took the prompt.
   */
  const failures = summary.piStartFailures;
  if ((failures === undefined) || (failures.length === 0)) {
    return '';
  }
  /**
   How many starts the run made, as text.
   */
  const starts = String(summary.piStartAttempts,);
  // The note begins a sentence of its own: what Pi did with the starts is not a clause of the end reason beside it.
  if (summary.endReason !== 'pi-exited-before-prompt') {
    return `. Pi was started ${starts} times, because a start before the last one exited before it took the prompt`;
  }
  /**
   The last start, which is the one that decided how the run reads.
   */
  const last = failures.at(-1,);
  /**
   What Pi printed instead of going on.
   */
  const said = standardErrorWordsOf({
    ...(last?.reason === undefined ? {} : { reason: last.reason, }),
    lines: last?.standardErrorLines ?? [],
  },);
  return (summary.piStartAttempts === 1)
    ? `. Pi exited before it took the prompt on its only start, ${exitWordsOf(last,)}${said}`
    : `. Pi exited before it took the prompt on each of its ${starts} starts, the last of them ${exitWordsOf(last,)}${said}`;
}

/**
 Says what Pi's own automatic retry reported when it gave up on the model error the run ended with, so the line names the layer
 that answered that error rather than leaving a reader to think nothing tried.

 @param summary - what the caller reads

 @returns the note, in words, or nothing when Pi reported giving up on nothing
 */
function piRetryNoteOf(summary: RunSummary,): string {
  /**
   What Pi reported when it stopped retrying.
   */
  const exhausted = summary.piRetryGaveUp;
  if (exhausted === undefined) {
    return '';
  }
  /**
   How many attempts Pi's retry made, as text.
   */
  const attempts = String(exhausted.attempts,);
  return (exhausted.attempts === 1)
    ? ', and Pi\'s own automatic retry gave up on it after 1 attempt'
    : `, and Pi's own automatic retry gave up on it after ${attempts} attempts`;
}

/**
 Says what Pi reported of a compaction, when one kept the run's prompt waiting, so the line names why Pi took nothing rather than
 leaving a reader to guess.

 @param summary - what the caller reads

 @returns the note, in words, or nothing when no compaction kept the prompt waiting
 */
function compactionNoteOf(summary: RunSummary,): string {
  return (summary.compactionProblem === undefined)
    ? ''
    : `, because Pi could not put its session in order: ${summary.compactionProblem}`;
}

/**
 Builds the line the driver prints as a run that did not end on its own terms ends, naming where the summary and Pi's standard
 error are.

 @param summary - what the caller reads

 @param runFolder - folder the run wrote to

 @returns the line, without its line ending

 @example
 ```ts
 const line = endingLineOf({ summary, runFolder: '/runs/a-1', },);
 ```
 */
export function endingLineOf({
  summary,
  runFolder,
}: {
  readonly summary: RunSummary;
  readonly runFolder: string
},): string {
  /**
   What the line says before the files it names, ending where the reason it quotes already ended a sentence.
   */
  const sentence = `The Pi run ended with "${summary.endReason}"${attemptsOf(summary,)}${startsNoteOf(summary,)}${piRetryNoteOf(summary,)}${compactionNoteOf(summary,)}`;
  return `${sentence}${sentence.endsWith('.') ? '' : '.'} The summary is in ${
    join(
      runFolder,
      'summary.json',
    )
  } and Pi's standard error in ${
    join(
      runFolder,
      'stderr.log',
    )
  }.`;
}
