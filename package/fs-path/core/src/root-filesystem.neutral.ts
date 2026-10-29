/**
 Root-discovery filesystem for runtimes without `node:fs`, selected by
 `package.json` `imports` under the `default` condition of
 `#root-filesystem`: the origin private file system (OPFS) through
 `navigator.storage.getDirectory()` where the platform exposes it, and an
 empty backend that never finds a marker everywhere else. No `node:`
 specifier and no dynamic import: this module is bundled only into the
 neutral artifact.

 Paths are absolute POSIX strings rooted at the OPFS root directory, so
 `/repo/mise.toml` names the file `mise.toml` in the directory `repo`
 under `navigator.storage.getDirectory()`.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { resolve, } from '#posix-path';
import {
  ABSENT,
  type RootFilesystem,
  unsupportedReadSymbolicLink,
} from './root-filesystem-contract.ts';

//region Shared

/**
 Tagged logger for backend selection diagnostics.
 */
const neutralFilesystemLogger = tagged({ tag: 'rootFilesystem', },);

/**
 Splits an absolute POSIX path into its non-empty segments after
 normalization, so `..` and `.` never reach the handle walk.

 @param path - absolute POSIX path

 @returns directory and file names from the root downward

 @example
 ```ts
 pathSegments('/repo/pkg/../mise.toml'); // ['repo', 'mise.toml']
 ```
 */
function pathSegments(path: string,): readonly string[] {
  return resolve([path,],)
    .split('/',)
    .filter(function nonEmpty(segment,): boolean {
      return segment !== '';
    },);
}

//endregion Shared

//region OPFS backend

/**
 Sentinel for a path with no entry in OPFS, distinct from every handle.
 */
const NO_HANDLE = Symbol('OPFS entry absent at the requested path',);

/**
 Handle kinds the walk can land on.
 */
type OpfsHandle = FileSystemDirectoryHandle | FileSystemFileHandle;

/**
 Walks the OPFS directory tree to the handle at an absolute path.

 Every segment but the last must be a directory; the last may be a
 directory or a file, tried in that order. A missing entry, a file where a
 directory was needed, or any platform refusal maps to {@link NO_HANDLE}:
 for root discovery every one of those means "no marker here".

 @param root - OPFS root directory

 @param path - absolute POSIX path to reach

 @returns handle at the path, or {@link NO_HANDLE}

 @example
 ```ts
 const handle = await opfsHandleAt({ root, path: '/repo/mise.toml' });
 ```
 */
async function opfsHandleAt({
  root,
  path,
}: {
  readonly root: FileSystemDirectoryHandle;
  readonly path: string;
},): Promise<OpfsHandle | typeof NO_HANDLE> {
  /**
   Names from the OPFS root down to the target.
   */
  const segments = pathSegments(path,);
  /**
   Parent directory names, every one of which must resolve to a directory.
   */
  const parents = segments.slice(
    0,
    -1,
  );
  /**
   Final name, absent when the path is the root itself.
   */
  const leaf = segments.at(-1,);
  try {
    /**
     Directory reached so far; starts at the root.
     */
    let current = root;
    // Handles resolve one level at a time: each getDirectoryHandle needs
    // the handle the previous one produced, so the awaits are sequential
    // by construction.
    for (const name of parents)
      // oxlint-disable-next-line no-await-in-loop -- each handle depends on the previous one
      current = await current.getDirectoryHandle(name,);
    if (leaf === undefined)
      return current;
    try {
      return await current.getDirectoryHandle(leaf,);
    }
    catch (directoryError: unknown) {
      neutralFilesystemLogger.debug(`${path} is not a directory in OPFS: ${caughtValueText(directoryError,)}`,);
      return await current.getFileHandle(leaf,);
    }
  }
  catch (error: unknown) {
    neutralFilesystemLogger.debug(`no OPFS entry at ${path}: ${caughtValueText(error,)}`,);
    return NO_HANDLE;
  }
}

/**
 Builds the root-discovery filesystem over one OPFS root directory.

 @param root - OPFS root directory

 @returns backend whose probes walk from the root

 @example
 ```ts
 const fs = opfsRootFilesystem({ root: await navigator.storage.getDirectory() });
 ```
 */
function opfsRootFilesystem({ root, }: { readonly root: FileSystemDirectoryHandle; },): RootFilesystem {
  return {
    exists: async function opfsExists(path: string,): Promise<boolean> {
      return (await opfsHandleAt({
        path,
        root,
      },)) !== NO_HANDLE;
    },

    isDirectory: async function opfsIsDirectory(path: string,): Promise<boolean> {
      /**
       Handle at the path, if any.
       */
      const handle = await opfsHandleAt({
        path,
        root,
      },);
      return (handle !== NO_HANDLE) && (handle.kind === 'directory');
    },

    isFile: async function opfsIsFile(path: string,): Promise<boolean> {
      /**
       Handle at the path, if any.
       */
      const handle = await opfsHandleAt({
        path,
        root,
      },);
      return (handle !== NO_HANDLE) && (handle.kind === 'file');
    },

    readSymbolicLink: unsupportedReadSymbolicLink,

    readTextFile: async function opfsReadTextFile(path: string,): Promise<string | typeof ABSENT> {
      /**
       Handle at the path, if any.
       */
      const handle = await opfsHandleAt({
        path,
        root,
      },);
      if ((handle === NO_HANDLE) || (handle.kind !== 'file'))
        return ABSENT;
      return await (await handle.getFile()).text();
    },

  };
}

//endregion OPFS backend

//region Empty backend

/**
 Empty text reader used when no filesystem backend exists.

 @param _path - ignored candidate path

 @returns promise resolving to {@link ABSENT}

 @example
 ```ts
 await emptyReadTextFile('/repo/mise.toml');
 ```
 */
function emptyReadTextFile(_path: string,): Promise<typeof ABSENT> {
  return Promise.resolve(ABSENT,);
}

/**
 Empty existence probe used when no filesystem backend exists.

 @param _path - ignored candidate path

 @returns promise resolving to `false`

 @example
 ```ts
 await emptyExists('/repo/.git');
 ```
 */
function emptyExists(_path: string,): Promise<boolean> {
  return Promise.resolve(false,);
}

/**
 Root-discovery filesystem that never finds marker files, used where the
 platform exposes neither `node:fs` nor OPFS.
 */
const emptyRootFilesystem: RootFilesystem = {
  exists: emptyExists,
  isDirectory: emptyExists,
  isFile: emptyExists,
  readSymbolicLink: unsupportedReadSymbolicLink,
  readTextFile: emptyReadTextFile,
};

//endregion Empty backend

//region Selection

/**
 Backend cache, stored in a const container for module-root state.
 */
const backendCache: { fs?: RootFilesystem; } = {};

/**
 Whether this runtime exposes the origin private file system.

 @returns `true` when `navigator.storage.getDirectory` is callable

 @example
 ```ts
 if (hasOriginPrivateFileSystem()) { ... }
 ```
 */
function hasOriginPrivateFileSystem(): boolean {
  /**
   Storage manager, absent outside browsers and workers.
   */
  const storage = globalThis.navigator
    ?.storage;
  return (storage !== undefined) && ((typeof storage.getDirectory) === 'function');
}

/**
 Resolves the filesystem backend for this runtime: OPFS where the platform
 grants it, the empty backend otherwise. Cached after the first call so
 every root finder shares one backend.

 @returns backend for this runtime

 @example
 ```ts
 const fs = await resolveRootFilesystem();
 ```
 */
export async function resolveRootFilesystem(): Promise<RootFilesystem> {
  if (backendCache.fs !== undefined)
    return backendCache.fs;
  if (hasOriginPrivateFileSystem()) {
    try {
      /**
       OPFS root directory for this origin.
       */
      const root = await navigator.storage
        .getDirectory();
      neutralFilesystemLogger.warn(
        'using the origin private file system for root discovery: marker files must exist there to be found',
      );
      backendCache.fs = opfsRootFilesystem({ root, },);
      return backendCache.fs;
    }
    catch (error: unknown) {
      // Headless WebKit exposes getDirectory and refuses the call.
      neutralFilesystemLogger.warn(`origin private file system refused: ${caughtValueText(error,)}`,);
    }
  }
  neutralFilesystemLogger.warn('no filesystem available for root discovery; search will fail',);
  backendCache.fs = emptyRootFilesystem;
  return backendCache.fs;
}

/**
 Returns the default root-search start directory for this runtime: the
 process working directory where a `process.cwd` function exists (Deno, or
 a Node consumer whose bundler resolved the `default` condition), otherwise
 the filesystem root.

 @returns absolute start directory

 @example
 ```ts
 const cwd = defaultRootSearchCwd();
 ```
 */
export function defaultRootSearchCwd(): string {
  if (((typeof process) !== 'undefined') && ((typeof process.cwd) === 'function'))
    return process.cwd();
  return '/';
}

//endregion Selection
