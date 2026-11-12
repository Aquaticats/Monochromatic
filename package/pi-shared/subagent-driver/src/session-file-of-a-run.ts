/**
 The session file of a finished run, read from the summary that run wrote. A run that resumes another one needs the file, because
 the summary is what names it and no caller should have to read a session folder to find it.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  isRecord,
  fieldsOf,
} from './read-pi-fields.ts';

/**
 A run whose summary names no session file, so the run asked for cannot resume it.
 */
export class SessionFileUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SessionFileUnreadableError';
}

/**
 Reads the session file out of a summary's text.

 @param shown - the summary as the driver wrote it

 @param runFolder - the run folder, for the message

 @returns the session file

 @throws SessionFileUnreadableError when the summary is not a record, or names no session file

 @example
 ```ts
 const file = sessionFileInSummary({ shown: '{"sessionStats":{"sessionFile":"/s/a.jsonl"}}', runFolder: 'a-1', },);
 ```
 */
export function sessionFileInSummary({
  shown,
  runFolder,
}: {
  readonly shown: string;
  readonly runFolder: string;
}): string {
  /**
   What the summary holds, of any shape.
   */
  const summary: unknown = JSON.parse(shown,);
  if (!isRecord(summary,)) {
    throw new SessionFileUnreadableError(
      `The summary of run ${runFolder} is not a JSON object, so the run this one would resume names no session file.`,
    );
  }
  /**
   Pi's own session statistics.
   */
  const { sessionFile, } = fieldsOf(summary.sessionStats,);
  if (((typeof sessionFile) !== 'string') || (sessionFile === '')) {
    throw new SessionFileUnreadableError(
      `The summary of run ${runFolder} names no session file (sessionStats.sessionFile), so there is nothing to resume. Read ${join(
        runFolder,
        'summary.json',
      )} to see how that run ended.`,
    );
  }
  return sessionFile;
}

/**
 Reads the session file of a finished run in the runs folder.

 @param runsFolder - folder the driver writes one run folder per run into

 @param runFolder - the finished run's folder's name

 @returns the session file

 @throws SessionFileUnreadableError when the summary cannot be read or names no session file

 @example
 ```ts
 const file = await sessionFileOfRun({ runsFolder: '/runs', runFolder: 'copy-route-4', },);
 ```
 */
export async function sessionFileOfRun({
  runsFolder,
  runFolder,
}: {
  readonly runsFolder: string;
  readonly runFolder: string;
}): Promise<string> {
  /**
   Where that run's summary is.
   */
  const summaryFile = join(
    runsFolder,
    runFolder,
    'summary.json',
  );
  try {
    return sessionFileInSummary({
      shown: await readFile(
        summaryFile,
        'utf8',
      ),
      runFolder,
    },);
  }
  catch (error) {
    if (error instanceof SessionFileUnreadableError) {
      throw error;
    }
    throw new SessionFileUnreadableError(
      `The summary of run ${runFolder} could not be read from ${summaryFile}: ${String(error,)}`,
      { cause: error, },
    );
  }
}
