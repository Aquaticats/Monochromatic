/**
 Filesystem path-existence guarantees for Node.js / Bun.

 Each function creates the target path (file or directory) when it does
 not exist and verifies read/write accessibility when it does, granting the
 owner read and write bits (plus traverse for directories) when the
 existing mode denies them.
 */

import type { Stats, } from 'node:fs';
import {
  access,
  chmod,
  constants,
  mkdir,
  stat,
  writeFile,
} from 'node:fs/promises';
import { posix, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module-scoped tagged logger.
 */
const l = tagged({ tag: 'path/ensure', },);

/**
 Sentinel for a path with no filesystem entry, so the stat result never
 needs a nullish union.
 */
const MISSING_ENTRY = Symbol('ENOENT: no filesystem entry at the requested path',);

/**
 Permission bits kept from an existing mode when repairing access.
 */
const PERMISSION_BITS = 0o777;

/**
 Owner bits granted to a file whose mode denies the owner: read and write.
 */
const OWNER_FILE_BITS = constants.S_IRUSR
  | constants.S_IWUSR;

/**
 Owner bits granted to a directory whose mode denies the owner: read,
 write, and traverse.
 */
const OWNER_DIRECTORY_BITS = constants.S_IRUSR
  | constants.S_IWUSR
  | constants.S_IXUSR;

/**
 Stats the path, mapping `ENOENT` to {@link MISSING_ENTRY}.

 @param path - Filesystem path to inspect

 @returns Stats of the entry, or {@link MISSING_ENTRY}

 @throws Every stat failure other than `ENOENT`

 @example
 ```ts
 const stats = await statOrMissing('logs');
 ```
 */
async function statOrMissing(path: string,): Promise<Stats | typeof MISSING_ENTRY> {
  try {
    return await stat(path,);
  }
  catch (error: unknown) {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- only the optional string `code` of the thrown value is read
    if ((error as { code?: string; }).code === 'ENOENT')
      return MISSING_ENTRY;
    throw error;
  }
}

/**
 Verifies owner read and write access, granting the bits when denied.

 The repaired mode keeps every existing permission bit and adds the owner's
 read and write bits, plus the traverse bit for directories: a chmod to the
 bare access-check constants would set mode `0o006` and lock the owner out.

 @param path - Existing path to check

 @param stats - Stats of that path, whose mode seeds the repair

 @example
 ```ts
 await ensureOwnerAccess({ path: 'logs', stats });
 ```
 */
async function ensureOwnerAccess({
  path,
  stats,
}: {
  readonly path: string;
  readonly stats: Stats;
},): Promise<void> {
  try {
    await access(
      path,
      constants.R_OK | constants.W_OK,
    );
    l.info(`${path} is accessible`,);
  }
  catch (error: unknown) {
    l.info(`${path} not accessible (${caughtValueText(error,)}), adjusting permissions`,);
    /**
     Owner bits to grant: read and write, plus traverse for directories.
     */
    const ownerBits = stats.isDirectory() ? OWNER_DIRECTORY_BITS : OWNER_FILE_BITS;
    /**
     Existing permission bits with the owner bits added.
     */
    const repairedMode = (stats.mode & PERMISSION_BITS) | ownerBits;
    await chmod(
      path,
      repairedMode,
    );
  }
}

/* oxlint-disable eslint/require-await -- delegates to ensureFile/ensureDir which are async */
/**
 Ensures a path exists as either a file or directory, based on whether
 the path has a file extension, dispatching to {@link ensureFile} or
 {@link ensureDir}.

 @param path - Filesystem path to ensure

 @returns Resolved path string

 @throws When the path exists but is the wrong kind (file vs directory)

 @example
 ```ts
 await ensurePath('logs/app.log');   // creates file + parent dirs
 await ensurePath('logs/archive/');  // creates directory tree
 ```
 */
export async function ensurePath(path: string,): Promise<string> {
  /**
   Parsed segments used solely to read `ext`, which decides file-vs-directory dispatch.
   */
  const parsed = posix.parse(path,);

  if (parsed.ext) {
    l.info(`${path} has extension, ensuring as file`,);
    return ensureFile(path,);
  }

  return ensureDir(path,);
}
/* oxlint-enable eslint/require-await */

/**
 Ensures a directory exists and is readable/writable.
 Creates it recursively when missing.

 @param path - Directory path to ensure

 @returns Resolved path string

 @throws When the path exists but is not a directory

 @example
 ```ts
 await ensureDir('logs/archive');
 ```
 */
export async function ensureDir(path: string,): Promise<string> {
  /**
   Metadata of the existing path, or the missing sentinel.
   */
  const stats = await statOrMissing(path,);

  if (stats === MISSING_ENTRY) {
    l.info(`${path} does not exist, creating recursively`,);
    await mkdir(
      path,
      { recursive: true, },
    );
    return path;
  }

  if (!stats.isDirectory())
    throw new Error(`Path ${path} exists but is not a directory.`,);

  l.info(`${path} already exists, checking accessibility`,);
  await ensureOwnerAccess({
    path,
    stats,
  },);
  return path;
}

/**
 Ensures a file exists and is readable/writable.
 Creates the file (and parent directories, via {@link ensureDir}) when missing.

 @param path - File path to ensure

 @returns Resolved path string

 @throws When the path exists but is not a regular file

 @example
 ```ts
 await ensureFile('config/app.json');
 ```
 */
export async function ensureFile(path: string,): Promise<string> {
  /**
   Metadata of the existing path, or the missing sentinel.
   */
  const stats = await statOrMissing(path,);

  if (stats === MISSING_ENTRY) {
    l.info(`${path} does not exist, creating`,);
    /**
     Parsed segments, read for the parent directory to create first.
     */
    const parsed = posix.parse(path,);
    await ensureDir(parsed.dir,);
    await writeFile(
      path,
      '',
      { flag: 'w', },
    );
    return path;
  }

  if (!stats.isFile())
    throw new Error(`Path ${path} exists but is not a file.`,);

  l.info(`${path} already exists, checking accessibility`,);
  await ensureOwnerAccess({
    path,
    stats,
  },);
  return path;
}
