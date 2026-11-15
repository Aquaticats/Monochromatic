/**
 Opens a file the way a run's own code must: without following a link, without waiting on anything that is not an ordinary file,
 and with the answer saying which of those it was.

 A named pipe at a path would otherwise hold a run open until somebody wrote to it, a device would read forever, and a link would
 lead the rewrite somewhere nobody meant. `readOrdinaryFile` in this package refuses everything but an ordinary file for a whole
 read; this adds the descriptor, so a caller can also read what it opened and put a replacement beside it.

 @module
 */

import {
  constants,
  open,
  type FileHandle,
} from 'node:fs/promises';
import { presenceFromFailure, } from './what-the-operating-system-answered.ts';

//region Opening

/**
 A file open for reading, or why the path is not one.
 */
export type OrdinaryFileOpened =
  | { readonly handle: FileHandle }
    | { readonly absent: true }
    | { readonly cause: string };

/**
 Opens a path without following a link and without waiting on a pipe or a device.

 @param path - the path

 @returns the handle, that there is nothing there, or why it could not be opened

 @example
 ```ts
 await using opened = await openAnOrdinaryFile({ path, },);
 ```
 */
export async function openAnOrdinaryFile({ path, }: { readonly path: string },): Promise<OrdinaryFileOpened> {
  /**
   The handle, when the operating system opened one.
   */
  const opened = await openWithoutWaiting({ path, },);
  if ('missing' in opened) {
    return { absent: true, };
  }
  if ('failed' in opened) {
    return { cause: opened.failed, };
  }
  /**
   What the path holds, which decides whether reading it can wait.
   */
  const status = await opened.handle
    .stat();
  if (!status.isFile()) {
    await opened.handle
      .close();
    return { cause: `${path} is not an ordinary file, so reading it would wait for a writer or never end`, };
  }
  return { handle: opened.handle, };
}

/**
 Opens a path without following a link and without waiting.

 @param path - the path

 @returns the handle, that the path is not there, or why it could not be opened
 */
async function openWithoutWaiting({ path, }: { readonly path: string },): Promise<
  | { readonly handle: FileHandle }
    | { readonly missing: true }
    | { readonly failed: string }
  > {
  try {
    return { handle: await open(
      path,
      constants.O_RDONLY | constants.O_NOFOLLOW
        | constants.O_NONBLOCK,
    ), };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    /**
     What the operating system answered about the path.
     */
    const presence = await presenceFromFailure({
      path,
      error,
    },);
    return ('absent' in presence)
      ? { missing: true, }
      : {
        failed: ('couldNot' in presence) ? presence.cause : error.name,
      };
  }
}

//endregion Opening

//region Reading

/**
 Reads a file's text when the path holds an ordinary file, without waiting on anything that is not one.

 @param path - the file

 @returns its text, or why there is none

 @example
 ```ts
 const read = await textOfAnOrdinaryFile({ path, },);
 ```
 */
export async function textOfAnOrdinaryFile({ path, }: { readonly path: string },): Promise<
  | { readonly text: string }
    | { readonly absent: true }
    | { readonly cause: string }
  > {
  /**
   The opened file, or why the path is not one.
   */
  const opened = await openAnOrdinaryFile({ path, },);
  if ('absent' in opened) {
    return { absent: true, };
  }
  if ('cause' in opened) {
    return { cause: opened.cause, };
  }
  /**
   The opened file, which is closed when this read ends.
   */
  await using handle = opened.handle;
  return { text: await handle.readFile({ encoding: 'utf8', },), };
}

//endregion Reading
