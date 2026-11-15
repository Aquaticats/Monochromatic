/**
 Reads a file only when it is an ordinary file, so a named pipe or a device cannot make a run wait for a writer that may never
 come, and within a bound of this process's own, so a read that is merely slow cannot hold it either.

 Why the bound: the unit's own `RuntimeMaxSec` covers a run's unit, and none of the reads this serves happen inside one. The
 launcher reads a checkout's marker, a remembered answer and finished runs' summaries before it asks for a unit, and a run's
 summary and feed after that unit has ended; a named pipe there waits for a writer forever, and a file on a filesystem that has
 stopped answering waits with it. The bound is a bound rather than a measurement: every ordinary file here is a marker, a summary
 or a feed of a few kilobytes, and a read of one that has not finished in this long names itself rather than ending the process
 that asked.

 What the bound does not reach: a read the kernel has already entered and will not leave, which no call inside this process can
 interrupt. That is the same limit every read here shares, and the reason a run's own reads happen inside a unit whose
 `RuntimeMaxSec` ends it.

 @module
 */

import { constants, } from 'node:fs';
import {
  open,
  type FileHandle,
} from 'node:fs/promises';
import { setTimeout as wait, } from 'node:timers/promises';

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
  /**
   The file, opened without waiting: a named pipe with no writer opens at once and is refused below instead of blocking here, and
   the handle closes on every way out of this function.
   */
  await using handle = await open(
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
  return await readWithinTheBound({
    path,
    handle,
  },);
}

/**
 Reads an open ordinary file, and answers the bound's own error when the read has not finished inside it.

 @param path - the file's path, which the bound's message names

 @param handle - the file, already known to be an ordinary file

 @returns the file's text

 @throws ReadThatTookTooLongError when the read did not finish inside {@link ORDINARY_FILE_READ_MILLISECONDS}

 @example
 ```ts
 const text = await readWithinTheBound({ path: '/runs/a-1/summary.json', handle });
 ```
 */
async function readWithinTheBound({
  path,
  handle,
}: {
  readonly path: string;
  readonly handle: FileHandle;
},): Promise<string> {
  /**
   What the text read answers, either way it ends, so a read the bound has already answered for never rejects unhandled.
   */
  const settled = await Promise.race([
    readOrFailure({ handle, },),
    refuseAfterTheBound({ path, },),
  ],);
  if (settled.failed !== undefined) {
    throw settled.failed;
  }
  return settled.text ?? '';
}

/**
 Reads an ordinary file's text, and answers what went wrong as a value rather than as a rejection.

 @param handle - the file, already known to be an ordinary file

 @returns the text, or what the read failed with

 @example
 ```ts
 const settled = await readOrFailure({ handle });
 ```
 */
async function readOrFailure({ handle, }: { readonly handle: FileHandle },): Promise<{
  readonly text?: string;
  readonly failed?: Error;
}> {
  try {
    return { text: await handle.readFile({ encoding: 'utf8', },), };
  }
  catch (error) {
    // Anything thrown by a read is reported as an error, so the caller has one shape to throw onward.
    return { failed: (Error.isError(error,)) ? error : new Error(String(error,)), };
  }
}

/**
 Waits out the bound, then refuses the read that has not finished inside it.

 @param path - the file being read, for the message

 @throws ReadThatTookTooLongError when the bound has passed, which is every way this ends

 @example
 ```ts
 await refuseAfterTheBound({ path: '/runs/a-1/summary.json', });
 ```
 */
async function refuseAfterTheBound({ path, }: { readonly path: string },): Promise<never> {
  await wait(ORDINARY_FILE_READ_MILLISECONDS,);
  throw new ReadThatTookTooLongError(
    `reading ${path} took longer than ${String(ORDINARY_FILE_READ_MILLISECONDS,)} ms, so this process stopped waiting for it.`,
  );
}
