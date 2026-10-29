/**
 Store discovery: explicit path, repo-root anchor, then a bounded walk.

 Discovery never leaves the caller's home directory when the working
 directory sits under it, never leaves the containing repository when one is
 found, and otherwise stops at the filesystem root. Repository roots come
 from `@monochromatic-dev/fs-path-core` markers rather than a second
 walker.

 @module
 */

import {
  realpath,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  isAbsolute,
  join,
  parse,
  relative,
  resolve,
  sep,
} from 'node:path';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  findRoot,
  GIT_REPOSITORY,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { STORE_FILE_NAME, } from './constants.ts';
import {
  EnvLocalJsonPathError,
  EnvLocalJsonReadError,
} from './errors.ts';

//region Constants

/**
 Logger for discovery decisions; names paths only.
 */
const discoveryLogger = tagged({ tag: 'envLocalJsonDiscovery', },);

/**
 Parent-relative prefix that marks a path outside an ancestor.
 */
const PARENT_PREFIX = `..${sep}`;

/**
 Sentinel returned when no repository root matches the search start.
 */
export const NO_REPOSITORY_ROOT: unique symbol = Symbol(
  'unmatched repository root search above the starting directory',
);

/**
 Sentinel returned when discovery finds no encrypted store.
 */
export const NO_STORE_FOUND: unique symbol = Symbol(
  'unmatched encrypted store search during discovery',
);

//endregion Constants

//region Types

/**
 Options for {@link resolveStorePath}.
 */
export type ResolveStorePathOptions = {
  /**
   Authoritative store path, when the caller supplied one.
   */
  readonly path?: string;

  /**
   Directory the bounded walk starts from; defaults to `process.cwd()`.
   */
  readonly cwd?: string;

  /**
   Home directory bounding the walk.
   */
  readonly home: string;

  /**
   Aborting stops discovery at the next step boundary.
   */
  readonly signal?: AbortSignal;
};

//endregion Types

//region Predicates

/**
 Whether a path exists as a regular file.

 @param path - candidate path

 @returns `true` when the path stats as a regular file

 @example
 ```ts
 await isRegularFile('/repo/.env.local.json');
 ```
 */
async function isRegularFile({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    /**
     Metadata of the candidate path.
     */
    const metadata = await stat(path,);
    return metadata.isFile();
  }
  catch (error) {
    discoveryLogger.debug(`path ${path} is not a regular file: ${caughtValueText(error,)}`,);
    return false;
  }
}

/**
 Whether an ancestor directory contains a candidate path, segment-aware.

 @param ancestor - directory expected to contain the candidate

 @param candidate - path tested against the ancestor

 @returns `true` when candidate equals or sits below ancestor

 @example
 ```ts
 isAncestorOrEqual({ ancestor: '/repo', candidate: '/repo/a' }); // true
 ```
 */
export function isAncestorOrEqual({
  ancestor,
  candidate,
}: {
  readonly ancestor: string;
  readonly candidate: string;
},): boolean {
  if (candidate === ancestor)
    return true;
  /**
   Candidate relative to the ancestor.
   */
  const rest = relative(
    ancestor,
    candidate,
  );
  return (rest !== '..')
    && (!rest.startsWith(PARENT_PREFIX,))
    && (!isAbsolute(rest,));
}

//endregion Predicates

//region Repository roots

/**
 Finds the nearest ancestor that is a Git repository or a mise monorepo.

 Both markers are probed concurrently; when both match at different depths,
 the deeper root wins, matching "nearest repository root".

 @param cwd - directory to start from

 @returns nearest repository root, or {@link NO_REPOSITORY_ROOT}

 @example
 ```ts
 const root = await findNearestRepositoryRoot({ cwd: process.cwd() });
 ```
 */
async function findNearestRepositoryRoot({
  cwd,
}: {
  readonly cwd: string;
},): Promise<string | typeof NO_REPOSITORY_ROOT> {
  /**
   Settled results of the two repository-root probes.
   */
  const probes = await Promise.allSettled([
    findRoot({
      cwd,
      marker: GIT_REPOSITORY,
    },),
    findRoot({
      cwd,
      marker: MISE_MONOREPO,
    },),
  ],);

  /**
   Roots found by whichever probes succeeded.
   */
  const roots: string[] = [];
  for (const probe of probes) {
    if (probe.status === 'fulfilled')
      roots.push(probe.value,);
  }

  if (roots.length === 0)
    return NO_REPOSITORY_ROOT;

  return roots.reduce(function deeperRoot(
    current: string,
    candidate: string,
  ): string {
    return candidate.length > current.length ? candidate : current;
  },);
}

//endregion Repository roots

//region Bounded walk

/**
 Canonicalizes a directory path, rejecting non-directories.

 @param path - directory path to canonicalize

 @returns real path of the directory

 @throws EnvLocalJsonPathError when the path is missing or not a directory

 @example
 ```ts
 const cwd = await canonicalDirectory({ path: '/repo' });
 ```
 */
async function canonicalDirectory({ path, }: { readonly path: string; },): Promise<string> {
  try {
    /**
     Canonical form of the candidate directory.
     */
    const canonical = await realpath(path,);
    /**
     Metadata of the canonical directory.
     */
    const metadata = await stat(canonical,);
    if (!metadata.isDirectory()) {
      throw new EnvLocalJsonPathError({
        path,
        reason: 'not-a-directory',
      },);
    }
    return canonical;
  }
  catch (error) {
    if (error instanceof EnvLocalJsonPathError)
      throw error;
    throw new EnvLocalJsonPathError({
      path,
      reason: 'missing',
    },);
  }
}

/**
 Computes the deepest walk bound: the repository root, home directory, or
 filesystem root, whichever is nearest to the start directory.

 @param cwd - canonical start directory

 @param home - canonical home directory

 @param repositoryRoot - nearest repository root, or {@link NO_REPOSITORY_ROOT}

 @returns nearest ancestor that bounds the walk

 @example
 ```ts
 const bound = walkBound({ cwd, home, repositoryRoot });
 ```
 */
function walkBound({
  cwd,
  home,
  repositoryRoot,
}: {
  readonly cwd: string;
  readonly home: string;
  readonly repositoryRoot: string | typeof NO_REPOSITORY_ROOT;
},): string {
  /**
   Bounds that may stop the walk.
   */
  const bounds: string[] = [parse(cwd,)
    .root,];
  if (isAncestorOrEqual({
    ancestor: home,
    candidate: cwd,
  },)) {
    bounds.push(home,);
  }
  if ((typeof repositoryRoot) !== 'symbol')
    bounds.push(repositoryRoot,);

  return bounds.reduce(function nearestBound(
    current: string,
    candidate: string,
  ): string {
    return candidate.length > current.length ? candidate : current;
  },);
}

/**
 Walks from `dir` up to `bound`, returning the first store file found.

 @param cwd - canonical start directory

 @param dir - directory currently inspected

 @param bound - nearest ancestor allowed in the walk

 @param repositoryRoot - nearest repository root, or {@link NO_REPOSITORY_ROOT}

 @param signal - abort signal checked at each level

 @returns canonical store path, or {@link NO_STORE_FOUND}

 @example
 ```ts
 const store = await walkForStore({ cwd, dir: cwd, bound, repositoryRoot });
 ```
 */
async function walkForStore({
  cwd,
  dir,
  bound,
  repositoryRoot,
  signal,
}: {
  readonly cwd: string;
  readonly dir: string;
  readonly bound: string;
  readonly repositoryRoot: string | typeof NO_REPOSITORY_ROOT;
  readonly signal?: AbortSignal;
},): Promise<string | typeof NO_STORE_FOUND> {
  signal?.throwIfAborted();
  /**
   Candidate store path at this level.
   */
  const candidate = join(
    dir,
    STORE_FILE_NAME,
  );
  if (await isRegularFile({ path: candidate, },)) {
    /**
     Canonical store path returned to the reader.
     */
    const canonical = await canonicalStorePath({ path: candidate, },);
    if (((typeof repositoryRoot) === 'symbol')
      || (!isAncestorOrEqual({
        ancestor: repositoryRoot,
        candidate: dir,
      }))) {
      discoveryLogger.warn(
        `env store ${canonical} was found outside a repository root`,
      );
    }
    return canonical;
  }
  if (dir === bound)
    return NO_STORE_FOUND;
  /**
   Parent directory inspected next.
   */
  const parent = dirname(dir,);
  if (parent === dir)
    return NO_STORE_FOUND;
  // Bounded structural walk: one level per call, ending at the bound or the
  // filesystem root, so the recursion depth is the path depth.
  return walkForStore({
    bound,
    cwd,
    dir: parent,
    repositoryRoot,
    ...(signal === undefined ? {} : { signal, }),
  },);
}

//endregion Bounded walk

//region Store path resolution

/**
 Canonicalizes an explicit store path and requires it to be a regular file.

 @param path - explicitly supplied store path

 @param cwd - directory a relative explicit path resolves against

 @returns canonical store file path

 @throws EnvLocalJsonPathError when the path is missing, not a file, or
 cannot be resolved

 @example
 ```ts
 const store = await canonicalExplicitStorePath({ path: '.env.local.json', cwd });
 ```
 */
async function canonicalExplicitStorePath({
  path,
  cwd,
}: {
  readonly path: string;
  readonly cwd: string;
},): Promise<string> {
  /**
   Absolute explicit store path, resolved against the caller's directory.
   */
  const absolute = isAbsolute(path,)
    ? path
    : resolve(
      cwd,
      path,
    );
  try {
    /**
     Canonical form of the explicit store path.
     */
    const canonical = await realpath(absolute,);
    /**
     Metadata of the canonical path.
     */
    const metadata = await stat(canonical,);
    if (!metadata.isFile()) {
      throw new EnvLocalJsonPathError({
        path,
        reason: 'not-a-file',
      },);
    }
    return canonical;
  }
  catch (error) {
    if (error instanceof EnvLocalJsonPathError)
      throw error;
    throw new EnvLocalJsonPathError({
      path,
      reason: 'missing',
    },);
  }
}

/**
 Returns the canonical store path, or throws a read error when it vanishes.

 @param path - discovered store path

 @returns canonical store path

 @throws EnvLocalJsonReadError when the discovered file cannot be resolved

 @example
 ```ts
 const store = await canonicalStorePath({ path });
 ```
 */
async function canonicalStorePath({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    discoveryLogger.error(
      `discovered env store ${path} disappeared before reading: ${caughtValueText(error,)}`,
    );
    throw new EnvLocalJsonReadError({ path, },);
  }
}

/**
 Resolves the store path without reading it.

 Explicit paths are authoritative and never fall through to discovery.
 Otherwise the nearest repository root containing a store wins, then a
 bounded upward walk from the canonical working directory.

 @param path - explicit store path, when supplied

 @param cwd - working directory; defaults to `process.cwd()`

 @param home - home directory bounding the walk

 @param signal - abort signal checked during discovery

 @returns canonical store path, or {@link NO_STORE_FOUND}

 @throws EnvLocalJsonPathError when an explicit path is missing or unreadable,
 or the working directory cannot be canonicalized

 @throws EnvLocalJsonReadError when a discovered store vanishes mid-check

 @example
 ```ts
 const store = await resolveStorePath({ cwd, home });
 ```
 */
export async function resolveStorePath({
  path,
  cwd,
  home,
  signal,
}: ResolveStorePathOptions,): Promise<string | typeof NO_STORE_FOUND> {
  if (path !== undefined) {
    return canonicalExplicitStorePath({
      cwd: cwd ?? process.cwd(),
      path,
    },);
  }
  return discoverStorePath({
    cwd: await canonicalDirectory({ path: cwd ?? process.cwd(), },),
    home: await canonicalDirectory({ path: home, },),
    ...(signal === undefined ? {} : { signal, }),
  },);
}

/**
 Discovery without an explicit path: repository anchor, then bounded walk.

 @param cwd - canonical working directory

 @param home - canonical home directory

 @param signal - abort signal checked during discovery

 @returns canonical store path, or {@link NO_STORE_FOUND}

 @example
 ```ts
 const store = await discoverStorePath({ cwd, home });
 ```
 */
async function discoverStorePath({
  cwd,
  home,
  signal,
}: {
  readonly cwd: string;
  readonly home: string;
  readonly signal?: AbortSignal;
},): Promise<string | typeof NO_STORE_FOUND> {
  /**
   Store found by the repository-root anchor, when one exists.
   */
  const anchored = await repositoryAnchoredStore({
    start: cwd,
    ...(signal === undefined ? {} : { signal, }),
  },);
  if ((typeof anchored) !== 'symbol')
    return anchored;

  /**
   Nearest repository root below the working directory, used both as the
   walk bound and as the "inside a repository" warning test.
   */
  const repositoryRoot = await findNearestRepositoryRoot({ cwd, },);
  /**
   Nearest ancestor that bounds the walk.
   */
  const bound = walkBound({
    cwd,
    home,
    repositoryRoot,
  },);
  discoveryLogger.debug(`bounded env store walk from ${cwd} to ${bound}`,);
  return walkForStore({
    bound,
    cwd,
    dir: cwd,
    repositoryRoot,
    ...(signal === undefined ? {} : { signal, }),
  },);
}

/**
 Looks for a store at the nearest repository root, then at each outer
 repository root.

 @param start - directory whose nearest repository root is tested

 @param signal - abort signal checked at each repository root

 @returns canonical store path, or {@link NO_STORE_FOUND}

 @example
 ```ts
 const store = await repositoryAnchoredStore({ start: process.cwd() });
 ```
 */
async function repositoryAnchoredStore({
  start,
  signal,
}: {
  readonly start: string;
  readonly signal?: AbortSignal;
},): Promise<string | typeof NO_STORE_FOUND> {
  signal?.throwIfAborted();
  /**
   Nearest repository root at or above the current start.
   */
  const repositoryRoot = await findNearestRepositoryRoot({ cwd: start, },);
  if ((typeof repositoryRoot) === 'symbol')
    return NO_STORE_FOUND;
  /**
   Candidate store at the repository root.
   */
  const candidate = join(
    repositoryRoot,
    STORE_FILE_NAME,
  );
  if (await isRegularFile({ path: candidate, },)) {
    discoveryLogger.debug(`using repository-root store ${candidate}`,);
    return canonicalStorePath({ path: candidate, },);
  }
  /**
   Parent of this repository root, searched for an outer repository.
   */
  const nextStart = dirname(repositoryRoot,);
  if (nextStart === start)
    return NO_STORE_FOUND;
  return repositoryAnchoredStore({
    start: nextStart,
    ...(signal === undefined ? {} : { signal, }),
  },);
}

//endregion Store path resolution
