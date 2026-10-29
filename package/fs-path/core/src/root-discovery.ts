/**
 Upward-walk root discovery: the interface of this package's root finding.

 `findRoot` walks from a start directory toward the filesystem root and
 returns the first ancestor a marker accepts; `findRootCached` memoises
 that walk per marker and start directory for the process lifetime. The
 filesystem comes from the `#root-filesystem` import map entry
 (`node:fs/promises` under the `node` condition, the origin private file
 system otherwise) unless the caller passes one, so this module names no
 platform API itself.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { dirname, } from '#posix-path';
import {
  defaultRootSearchCwd,
  resolveRootFilesystem,
} from '#root-filesystem';
import type { RootFilesystem, } from './root-filesystem-contract.ts';
import type { RootMarker, } from './root-marker-contract.ts';

//region Types

/**
 Options for {@link findRoot}.
 */
export type FindRootOptions = {
  /**
   Kind of root to look for.
   */
  readonly marker: RootMarker;

  /**
   Starting directory. Defaults to the process working directory, or `/`
   where no process exists.
   */
  readonly cwd?: string;

  /**
   Filesystem to walk. Defaults to the runtime adapter; pass
   `createMemoryRootFilesystem(...)` to walk a tree held in memory.
   */
  readonly fs?: RootFilesystem;
};

/**
 Options for {@link findRootCached}: the same as {@link findRoot} without a
 filesystem, because the memo belongs to the runtime adapter.
 */
export type FindRootCachedOptions = {
  /**
   Kind of root to look for; its `name` is half of the memo key.
   */
  readonly marker: RootMarker;

  /**
   Starting directory, resolved at call time to the runtime default when
   omitted; the other half of the memo key.
   */
  readonly cwd?: string;
};

/**
 Options for the internal upward walk.
 */
type WalkUpOptions = {
  /**
   Directory currently being tested.
   */
  readonly dir: string;

  /**
   Filesystem shared by every level.
   */
  readonly fs: RootFilesystem;

  /**
   Marker whose probe runs at each ancestor.
   */
  readonly marker: RootMarker;

  /**
   Directory the walk began at, reported when no ancestor matches.
   */
  readonly startDir: string;
};

//endregion Types

//region Error

/**
 Raised when no ancestor of the start directory satisfies the marker.

 @example
 ```ts
 try {
   await findRoot({ cwd, marker: GIT_REPOSITORY });
 }
 catch (error) {
   if (error instanceof RootNotFoundError) console.log(error.marker, error.startDir);
 }
 ```
 */
export class RootNotFoundError extends Error {
  /**
   Name of the marker that matched no ancestor.
   */
  public readonly marker: string;

  /**
   Directory the walk began at.
   */
  public readonly startDir: string;

  /**
   Creates the error for one failed walk.

   @param marker - name of the marker that matched no ancestor

   @param startDir - directory the walk began at

   @example
   ```ts
   throw new RootNotFoundError({ marker: 'git repository', startDir: '/tmp/x' });
   ```
   */
  public constructor({
    marker,
    startDir,
  }: {
    readonly marker: string;
    readonly startDir: string;
  },) {
    super(`no ${marker} root found walking up from ${startDir}`,);
    this.name = 'RootNotFoundError';
    this.marker = marker;
    this.startDir = startDir;
  }
}

//endregion Error

//region Constants

/**
 Tagged logger for root discovery diagnostics.
 */
const rootDiscoveryLogger = tagged({ tag: 'rootDiscovery', },);

/**
 Process-lifetime memo for {@link findRootCached}, keyed by marker name and
 start directory. Holds the in-flight promise so concurrent first callers
 share one walk, and keeps rejections because the tree does not change
 during a process lifetime.
 */
const walkMemo = new Map<string, Promise<string>>();

/**
 Separator between the two halves of a memo key; a NUL never appears in a
 marker name or a path.
 */
const MEMO_KEY_SEPARATOR = '\0';

//endregion Constants

//region Walk

/**
 Start directory for a walk: the caller's `cwd`, or the runtime default
 resolved now, not at module load.

 @param cwd - caller-provided start directory

 @returns directory the walk begins at

 @example
 ```ts
 const startDir = startDirectory(undefined);
 ```
 */
function startDirectory(cwd?: string,): string {
  return cwd ?? defaultRootSearchCwd();
}

/**
 Walks upward from a candidate directory until the marker accepts one.

 @param dir - candidate directory tested first

 @param fs - filesystem used for every probe

 @param marker - kind of root sought

 @param startDir - directory the walk began at

 @returns matching root directory

 @throws RootNotFoundError when the filesystem root is reached without a match

 @example
 ```ts
 const root = await walkUp({ dir: '/repo/src', fs, marker, startDir: '/repo/src' });
 ```
 */
async function walkUp({
  dir,
  fs,
  marker,
  startDir,
}: WalkUpOptions,): Promise<string> {
  if (await marker.matches({
    dir,
    fs,
  },))
    return dir;

  /**
   Parent directory inspected after the current candidate misses.
   */
  const parent = dirname(dir,);
  if (parent === dir) {
    throw new RootNotFoundError({
      marker: marker.name,
      startDir,
    },);
  }

  // Bounded structural walk: one level per call, ending at the filesystem
  // root, so the recursion depth is the path depth.
  return walkUp({
    dir: parent,
    fs,
    marker,
    startDir,
  },);
}

/**
 Finds the nearest ancestor of `cwd` (itself included) that `marker`
 accepts.

 @param marker - kind of root sought

 @param cwd - start directory; defaults to the runtime working directory

 @param fs - filesystem to walk; defaults to the runtime adapter

 @returns matching root directory, spelled as the caller spelled `cwd`

 @throws RootNotFoundError when no ancestor satisfies the marker

 @example
 ```ts
 const root = await findRoot({ cwd: import.meta.dirname, marker: GIT_REPOSITORY });
 ```

 @example
 ```ts
 const fs = createMemoryRootFilesystem({ files: { '/repo/pnpm-workspace.yaml': '' } });
 const root = await findRoot({ cwd: '/repo/a/b', fs, marker: PNPM_WORKSPACE });
 ```
 */
export async function findRoot({
  marker,
  cwd,
  fs,
}: FindRootOptions,): Promise<string> {
  /**
   Directory where the upward search starts.
   */
  const startDir = startDirectory(cwd,);
  /**
   Filesystem for this walk: the caller's, or the runtime adapter.
   */
  const walkFs = fs ?? await resolveRootFilesystem();
  /**
   Matching root, spelled with the caller's path identity.
   */
  const root = await walkUp({
    dir: startDir,
    fs: walkFs,
    marker,
    startDir,
  },);
  rootDiscoveryLogger.debug(`${marker.name} root of ${startDir} is ${root}`,);
  return root;
}

/**
 Memoised {@link findRoot} over the runtime filesystem.

 The memo key is the marker name plus the start directory, resolved at call
 time, so callers that name their directory never receive another
 directory's answer, and a walk after `process.chdir` starts from the new
 directory. Concurrent first callers share one in-flight walk; a rejection
 stays memoised for its key.

 @param marker - kind of root sought; its `name` keys the memo

 @param cwd - start directory; defaults to the runtime working directory

 @returns matching root directory, locked in per key for the process lifetime

 @throws RootNotFoundError same rejection as the first walk for this key

 @example
 ```ts
 const root = await findRootCached({ marker: MISE_MONOREPO });
 ```
 */
export function findRootCached({
  marker,
  cwd,
}: FindRootCachedOptions,): Promise<string> {
  /**
   Directory where the upward search starts, fixed now for the key.
   */
  const startDir = startDirectory(cwd,);
  /**
   Memo key for this marker and start directory.
   */
  const key = `${marker.name}${MEMO_KEY_SEPARATOR}${startDir}`;
  /**
   In-flight or settled walk from a prior call with the same key.
   */
  const existing = walkMemo.get(key,);
  if (existing !== undefined)
    return existing;
  /**
   Fresh walk, stored before it settles so concurrent callers share it.
   */
  const walking = findRoot({
    cwd: startDir,
    marker,
  },);
  walkMemo.set(
    key,
    walking,
  );
  return walking;
}

//endregion Walk
