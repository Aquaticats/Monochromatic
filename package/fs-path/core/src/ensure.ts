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

import { stripQuerySuffix, } from './query-suffix.ts';

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
 Flag of the create that leaves an existing entry alone.

 `wx` fails with `EEXIST` when anything already occupies the path, so a file a
 second writer created between the existence check and this open is never
 opened for writing and never truncated.
 */
const EXCLUSIVE_CREATE_FLAG = 'wx';

/**
 Flag of the create a symbolic link forces.

 An append creates the file when it is absent and writes nothing to a file that
 exists, so the target of a link that appeared in the meantime keeps its bytes.
 */
const CREATE_WITHOUT_TRUNCATION_FLAG = 'a';

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
 Sentinel for a rejected value that carries no POSIX error code, so the code
 reader never needs a nullish union.
 */
const NO_ERROR_CODE = Symbol('rejected value carries no POSIX error code',);

/**
 Reads the POSIX error code a rejected filesystem call carries.

 @param error - Value a `node:fs/promises` call rejected with

 @returns the error code, or {@link NO_ERROR_CODE} when the value carries none

 @example
 ```ts
 if (errorCodeOf(error) === 'EEXIST') return false;
 ```
 */
function errorCodeOf(error: unknown,): string | typeof NO_ERROR_CODE {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- only the optional string `code` of the rejected value is read
  return (error as { code?: string; }).code ?? NO_ERROR_CODE;
}

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
    if (errorCodeOf(error,) === 'ENOENT')
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
 A `?query` suffix is stripped before that decision, so one spelling reaches
 one target in the ensure and empty families alike.

 @param path - Filesystem path to ensure

 @returns the path as given, query suffix included

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
  const parsed = posix.parse(stripQuerySuffix(path,),);

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
 A `?query` suffix names no directory entry and is stripped before the
 filesystem is touched.

 @param path - Directory path to ensure

 @returns the path as given, query suffix included

 @throws When the path exists but is not a directory

 @example
 ```ts
 await ensureDir('logs/archive');
 ```
 */
export async function ensureDir(path: string,): Promise<string> {
  /**
   Path the filesystem operations use: everything from its first `?` on names no entry.
   */
  const cleanPath = stripQuerySuffix(path,);

  /**
   Metadata of the existing path, or the missing sentinel.
   */
  const stats = await statOrMissing(cleanPath,);

  if (stats === MISSING_ENTRY) {
    l.info(`${cleanPath} does not exist, creating recursively`,);
    await mkdir(
      cleanPath,
      { recursive: true, },
    );
    return path;
  }

  if (!stats.isDirectory())
    throw new Error(`Path ${cleanPath} exists but is not a directory.`,);

  l.info(`${cleanPath} already exists, checking accessibility`,);
  await ensureOwnerAccess({
    path: cleanPath,
    stats,
  },);
  return path;
}

/**
 Ensures the parent directory of a path exists.

 A bare name such as `bare.txt` has no parent segment: `posix.parse` reports
 its parent as the empty string, and `mkdir('')` throws `ENOENT`. Its parent
 is the current working directory, so there is nothing to create.

 @param path - Path whose parent directory must exist

 @example
 ```ts
 await ensureParentDirectory('config/app.json');
 ```
 */
async function ensureParentDirectory(path: string,): Promise<void> {
  /**
   Parent directory of the path, `''` when the path is a bare name.
   */
  const { dir, } = posix.parse(path,);

  if (dir === '') {
    l.debug(`${path} names no parent directory, the current one holds it`,);
    return;
  }

  await ensureDir(dir,);
}

/**
 Creates an empty file only when the path holds no entry.

 The create is exclusive, so an entry that appears between any earlier check
 and this open is reported as `EEXIST` instead of being truncated.

 @param path - Path to create

 @returns `true` when this call created the file, `false` when an entry already occupies the path

 @throws Every create failure other than `EEXIST`, a missing parent included

 @example
 ```ts
 const created = await createEmptyFileExclusively('logs/app.log');
 ```
 */
async function createEmptyFileExclusively(path: string,): Promise<boolean> {
  try {
    await writeFile(
      path,
      '',
      { flag: EXCLUSIVE_CREATE_FLAG, },
    );
    return true;
  }
  catch (error: unknown) {
    if (errorCodeOf(error,) === 'EEXIST')
      return false;
    throw error;
  }
}

/**
 Creates the file {@link ensureFile} was asked for, creating its parent
 directory on the one failure that a missing or inaccessible parent causes.

 @param path - Path to create

 @returns `true` when this call created the file, `false` when an entry already occupies the path

 @throws Every failure the exclusive create reports, other than `EEXIST`

 @example
 ```ts
 const created = await createFileWhenAbsent('config/app.json');
 ```
 */
async function createFileWhenAbsent(path: string,): Promise<boolean> {
  try {
    return await createEmptyFileExclusively(path,);
  }
  catch (error: unknown) {
    /**
     Whether the create named a parent directory that is missing.
     */
    const parentIsMissing = errorCodeOf(error,) === 'ENOENT';
    /**
     Whether the create named a parent directory that denies access.
     */
    const parentDeniesAccess = errorCodeOf(error,) === 'EACCES';

    if (!(parentIsMissing || parentDeniesAccess))
      throw error;
  }

  l.info(`${path} has no usable parent directory yet, ensuring it`,);
  await ensureParentDirectory(path,);
  return createEmptyFileExclusively(path,);
}

/**
 Ensures a file exists and is readable/writable.
 Creates the file (and parent directories, via {@link ensureDir}) when missing.
 The create is exclusive: a file a second writer creates while this call runs
 keeps its contents.
 A `?query` suffix names no file entry and is stripped before the filesystem
 is touched.

 @param path - File path to ensure

 @returns the path as given, query suffix included

 @throws When the path exists but is not a regular file

 @example
 ```ts
 await ensureFile('config/app.json');
 ```
 */
export async function ensureFile(path: string,): Promise<string> {
  /**
   Path the filesystem operations use: everything from its first `?` on names no entry.
   */
  const cleanPath = stripQuerySuffix(path,);

  /**
   Whether this call created the file, rather than finding an entry already there.
   */
  const created = await createFileWhenAbsent(cleanPath,);

  if (created) {
    l.info(`${cleanPath} did not exist, created exclusively`,);
    return path;
  }

  /**
   Metadata of the entry that occupies the path now, or the missing sentinel.
   */
  const stats = await statOrMissing(cleanPath,);

  if (stats === MISSING_ENTRY) {
    l.info(`${cleanPath} is a symbolic link whose target is absent, creating the target`,);
    await writeFile(
      cleanPath,
      '',
      { flag: CREATE_WITHOUT_TRUNCATION_FLAG, },
    );
    return path;
  }

  if (!stats.isFile())
    throw new Error(`Path ${cleanPath} exists but is not a file.`,);

  l.info(`${cleanPath} already exists, checking accessibility`,);
  await ensureOwnerAccess({
    path: cleanPath,
    stats,
  },);
  return path;
}
