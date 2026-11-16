/**
 The summary and the session one finished run left: why it ended, the model it started on, and the commands its session records.

 Nothing here opens a file by its path and reads it afterwards. The summary and the session are each read through one descriptor
 that is opened without waiting for a writer, so a named pipe at either path is answered as a run whose evidence was cut short
 rather than holding the check for ever. A summary that names no session, or a session that is not there, is the same answer: a run
 the check cannot compare.

 @module
 */

import {
  type SessionReadings,
  SessionRecordError,
  sessionReadings,
  sessionTextIn,
} from './session-commands-of-a-run.ts';
import { readRegularFileAt, } from './reading-a-regular-file.ts';
import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';

/**
 What a run's summary says about it.
 */
export type SummaryOfARun =
  | {
    readonly kind: 'read';

    /**
     Why the run ended, as its summary names it.
     */
    readonly endReason: string;

    /**
     The session file the summary names.
     */
    readonly sessionFile: string;
  }
    | {
      readonly kind: 'unreadable';
      readonly reason: string;
    };

/**
 What a finished run's session records, or why it cannot be read.
 */
export type SessionOfARun =
  | {
    readonly kind: 'read';
    readonly session: SessionReadings;
  }
    | {
      readonly kind: 'unreadable';
      readonly reason: string;
    };

/**
 Reads one file's text as the JSON value it holds.

 @param shown - the file's text

 @param file - the file's path, for the message

 @returns the value, or why the text is no JSON

 @example
 ```ts
 const kept = jsonIn({ shown: '{"endReason":"idle"}', file: '/runs/a-1/summary.json' });
 ```
 */
function jsonIn({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): {
  readonly kind: 'read';
  readonly value: unknown
} | {
  readonly kind: 'unreadable';
  readonly reason: string
} {
  try {
    return {
      kind: 'read',
      value: JSON.parse(shown,),
    };
  }
  catch (error) {
    return {
      kind: 'unreadable',
      reason: `${file} is no JSON: ${String(error,)}`,
    };
  }
}

/**
 Reads why a finished run ended and the session file its summary names.

 @param runsFolder - folder the driver writes its runs into

 @param run - the run's folder's name

 @returns what the summary says, or why it cannot be read

 @example
 ```ts
 const summary = await summaryOfTheRun({ runsFolder: '/runs', run: 'review-a-1' });
 ```
 */
export async function summaryOfTheRun({
  runsFolder,
  run,
}: {
  readonly runsFolder: string;
  readonly run: string;
},): Promise<SummaryOfARun> {
  /**
   The summary's path, which the driver writes whole and last.
   */
  const file = `${runsFolder}/${run}/summary.json`;
  /**
   What is at that path, read as one regular file without waiting.
   */
  const read = await readRegularFileAt({ path: file, },);
  if (read.kind !== 'text') {
    return {
      kind: 'unreadable',
      reason: (read.kind === 'no file')
        ? `${file} is not there, so why that run ended cannot be read`
        : `${read.reason}, so why that run ended cannot be read`,
    };
  }
  /**
   What the summary holds, of any shape.
   */
  const kept = jsonIn({
    shown: read.text,
    file,
  },);
  if (kept.kind === 'unreadable') {
    return kept;
  }
  if (!isRecord(kept.value,)) {
    return {
      kind: 'unreadable',
      reason: `${file} is no JSON object, so why that run ended cannot be read`,
    };
  }
  /**
   The summary's own fields.
   */
  const summary = fieldsOf(kept.value,);
  /**
   Pi's own session statistics, which name the session file.
   */
  const stats = fieldsOf(summary.sessionStats,);
  /**
   The session file's path, as the summary names it.
   */
  const {sessionFile} = stats;
  if (((typeof sessionFile) !== 'string') || (sessionFile === '')) {
    return {
      kind: 'unreadable',
      reason: `${file} names no session file (sessionStats.sessionFile), so there is nothing to compare`,
    };
  }
  return {
    kind: 'read',
    endReason: (typeof summary.endReason) === 'string' ? summary.endReason : '',
    sessionFile,
  };
}

/**
 Reads what a finished run's session records.

 @param sessionFile - the session file the run's summary names

 @returns the commands the session records, or why they cannot be read

 @example
 ```ts
 const session = await sessionOfTheRun({ sessionFile: '/sessions/a.jsonl' });
 ```
 */
export async function sessionOfTheRun({
  sessionFile,
}: {
  readonly sessionFile: string;
},): Promise<SessionOfARun> {
  try {
    return {
      kind: 'read',
      session: sessionReadings({
        shown: await sessionTextIn({ file: sessionFile, },),
        file: sessionFile,
      },),
    };
  }
  catch (error) {
    // A session that is not there, is no regular file, or holds a line that is no JSON object is a run whose evidence was cut
    // short; the check reports it as a cut run rather than raising the failure.
    if (error instanceof SessionRecordError) {
      return {
        kind: 'unreadable',
        reason: error.message,
      };
    }
    throw error;
  }
}
