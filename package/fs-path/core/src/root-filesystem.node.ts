/**
 Root-discovery filesystem backed by `node:fs/promises`, selected by
 `package.json` `imports` under the `node` condition of `#root-filesystem`.
 Static `node:` imports: this module is bundled only into the node
 artifact, never into the neutral one.

 @module
 */

import {
  lstat,
  readFile,
  readlink,
  stat,
} from 'node:fs/promises';

import {
  ABSENT,
  type RootFilesystem,
} from './root-filesystem-contract.ts';

/**
 Error codes that mean a candidate path is absent for discovery purposes.
 */
const NO_ENTRY_ERROR_CODES: ReadonlySet<string> = new Set([
  'ENOENT',
  'ENOTDIR',
],);

/**
 Checks whether an error represents an absent candidate path.

 @param error - thrown value from filesystem operation

 @returns `true` when discovery should continue upward

 @example
 ```ts
 if (isNoEntryError(error)) return ABSENT;
 ```
 */
function isNoEntryError(error: unknown,): boolean {
  if (!Error.isError(error,))
    return false;
  if (!('code' in error))
    return false;
  if ((typeof error.code) !== 'string')
    return false;
  return NO_ENTRY_ERROR_CODES.has(error.code,);
}

/**
 Reads UTF-8 text, mapping absent paths to {@link ABSENT}.

 @param path - candidate file path

 @returns file text or {@link ABSENT}

 @example
 ```ts
 await nodeReadTextFile('/repo/mise.toml');
 ```
 */
async function nodeReadTextFile(path: string,): Promise<string | typeof ABSENT> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error: unknown) {
    if (isNoEntryError(error,))
      return ABSENT;
    throw error;
  }
}

/**
 Reads a symbolic link's target, mapping absent paths and non-links to {@link ABSENT}.

 @param path - candidate link path

 @returns link target text or {@link ABSENT}

 @example
 ```ts
 await nodeReadSymbolicLink('/repo/.git/HEAD');
 ```
 */
async function nodeReadSymbolicLink(path: string,): Promise<string | typeof ABSENT> {
  try {
    if (!(await lstat(path,)).isSymbolicLink())
      return ABSENT;
    return await readlink(path,);
  }
  catch (error: unknown) {
    if (isNoEntryError(error,))
      return ABSENT;
    throw error;
  }
}

/**
 Checks whether any entry exists at a path, links included.

 @param path - candidate path

 @returns whether an entry exists

 @example
 ```ts
 await nodeExists('/repo/pnpm-workspace.yaml');
 ```
 */
async function nodeExists(path: string,): Promise<boolean> {
  try {
    await lstat(path,);
    return true;
  }
  catch (error: unknown) {
    if (isNoEntryError(error,))
      return false;
    throw error;
  }
}

/**
 Checks whether a path resolves to a directory.

 @param path - candidate path

 @returns whether the resolved entry is a directory

 @example
 ```ts
 await nodeIsDirectory('/repo/.git');
 ```
 */
async function nodeIsDirectory(path: string,): Promise<boolean> {
  try {
    return (await stat(path,)).isDirectory();
  }
  catch (error: unknown) {
    if (isNoEntryError(error,))
      return false;
    throw error;
  }
}

/**
 Checks whether a path resolves to a regular file.

 @param path - candidate path

 @returns whether the resolved entry is a regular file

 @example
 ```ts
 await nodeIsFile('/repo/.git');
 ```
 */
async function nodeIsFile(path: string,): Promise<boolean> {
  try {
    return (await stat(path,)).isFile();
  }
  catch (error: unknown) {
    if (isNoEntryError(error,))
      return false;
    throw error;
  }
}

/**
 Filesystem backend for Node and Bun, shared by every root finder.
 */
const nodeRootFilesystem: RootFilesystem = {
  exists: nodeExists,
  isDirectory: nodeIsDirectory,
  isFile: nodeIsFile,
  readSymbolicLink: nodeReadSymbolicLink,
  readTextFile: nodeReadTextFile,
};

/**
 Resolves the filesystem backend for this runtime.

 @returns backend over `node:fs/promises`

 @example
 ```ts
 const fs = await resolveRootFilesystem();
 ```
 */
export function resolveRootFilesystem(): Promise<RootFilesystem> {
  return Promise.resolve(nodeRootFilesystem,);
}

/**
 Returns the default root-search start directory for this runtime.

 @returns process working directory

 @example
 ```ts
 const cwd = defaultRootSearchCwd();
 ```
 */
export function defaultRootSearchCwd(): string {
  return process.cwd();
}
