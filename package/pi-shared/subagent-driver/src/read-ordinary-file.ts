/**
 Reads a file only when it is an ordinary file, so a named pipe or a device cannot make a run wait for a writer that may never
 come.

 @module
 */

import { constants, } from 'node:fs';
import { open, } from 'node:fs/promises';

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
 Reads an ordinary file.

 @param path - file to read

 @returns the file's text

 @throws NotAnOrdinaryFileError when the path holds a pipe, a device, a folder or anything else that is not an ordinary file

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
  return await handle.readFile({ encoding: 'utf8', },);
}
