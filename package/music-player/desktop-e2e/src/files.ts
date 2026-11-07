/**
 Small file-system probes shared by the runner.
 
 @example
 ```ts
 import { pathExists, } from './files.ts';
 
 await pathExists('/etc/hostname',); // true
 ```
 
 @packageDocumentation
 */

import {
  access,
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

/**
 Tells whether a path exists and can be reached.
 
 @param path - file or directory path
 
 @returns whether the path can be accessed
 
 @example
 ```ts
 await pathExists('/etc/hostname',); // true
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  /**
   Tagged logger naming the probe.
   */
  const rl = tagged({
    tag: pathExists.name,
    l: logger,
  },);
  try {
    await access(path,);
    return true;
  }
  catch (error: unknown) {
    rl.debug(`${path} is not reachable: ${String(error,)}`,);
    return false;
  }
}

/**
 Creates a directory and any missing parents; an existing directory is fine.
 
 @param path - directory path
 
 @example
 ```ts
 await ensureDirectory('/work/run/shots',);
 ```
 */
export async function ensureDirectory(path: string,): Promise<void> {
  await mkdir(
    path,
    { recursive: true, },
  );
}

/**
 A file planted for the length of one scope: leaving the `await using` scope removes it.
 */
export type PlantedFile = {
  readonly [Symbol.asyncDispose]: () => Promise<void>;
  /**
   Where the file was written.
   */
  readonly path: string;
};

/**
 Writes a file and returns a handle whose disposal removes it again, then waits so that a watcher in the app has seen
 the removal. It replaces `try` with `finally` around a check that needs a file to exist only while it runs.

 @param path - where to write the file

 @param content - what to write: text or bytes

 @param settleAfterRemovalMs - how long to wait after the removal before the scope ends, in milliseconds

 @returns the planted file; disposing it removes the file, and removing a file that is already gone is fine

 @example
 ```ts
 await using planted = await plantFile({ path: '/work/broken.wav', content: 'not audio', settleAfterRemovalMs: 2_500, },);
 ```
 */
export async function plantFile(
  {
    path,
    content,
    settleAfterRemovalMs,
  }: {
    readonly content: string | Uint8Array;
    readonly path: string;
    readonly settleAfterRemovalMs: number;
  },
): Promise<PlantedFile> {
  await writeFile(
    path,
    content,
  );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        { force: true, },
      );
      await wait(settleAfterRemovalMs,);
    },
  };
}
