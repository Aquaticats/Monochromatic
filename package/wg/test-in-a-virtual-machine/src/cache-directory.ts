/**
 Where this package keeps what a rebuild recreates.

 The archives a guest installs and the archive it unpacks are downloaded or
 packed again on demand, so they live in the user's cache directory and never in
 a checkout, where a build would leave them behind as untracked files.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

/**
 This package's cache directory, created on first use.
 */
export const CACHE_DIRECTORY: string = join(
  process.env.XDG_CACHE_HOME ?? join(
    homedir(),
    '.cache',
  ),
  'monochromatic-dev',
  'wg-test-in-a-virtual-machine',
);

/**
 Creates the cache directory.

 @returns its path

 @example
 ```ts
 await ensureCacheDirectory();
 ```
 */
export async function ensureCacheDirectory(): Promise<string> {
  await mkdir(
    CACHE_DIRECTORY,
    { recursive: true, },
  );
  return CACHE_DIRECTORY;
}
