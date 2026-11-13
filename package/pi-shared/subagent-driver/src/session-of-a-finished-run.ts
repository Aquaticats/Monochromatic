/**
 The session one finished run left, read from the summary that names its file: whether that file can be read, and the commands the
 session records.

 A run whose summary names no session, or whose session cannot be read, is a run whose evidence the check cannot compare, and this
 module answers that as its own case rather than raising it: the check reports it as a cut run.

 @module
 */

import {
  type SessionReadings,
  SessionRecordError,
  sessionReadings,
  sessionTextIn,
} from './session-commands-of-a-run.ts';
import {
  SessionFileUnreadableError,
  sessionFileOfRun,
} from './session-file-of-a-run.ts';

/**
 The session file one run's summary names, or why it cannot be read.
 */
type SessionFileOfARun =
  | {
    readonly kind: 'a file';
    readonly file: string;
  }
    | {
      readonly kind: 'unreadable';
      readonly reason: string;
    };

/**
 What a run's session records, or why it cannot be read.
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
 Reads the session file one run's summary names.

 @param runsFolder - folder the driver writes its runs into

 @param run - the run's folder's name

 @returns the file, or why it cannot be read

 @example
 ```ts
 const file = await sessionFileOfTheRun({ runsFolder: '/runs', run: 'review-a-1' });
 ```
 */
async function sessionFileOfTheRun({
  runsFolder,
  run,
}: {
  readonly runsFolder: string;
  readonly run: string;
},): Promise<SessionFileOfARun> {
  try {
    return {
      kind: 'a file',
      file: await sessionFileOfRun({
        runsFolder,
        runFolder: run,
      },),
    };
  }
  catch (error) {
    // A summary that names no session, or a file that cannot be read, is what the check reports as a cut run.
    if (error instanceof SessionFileUnreadableError) {
      return {
        kind: 'unreadable',
        reason: error.message,
      };
    }
    throw error;
  }
}

/**
 Reads what a finished run's session records.

 @param runsFolder - folder the driver writes its runs into

 @param run - the run's folder's name

 @returns the commands the session records, or why they cannot be read

 @example
 ```ts
 const session = await sessionOfTheRun({ runsFolder: '/runs', run: 'review-a-1' });
 ```
 */
export async function sessionOfTheRun({
  runsFolder,
  run,
}: {
  readonly runsFolder: string;
  readonly run: string;
},): Promise<SessionOfARun> {
  /**
   The session file the run's summary names, or why it cannot be read.
   */
  const file = await sessionFileOfTheRun({
    runsFolder,
    run,
  },);
  if (file.kind === 'unreadable') {
    return file;
  }
  try {
    return {
      kind: 'read',
      session: sessionReadings({
        shown: await sessionTextIn({ file: file.file, },),
        file: file.file,
      },),
    };
  }
  catch (error) {
    // A session too large to read, or one holding a line that is no JSON object, is a run whose evidence was cut short.
    if (error instanceof SessionRecordError) {
      return {
        kind: 'unreadable',
        reason: error.message,
      };
    }
    throw error;
  }
}
