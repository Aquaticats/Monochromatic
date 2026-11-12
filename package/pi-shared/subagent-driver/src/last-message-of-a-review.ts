/**
 A review that ends without writing `review.md` leaves its findings only in its last message, and a review's worktree is kept until
 that file exists. This reads the last message whole out of the Pi session file the newest finished run names, and writes it as
 the review.

 The summary's own `lastAssistantText` cannot be the source: it is cut to `FEED_PREVIEW_CHARACTERS.assistantText`, which is 900
 characters, and a review of six findings is longer than that.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';
import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';
import {
  REVIEW_FILE_NAME,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { sessionFileInSummary, } from './session-file-of-a-run.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The note a saved review carries, so a reader knows the file was written after the run rather than by it.
 */
export const SAVED_REVIEW_NOTE: string =
  'Saved by save-the-final-message-of-a-review-that-wrote-no-review-file from the last message of the run named beside it, '
    + 'which wrote no review.md.';

/**
 What saving one review did.
 */
export type SavedReview = {
  /**
   The review's name.
   */
  readonly review: string;

  /**
   Whether a review file was written.
   */
  readonly saved: boolean;

  /**
   The line to print.
   */
  readonly line: string;
};

/**
 Reads the text of the last assistant message in a session's records.

 @param shown - the session file, one JSON record per line

 @returns the message's text parts joined, or empty text when the session holds no assistant message

 @example
 ```ts
 const text = lastAssistantTextIn({ shown: '{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"a"}]}}', },);
 ```
 */
export function lastAssistantTextIn({ shown, }: { readonly shown: string },): string {
  /**
   The last assistant message found so far.
   */
  const parts: string[][] = [];
  for (const line of shown.split('\n',)) {
    if (line === '') {
      continue;
    }
    /**
     One record of the session.
     */
    const record: unknown = JSON.parse(line,);
    if ((!isRecord(record,)) || (record.type !== 'message')) {
      continue;
    }
    /**
     The message the record carries.
     */
    const message = fieldsOf(record.message,);
    if (message.role !== 'assistant') {
      continue;
    }
    parts.push((Array.isArray(message.content) ? message.content : [])
      .flatMap(function textOf(part,) {
        return (isRecord(part,) && (part.type === 'text')
          && ((typeof part.text) === 'string'))
          ? [part.text,]
          : [];
      },));
  }
  return (parts.at(-1) ?? []).join('\n',)
    .trim();
}

/**
 Reads a folder's entries, or nothing when the folder is not there.

 @param folder - the folder to list

 @returns its entries' names, or nothing at all for a folder that is not there

 @example
 ```ts
 const entries = await entriesOrNothingIn('/runs');
 ```
 */
async function entriesOrNothingIn(folder: string,): Promise<readonly string[]> {
  try {
    return await readdir(folder,);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Reads the newest finished run of a review.

 @param runsFolder - folder the driver writes one run folder per run into

 @param review - the review's name

 @returns the run folder's name, or nothing when no run of it has finished

 @example
 ```ts
 const newest = await newestRunOf({ runsFolder: '/runs', review: 'a-round-1', },);
 ```
 */
export async function newestRunOf({
  runsFolder,
  review,
}: {
  readonly runsFolder: string;
  readonly review: string;
}): Promise<{ readonly run?: string }> {
  /**
   The run's name, which every folder of it starts with.
   */
  const run = runNameOfAReview({ review, },);
  /**
   The entries of the runs folder, of which a folder that is not there holds none.
   */
  const entries = await entriesOrNothingIn(runsFolder,);
  /**
   The numbers of the folders of that name, highest first.
   */
  const numbers = entries.flatMap(function numberOf(folder,) {
    if (!folder.startsWith(`${run}-`,)) {
      return [];
    }
    /**
     The text after the run's name.
     */
    const written = folder.slice(run.length + 1,);
    return isDigitsOnly(written,)
      ? [{
        folder,
        number: Number(written,),
      },]
      : [];
  },)
    .toSorted(function newestFirst(
      left: {
        readonly folder: string;
        readonly number: number
      },
      right: {
        readonly folder: string;
        readonly number: number
      },
    ) {
      return right.number - left.number;
    },);
  for (const candidate of numbers) {
    /**
     Whether this folder holds a summary.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- the newest folder with a summary wins, so the folders are tried newest first and the first answer ends it; see doc/lint/sequential-awaits.md
    const there = await isThere(join(
      runsFolder,
      candidate.folder,
      'summary.json',
    ),);
    if (there) {
      return { run: candidate.folder, };
    }
  }
  return {};
}

/**
 Says whether a path is there.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/runs/a-1/summary.json',);
 ```
 */
async function isThere(path: string,): Promise<boolean> {
  try {
    await readFile(
      path,
      { encoding: 'utf8', },
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Saves the last message of a review's newest run as its `review.md`, unless the review wrote one.

 @param runsFolder - folder the driver writes one run folder per run into

 @param reviewFolder - folder that holds one folder per review

 @param review - the review's name

 @returns what was done, and the line to print

 @example
 ```ts
 const saved = await saveTheLastMessage({ runsFolder: '/runs', reviewFolder: '/reviews', review: 'a-round-1', },);
 ```
 */
export async function saveTheLastMessage({
  runsFolder,
  reviewFolder,
  review,
}: {
  readonly runsFolder: string;
  readonly reviewFolder: string;
  readonly review: string;
}): Promise<SavedReview> {
  /**
   The file a review is read from.
   */
  const reviewFile = join(
    reviewFolder,
    review,
    REVIEW_FILE_NAME,
  );
  if (await isThere(reviewFile,)) {
    return {
      review,
      saved: false,
      line: `${review}: has its review.md`,
    };
  }
  /**
   The newest finished run of that review.
   */
  const newest = await newestRunOf({
    runsFolder,
    review,
  },);
  if (newest.run === undefined) {
    return {
      review,
      saved: false,
      line: `${review}: no review.md and no finished run`,
    };
  }
  /**
   The session file that run's summary names.
   */
  const sessionFile = sessionFileInSummary({
    shown: await readFile(
      join(
        runsFolder,
        newest.run,
        'summary.json',
      ),
      'utf8',
    ),
    runFolder: newest.run,
  },);
  /**
   The review's text, which is its last assistant message.
   */
  const text = lastAssistantTextIn({ shown: await readFileOrNothing(sessionFile,), },);
  if (text === '') {
    return {
      review,
      saved: false,
      line: `${review}: no review.md and an empty last message in ${newest.run}`,
    };
  }
  await mkdir(
    join(
      reviewFolder,
      review,
    ),
    { recursive: true, },
  );
  await writeFile(
    reviewFile,
    `<!-- ${SAVED_REVIEW_NOTE} ${newest.run} -->\n\n${text}\n`,
    { encoding: 'utf8', },
  );
  return {
    review,
    saved: true,
    line: `${review}: saved its last message as review.md (${String(text.length,)} characters)`,
  };
}

/**
 Reads a file, or answers nothing when it is not there.

 @param file - the file to read

 @returns its text, or empty text when it is missing

 @example
 ```ts
 const shown = await readFileOrNothing('/sessions/a.jsonl');
 ```
 */
async function readFileOrNothing(file: string,): Promise<string> {
  try {
    return await readFile(
      file,
      'utf8',
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}
