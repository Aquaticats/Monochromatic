/**
 The runs that ended since a given time, and how each ended: the summary's own reason, the status Pi left with, and the head of
 the last message the run wrote.

 A coordinator's hourly pass asks what finished since the last one, and a run whose summary says how it ended is a better witness
 than the tail of a feed. The summary is the file the driver writes last, so its own last write is when the run ended.

 @module
 */

import {
  readFile,
  readdir,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The name the driver writes a run's summary under.
 */
export const SUMMARY_FILE_NAME = 'summary.json';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a path whose parent is not a folder, which is what a batch list or an output file in the
 runs folder answers when its own summary is looked for.
 */
const NOT_A_FOLDER_CODE = 'ENOTDIR';

/**
 How much of the last message a line carries, enough to tell a report from a refusal and short enough to stay one line.
 */
const LAST_MESSAGE_CHARACTERS = 110;

/**
 One run that ended, as its summary records it.
 */
export type EndedRun = {
  /**
   The run's folder name, which is its name and the attempt's number.
   */
  readonly folder: string;

  /**
   When the run ended, as milliseconds since the epoch.
   */
  readonly endedAtMilliseconds: number;

  /**
   Why the driver says it ended.
   */
  readonly endReason: string;

  /**
   The status Pi left with, or nothing when the summary records none.
   */
  readonly piStatus?: number;

  /**
   The head of the last message the run wrote.
   */
  readonly lastMessage: string;
};

/**
 Reads a message as one line, with every run of whitespace made a single space.

 A scan rather than a regular expression, so the cost is one pass over the message whatever it holds, and a message of any length
 is read the same way.

 @param text - the message

 @returns the message on one line, without leading or trailing space

 @example
 ```ts
 const line = oneLineOf('the first line\n\tand the second');
 ```
 */
export function oneLineOf(text: string,): string {
  /**
   The line as it is built, and whether the last character kept was a space, so no run of space is kept twice.
   */
  const built: LineBeingBuilt = {
    characters: [],
    afterSpace: true,
  };
  for (const character of text) {
    if (isSpaceOf({ character, },)) {
      if (!built.afterSpace) {
        built.characters
          .push(' ',);
        built.afterSpace = true;
      }
      continue;
    }
    built.characters
      .push(character,);
    built.afterSpace = false;
  }
  return built.characters
    .join('',)
    .trimEnd();
}

/**
 The line one message is built into, and whether the last character kept was a space.
 */
type LineBeingBuilt = {
  /**
   The characters kept so far.
   */
  readonly characters: string[];

  /**
   Whether the last character kept was a space, so no run of space is kept twice.
   */
  afterSpace: boolean;
};

/**
 Tells whether a character is one of the spaces a message may hold between words.

 @param character - the character to read

 @returns whether it is a space, a line ending or a tab

 @example
 ```ts
 const space = isSpaceOf({ character: '\n' });
 ```
 */
function isSpaceOf({ character, }: { readonly character: string },): boolean {
  return (character === ' ')
    || (character === '\n')
    || (character === '\r')
    || (character === '\t');
}

/**
 Reads the head of a message as one line.

 @param text - the message

 @returns its first characters, with every line ending made a space

 @example
 ```ts
 const head = headOf('the first line\nand the second');
 ```
 */
function headOf(text: string,): string {
  /**
   The message as one line.
   */
  const oneLine = oneLineOf(text,);
  return (oneLine.length <= LAST_MESSAGE_CHARACTERS)
    ? oneLine
    : `${oneLine.slice(
      0,
      LAST_MESSAGE_CHARACTERS,
    )}...`;
}

/**
 Reads how one run ended, from the summary the driver wrote.

 @param summaryShown - the summary, as it was written

 @returns how the run ended, or nothing when the file is no summary

 @example
 ```ts
 const ended = endedRunIn({ summaryShown: '{"endReason":"idle","lastAssistantText":"done"}', },);
 ```
 */
export function endedRunIn({ summaryShown, }: { readonly summaryShown: string },): {
  readonly read?: {
    readonly endReason: string;
    readonly piStatus?: number;
    readonly lastMessage: string
  }
} {
  /**
   The summary, of any shape.
   */
  const summary: unknown = JSON.parse(summaryShown,);
  if (!isRecord(summary,)) {
    return {};
  }
  /**
   The fields this reads, kept as the summary holds them.
   */
  const {
    endReason,
    lastAssistantText,
    piExit,
  } = summary;
  if ((typeof endReason) !== 'string') {
    return {};
  }
  /**
   The status Pi left with, when the summary records one.
   */
  const exit = isRecord(piExit,) ? piExit : {};
  return {
    read: {
      endReason,
      ...(((typeof exit.code) === 'number') ? { piStatus: exit.code, } : {}),
      lastMessage: headOf(((typeof lastAssistantText) === 'string') ? lastAssistantText : ''),
    },
  };
}

/**
 Reads the run folders a runs folder holds, which is one per run: a batch list, a prompt file and a unit's output file live there
 too, and none of them is a run.

 @param runsFolder - the folder the driver writes one run folder per run into

 @returns the names of its entries that can be folders, in the order the file system lists them

 @example
 ```ts
 const folders = await runFoldersIn('/runs',);
 ```
 */
async function runFoldersIn(runsFolder: string,): Promise<readonly string[]> {
  return (await readdir(
    runsFolder,
    { withFileTypes: true, },
  ))
    .filter(function isARunFolder(entry,) {
      // A symbolic link is followed by the read of its summary, so a link to a run folder is one.
      return entry.isDirectory() || entry.isSymbolicLink();
    },)
    .map(function nameOf(entry,) {
      return entry.name;
    },);
}

/**
 Reads every run that ended since a time.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param since - the time to read from, as milliseconds since the epoch

 @returns the runs that ended since then, oldest first

 @example
 ```ts
 const ended = await endedRunsSince({ runsFolder: '/runs', since: Date.now() - 3_600_000, },);
 ```
 */
export async function endedRunsSince({
  runsFolder,
  since,
}: {
  readonly runsFolder: string;
  readonly since: number;
},): Promise<readonly EndedRun[]> {
  /**
   The runs that ended since then, in the order they are read.
   */
  const ended: EndedRun[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one run folder after another, each summary read before the next, so a broken
     summary names its own run. See doc/lint/sequential-awaits.md. */
  for (const folder of await runFoldersIn(runsFolder,)) {
    /**
     The summary the driver wrote for that run.
     */
    const file = join(
      runsFolder,
      folder,
      SUMMARY_FILE_NAME,
    );
    try {
      /**
       When the summary was last written, which is when the run ended.
       */
      const { mtimeMs, } = await stat(file,);
      if (mtimeMs < since) {
        continue;
      }
      /**
       How that run ended.
       */
      const { read, } = endedRunIn({
        summaryShown: await readFile(
          file,
          'utf8',
        ),
      },);
      if (read === undefined) {
        continue;
      }
      ended.push({
        folder,
        endedAtMilliseconds: mtimeMs,
        ...read,
      },);
    }
    catch (error) {
      // A run folder with no summary is a run that has not ended, which is what every run going now looks like.
      // A run that has not ended has no summary, and a name in the runs folder that is a batch list rather than a run has no
      // folder to look in: neither is a failure of this read.
      if ((errorCodeOf(error,) !== MISSING_PATH_CODE) && (errorCodeOf(error,) !== NOT_A_FOLDER_CODE)) {
        throw error;
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return ended.toSorted(function oldestFirst(
    left,
    right,
  ) {
    return left.endedAtMilliseconds - right.endedAtMilliseconds;
  },);
}

/**
 Builds the line one ended run prints.

 @param run - the run

 @returns the line, which names when it ended, how, and the head of its last message

 @example
 ```ts
 const line = endedRunLineOf({ folder: 'a-1', endedAtMilliseconds: 0, endReason: 'idle', piStatus: 0, lastMessage: 'done' },);
 ```
 */
export function endedRunLineOf(run: EndedRun,): string {
  return `${new Date(run.endedAtMilliseconds,)
    .toISOString()}\t${run.folder}\t${run.endReason}\tpi ${String(run.piStatus ?? 'none',)}\t${run.lastMessage}`;
}

/**
 Reads the time a caller asked for, as milliseconds since the epoch.

 @param written - the time as the caller wrote it, in any form `Date.parse` reads

 @returns the time, or nothing when the text names no time

 @example
 ```ts
 const since = timeIn({ written: '2026-11-12T10:00:00-05:00', },);
 ```
 */
export function timeIn({ written, }: { readonly written: string },): { readonly since?: number } {
  /**
   The time the text names.
   */
  const since = Date.parse(written,);
  return Number.isNaN(since,) ? {} : { since, };
}
