/**
 Reads a file only when it is an ordinary file, so a named pipe or a device cannot make a run wait for a writer that may never
 come, and within a bound of this process's own, so a read that is merely slow cannot hold it either.

 Why the bound: the unit's own `RuntimeMaxSec` covers a run's unit, and none of the reads this serves happen inside one. The
 launcher reads a checkout's marker, a remembered answer and finished runs' summaries before it asks for a unit, and a run's
 summary and feed after that unit has ended; a named pipe there waits for a writer forever, and a file on a filesystem that has
 stopped answering waits with it. The bound is created before anything is opened, so it covers the open, the metadata read and
 the read itself, and not only the last of the three. It is a bound rather than a measurement: every ordinary file here is a
 marker, a summary or a feed of a few kilobytes, and a read of one that has not finished in this long names itself rather than
 ending the process that asked.

 What the bound does not reach: work the kernel has already been asked for and will not answer. When the bound fires, the open,
 the metadata read or the read it interrupted is left with the kernel, which holds that request until the filesystem answers or
 the process ends; this process stops waiting for it, closes the descriptor if the open ever returns, and nothing later waits on
 it. A read that is stuck that way cannot be interrupted from inside the process, which is the same limit every read here shares
 and the reason a run's own reads happen inside a unit whose `RuntimeMaxSec` ends it.

 A reading that answers first stops the bound with it: the timer is cleared, so nothing of it keeps the process alive after the
 answer is in hand.

 @module
 */

import { constants, } from 'node:fs';
import {
  open,
  type FileHandle,
} from 'node:fs/promises';
import { answerWithinABound, } from './an-answer-within-a-bound.ts';

/**
 Opens a file with flags, which a test replaces to hold an open that never answers.

 @param path - the file to open

 @param flags - the flags to open it with

 @returns the open file

 @example
 ```ts
 const handle = await openAFile('/runs/a-1/summary.json', constants.O_RDONLY | constants.O_NONBLOCK,);
 ```
 */
export type OpenAFile = (
  path: string,
  flags: number,
) => Promise<FileHandle>;

/**
 How long one read of an ordinary file may take, in milliseconds.

 A bound rather than a measurement: the files read here are a Git marker, a remembered answer, a summary and a feed, each a few
 kilobytes at most, and a read that has not finished in half a minute is one nothing waits longer for.
 */
export const ORDINARY_FILE_READ_MILLISECONDS: number = 30_000;

/**
 A path that holds something other than an ordinary file, which reading would wait on.
 */
export class NotAnOrdinaryFileError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'NotAnOrdinaryFileError';
}

/**
 A read of an ordinary file that did not finish inside its bound.
 */
export class ReadThatTookTooLongError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReadThatTookTooLongError';
}

/**
 Reads an ordinary file, within this process's own bound.

 @param path - file to read

 @returns the file's text

 @throws NotAnOrdinaryFileError when the path holds a pipe, a device, a folder or anything else that is not an ordinary file

 @throws ReadThatTookTooLongError when the read did not finish inside {@link ORDINARY_FILE_READ_MILLISECONDS}

 @throws the error the open or the read gives, such as a missing file

 @example
 ```ts
 const prompt = await readOrdinaryFile(options.promptFile,);
 ```
 */
export async function readOrdinaryFile(path: string,): Promise<string> {
  return await readOrdinaryFileWithin({ path, },);
}

/**
 Reads an ordinary file within a bound, which covers the open, the metadata read and the read itself.

 The bound is created before the file is opened, so an open or a metadata read that does not answer is answered by the bound rather
 than by nothing.

 @param path - file to read

 @param milliseconds - how long the whole of it may take, which a test shortens and no caller else names

 @param openFile - how a file is opened, which a test replaces to hold an open that never answers

 @returns the file's text

 @throws NotAnOrdinaryFileError when the path holds a pipe, a device, a folder or anything else that is not an ordinary file

 @throws ReadThatTookTooLongError when the open, the metadata read or the read did not finish inside the bound

 @throws the error the open or the read gives, such as a missing file

 @example
 ```ts
 const text = await readOrdinaryFileWithin({ path: '/runs/a-1/summary.json', });
 ```
 */
export async function readOrdinaryFileWithin({
  path,
  milliseconds = ORDINARY_FILE_READ_MILLISECONDS,
  openFile = open,
}: {
  /**
   The file to read.
   */
  readonly path: string;

  /**
   How long the whole of it may take.
   */
  readonly milliseconds?: number;

  /**
   How a file is opened, which a test replaces to hold an open that never answers.
   */
  readonly openFile?: OpenAFile;
},): Promise<string> {
  return await answerWithinABound({
    call: readAnOrdinaryFile({
      path,
      openFile,
    },),
    milliseconds,
    refusal: function readingTookTooLong() {
      return new ReadThatTookTooLongError(
        `reading ${path} took longer than ${String(milliseconds,)} ms, so this process stopped waiting for it. What it opened or asked the kernel for is still the kernel's until the filesystem answers; nothing here waits on it any more.`,
      );
    },
  },);
}

/**
 Reads an ordinary file: opens it without waiting, refuses anything that is not an ordinary file, and reads it.

 @param path - the file to read

 @param openFile - how a file is opened

 @returns the file's text

 @throws NotAnOrdinaryFileError when the path holds a pipe, a device, a folder or anything else that is not an ordinary file

 @throws the error the open or the read gives, such as a missing file

 @example
 ```ts
 const text = await readAnOrdinaryFile({ path: '/repo/.git', openFile: open, });
 ```
 */
async function readAnOrdinaryFile({
  path,
  openFile,
}: {
  readonly path: string;
  readonly openFile: OpenAFile;
},): Promise<string> {
  /**
   The file, opened without waiting: a named pipe with no writer opens at once and is refused below instead of blocking here, and
   the handle closes on every way out of this function.
   */
  await using handle = await openFile(
    path,
    constants.O_RDONLY | constants.O_NONBLOCK,
  );
  /**
   What the path holds, which decides whether reading it can wait.
   */
  const stats = await handle.stat();
  if (!stats.isFile()) {
    throw new NotAnOrdinaryFileError(
      `${path} is not an ordinary file, so reading it would wait for a writer that may never come.`,
    );
  }
  return await handle.readFile({ encoding: 'utf8', },);
}
