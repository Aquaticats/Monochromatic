/**
 The Rust toolchain that `toolchain.lock.json` records, for container builds that install Rust themselves.

 Rationale: the Containerfiles of `package/terminal/app`, `package/nested-wayland-session/cli`,
 `package/music-player/desktop-app` and `package/forbidden-strings/core` once carried a hand-kept copy of the lock's
 nightly that went stale; they now take it as a build argument from their callers, and this module is where callers read it.
 */

import { readFile, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

/**
 Name of the build argument the Containerfiles declare for the Rust toolchain.
 */
export const RUST_TOOLCHAIN_BUILD_ARGUMENT_NAME = 'RUST_TOOLCHAIN';

/**
 Name of the lock file that records the toolchain.
 */
export const TOOLCHAIN_LOCK_FILE_NAME = 'toolchain.lock.json';

/**
 Thrown when no lock file is found above a directory or the lock carries no Rust toolchain.
 */
export class LockedRustToolchainError extends Error {
  /**
   Builds the error.

   @param message - what was missing and where it was looked for
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'LockedRustToolchainError';
  }
}

/**
 Parses the lock's text.

 @param lockText - contents of `toolchain.lock.json`

 @returns the parsed document, whose shape is not yet checked

 @throws LockedRustToolchainError when the text is not JSON
 */
function parseLock(lockText: string,): unknown {
  try {
    return JSON.parse(lockText,);
  }
  catch (error) {
    throw new LockedRustToolchainError(`${TOOLCHAIN_LOCK_FILE_NAME} is not JSON: ${String(error,)}`,);
  }
}

/**
 Reads one property of a value that may not be an object.

 @param value - any parsed JSON value

 @param key - property name to read

 @returns the property, or `undefined` when the value is not an object
 */
function childOf({
  value,
  key,
}: {
  readonly value: unknown;
  readonly key: string;
},): unknown {
  return (((typeof value) === 'object') && (value !== null)) ? Reflect.get(
    value,
    key,
  ) : undefined;
}

/**
 Reads the Rust toolchain name out of the lock's text.

 @param lockText - contents of `toolchain.lock.json`

 @returns the dated toolchain name, for example `nightly-2026-11-07`

 @throws LockedRustToolchainError when the text is not JSON or `tools.rust.version` is not a non-empty string

 @example
 ```ts
 lockedRustToolchainFromText('{"tools":{"rust":{"version":"nightly-2026-11-07"}}}',);
 // 'nightly-2026-11-07'
 ```
 */
export function lockedRustToolchainFromText(lockText: string,): string {
  /**
   The `tools.rust.version` value of the parsed lock, `undefined` along any missing step.
   */
  const version: unknown = childOf({
    value: childOf({
      value: childOf({
        value: parseLock(lockText,),
        key: 'tools',
      },),
      key: 'rust',
    },),
    key: 'version',
  },);
  if (((typeof version) !== 'string') || (version === '')) {
    throw new LockedRustToolchainError(`${TOOLCHAIN_LOCK_FILE_NAME} has no tools.rust.version string`,);
  }
  return version;
}

/**
 Reads the lock in one directory.

 @param directory - directory that may hold `toolchain.lock.json`

 @returns the lock's text, or the empty string when the directory has no lock
 */
async function readLockTextIn(directory: string,): Promise<string> {
  try {
    return await readFile(
      join(
        directory,
        TOOLCHAIN_LOCK_FILE_NAME,
      ),
      'utf8',
    );
  }
  catch (error) {
    if ((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ENOENT')) {
      return '';
    }
    throw error;
  }
}

/**
 Reads the locked Rust toolchain from the nearest lock at or above a directory.

 @param directory - directory being tried

 @param startDirectory - directory the search began in, named in the error when nothing is found

 @returns the dated toolchain name

 @throws LockedRustToolchainError when no ancestor holds a lock or the lock has no Rust toolchain
 */
async function readFromNearestLock({
  directory,
  startDirectory,
}: {
  readonly directory: string;
  readonly startDirectory: string;
},): Promise<string> {
  /**
   Text of the lock in this directory, empty when there is none.
   */
  const lockText = await readLockTextIn(directory,);
  if (lockText !== '') {
    return lockedRustToolchainFromText(lockText,);
  }

  /**
   Next directory up, equal to this one at the filesystem root.
   */
  const parent = dirname(directory,);
  if (parent === directory) {
    throw new LockedRustToolchainError(
      `no ${TOOLCHAIN_LOCK_FILE_NAME} in ${startDirectory} or any directory above it`,
    );
  }
  return readFromNearestLock({
    directory: parent,
    startDirectory,
  },);
}

/**
 Reads the locked Rust toolchain from the nearest lock above a directory.

 @param startDirectory - directory to start from, usually the caller's working directory

 @returns the dated toolchain name

 @throws LockedRustToolchainError when no ancestor holds a lock or the lock has no Rust toolchain

 @example
 ```ts
 await readLockedRustToolchain(process.cwd(),);
 // 'nightly-2026-11-07'
 ```
 */
export function readLockedRustToolchain(startDirectory: string,): Promise<string> {
  return readFromNearestLock({
    directory: startDirectory,
    startDirectory,
  },);
}
