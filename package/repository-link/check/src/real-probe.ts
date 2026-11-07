/**
 The real probe: answers the check's filesystem questions from the running system, creating nothing.

 @module
 */

import {
  lstat,
  readFile,
  readlink,
  realpath,
} from 'node:fs/promises';
import type {
  EntryInfo,
  Probe,
} from './link-types.ts';
import {
  mountHolding,
  readMounts,
  uuidOfSource,
} from './mountinfo.ts';

/**
 Error codes that mean a path is absent or cannot be followed, which is an answer and not a failure.
 */
const ABSENT_CODES: readonly string[] = ['ENOENT', 'ENOTDIR', 'ELOOP',];

/**
 Tells whether an error is one of the codes that mean "not there".

 @param error - the caught value
 @returns true for an absence code
 */
function isAbsent(error: unknown,): boolean {
  return ABSENT_CODES.includes((error as NodeJS.ErrnoException).code ?? '',);
}

/**
 Looks at a path without following a final link.

 @param path - the path
 @returns what is there
 */
async function lstatEntry(path: string,): Promise<EntryInfo> {
  try {
    const info = await lstat(path,);
    if (info.isSymbolicLink()) {
      return { exists: true, kind: 'link', };
    }
    if (info.isDirectory()) {
      return { exists: true, kind: 'directory', };
    }
    return { exists: true, kind: info.isFile() ? 'file' : 'other', };
  }
  catch (error) {
    if (isAbsent(error,)) {
      return { exists: false, kind: 'missing', };
    }
    throw error;
  }
}

/**
 Resolves every link of a path.

 @param path - the path
 @returns the resolved path, or an empty string when it does not resolve
 */
async function resolvePath(path: string,): Promise<string> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    if (isAbsent(error,)) {
      return '';
    }
    throw error;
  }
}

/**
 Reads a small text file.

 @param path - the file
 @returns its text, or an empty string when it is absent
 */
async function readSmallText(path: string,): Promise<string> {
  try {
    return await readFile(path, 'utf8',);
  }
  catch (error) {
    if (isAbsent(error,)) {
      return '';
    }
    throw error;
  }
}

/**
 Finds the filesystem UUID that holds a path.

 @param path - the path, resolved here
 @returns the UUID, or an empty string
 */
async function filesystemUuid(path: string,): Promise<string> {
  const resolved = await resolvePath(path,);
  if (resolved === '') {
    return '';
  }
  return await uuidOfSource(mountHolding({ mounts: await readMounts(), path: resolved, },).source,);
}

/**
 Tells whether a path is a mount point of this process.

 @param path - the path, resolved here
 @returns true when mountinfo lists it as a mount point
 */
async function isMountPointPath(path: string,): Promise<boolean> {
  const resolved = await resolvePath(path,);
  return resolved !== '' && (await readMounts()).some(function sameMountPoint({ mountPoint, },) {
    return mountPoint === resolved;
  },);
}

/**
 The probe that reads the running system.
 */
export const realProbe: Probe = {
  lstat: lstatEntry,
  readlink,
  realpath: resolvePath,
  readText: readSmallText,
  uuidOf: filesystemUuid,
  isMountPoint: isMountPointPath,
};
