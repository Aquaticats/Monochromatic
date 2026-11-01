import { realpath, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { OpenSnitchConfigError, } from './errors.ts';
import {
  type PathAliasPair,
  pathSpellings,
  type ToolchainRuleRoots,
} from './opensnitch-toolchain-rule-patterns.ts';

/**
 Sentinel for a path that does not exist.
 */
const PATH_ABSENT: unique symbol = Symbol('path does not exist',);

/**
 Sentinel for a start directory with no checkout above it.
 */
export const REPOSITORY_ROOT_ABSENT: unique symbol = Symbol('wg-quicker repository root not found',);

/**
 Marker file at the repository root,
 used to find the checkout from the running bundle.
 */
const REPOSITORY_MARKER = 'toolchain.lock.json';

/**
 Path segment linked worktrees live under inside the main checkout.
 */
const WORKTREE_SEGMENT = '/.claude/worktrees/';

/**
 Resolves a path to its real location.

 @param path - Absolute path.

 @returns Real path,
 or the absence sentinel when the path or a parent is missing.

 @throws {@link OpenSnitchConfigError} when resolution fails for any other reason.

 @example
 ```ts
 await realpathOrAbsent({ path: '/home' });
 ```
 */
async function realpathOrAbsent(
  { path, }: { readonly path: string; },
): Promise<string | typeof PATH_ABSENT> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    if (Error.isError(error,) && [
      'ENOENT',
      'ENOTDIR',
    ].includes((error as NodeJS.ErrnoException).code ?? '',))
      return PATH_ABSENT;
    throw new OpenSnitchConfigError(
      `Cannot resolve ${path}`,
      { cause: error, },
    );
  }
}

/**
 Discovers the prefix pairs that name one directory on this host.

 On Fedora Atomic `/home` links to `/var/home`;
 elsewhere the pair list is empty.

 @returns Alias pairs for the `/home` prefix.

 @example
 ```ts
 await homeAliasPairs(); // [['/home', '/var/home']] on Fedora Atomic
 ```
 */
export async function homeAliasPairs(): Promise<readonly PathAliasPair[]> {
  /**
   Real location of `/home`.
   */
  const real = await realpathOrAbsent({ path: '/home', },);
  if ((real === PATH_ABSENT) || (real === '/home'))
    return [];
  return [[
    '/home',
    real,
  ],];
}

/**
 Expands one directory into the spellings a process path may use: as given, real, and each alias.

 @param path - Absolute directory.

 @param aliasPairs - Host prefix aliases.

 @returns Distinct spellings,
 requested spelling first.

 @example
 ```ts
 await directorySpellings({ path: '/home/me', aliasPairs });
 ```
 */
export async function directorySpellings(
  {
    path,
    aliasPairs,
  }: {
    readonly path: string;
    readonly aliasPairs: readonly PathAliasPair[];
  },
): Promise<readonly string[]> {
  /**
   Real path when the directory exists.
   */
  const real = await realpathOrAbsent({ path, },);
  return [
    ...new Set([
      ...pathSpellings({
        path,
        aliasPairs,
      },),
      ...(real === PATH_ABSENT ? [] : pathSpellings({
        path: real,
        aliasPairs,
      },)),
    ],),
  ];
}

/**
 Finds the main checkout containing a directory.

 Walks up from `start` to the nearest directory holding `toolchain.lock.json`,
 then drops a trailing `/.claude/worktrees/<id>` so a worktree run covers the main checkout and all its worktrees.

 @param start - Directory inside a checkout.

 @returns Main checkout root,
 or the absence sentinel when no ancestor is a checkout.

 @example
 ```ts
 await findRepositoryRoot({ start: import.meta.dirname });
 ```
 */
export async function findRepositoryRoot(
  { start, }: { readonly start: string; },
): Promise<string | typeof REPOSITORY_ROOT_ABSENT> {
  if (await realpathOrAbsent({ path: join(
    start,
    REPOSITORY_MARKER,
  ), },) !== PATH_ABSENT) {
    /**
     Index of the worktree segment when the checkout is a linked worktree.
     */
    const segment = start.indexOf(WORKTREE_SEGMENT,);
    return segment === (-1)
      ? start
      : start.slice(
        0,
        segment,
      );
  }
  /**
   Parent directory.
   */
  const parent = dirname(start,);
  if (parent === start)
    return REPOSITORY_ROOT_ABSENT;
  // The walk is bounded by the directory depth and ends at the filesystem root.
  return findRepositoryRoot({ start: parent, },);
}

/**
 Caller choices that move a covered directory away from its default.
 */
export type ToolchainRootOptions = {
  /**
   Caller home,
   the base of every default.
   */
  readonly home: string;

  /**
   XDG data home when not `<home>/.local/share`.
   */
  readonly dataHome?: string;

  /**
   Cargo home when not `<home>/.cargo`.
   */
  readonly cargoHome?: string;

  /**
   Rustup home when not `<home>/.rustup`.
   */
  readonly rustupHome?: string;

  /**
   Main checkout roots whose `.cache` holds exposure links.
   */
  readonly repositoryRoots: readonly string[];
};

/**
 Resolves every covered directory into all of its spellings.

 @param options - Home and overrides.

 @returns Roots for the rule generator.

 @example
 ```ts
 await resolveToolchainRuleRoots({ options: { home: '/home/me', repositoryRoots: [] } });
 ```
 */
export async function resolveToolchainRuleRoots(
  { options, }: { readonly options: ToolchainRootOptions; },
): Promise<ToolchainRuleRoots> {
  /**
   Host prefix aliases applied to each directory.
   */
  const aliasPairs = await homeAliasPairs();
  /**
   Spellings of the data home,
   resolved concurrently with the other directories.
   */
  const dataHomes = directorySpellings({
    path: options.dataHome ?? join(
      options.home,
      '.local',
      'share',
    ),
    aliasPairs,
  },);
  /**
   Spellings of the Cargo home.
   */
  const cargoHomes = directorySpellings({
    path: options.cargoHome ?? join(
      options.home,
      '.cargo',
    ),
    aliasPairs,
  },);
  /**
   Spellings of the rustup home.
   */
  const rustupHomes = directorySpellings({
    path: options.rustupHome ?? join(
      options.home,
      '.rustup',
    ),
    aliasPairs,
  },);
  /**
   Spellings of each requested checkout root.
   */
  const repositoryGroups = await Promise.all(options.repositoryRoots
    .map(
    function spellRoot(path,): Promise<readonly string[]> {
      return directorySpellings({
        path,
        aliasPairs,
      },);
    },
    ),);
  return {
    dataHomes: await dataHomes,
    cargoHomes: await cargoHomes,
    rustupHomes: await rustupHomes,
    repositoryRoots: [...new Set(repositoryGroups.flat(),),],
  };
}
