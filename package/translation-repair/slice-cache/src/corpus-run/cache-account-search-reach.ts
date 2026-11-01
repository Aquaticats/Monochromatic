import { realpath, } from 'node:fs/promises';
import {
  basename,
  dirname,
  isAbsolute,
  join,
  relative,
  sep,
} from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  RUNS_SEARCH_DEPTH,
  searchEnters,
} from './cache-account-slices.ts';

//region Search reach
// WHETHER A SEARCH FOR RUNS DIRECTORIES WOULD FIND THE RUNS OF A DIRECTORY, for the audit's warning that a launch which
// follows the runbook writes where the roots searched do not look. Two spellings of one directory (the home directory
// and a link to it) are one directory, and a root reaches only as far down as the walk goes. The readings that follow
// the walk (the runbook's runs root once, then each searched root against it) live here with it, since a refusal of the
// filesystem to resolve either is what they turn into a reading instead of an end of the audit.

/**
 The real path of a directory, or of the nearest ancestor that exists with the missing names kept after it.

 A BOUNDED STRUCTURAL WALK up the path, one level per call, so the number of names in the path is also the stack bound.

 THE `..` OF A PATH IS LEFT TO THE FILESYSTEM. Folding it away by name first reads `base/lnk/../d` as `base/d`,
 where the kernel, and so the walk's own listing, reads the parent of `lnk`'s target plus `d`.

 A RELATIVE PATH IS JOINED ONTO THE WORKING DIRECTORY AS WRITTEN FIRST. `realpath` would resolve it against that
 directory itself and give the same answers and codes, but the text of its failure would name the relative path, and
 the audit logs that text: a reader of the line could not tell which directory a relative root was read against. The
 join is plain concatenation, which leaves `..` to the filesystem like the rest of the path.

 @param path - directory to resolve, which need not exist

 @returns Absolute path with every link on the way to the existing part resolved

 @throws What `realpath` raises for any failure but an absent path, such as a loop of links or a denied permission,
 and what `process.cwd` raises where the working directory was removed

 @example
 ```ts
 const real = await realPathOf({ path: '/cats/home/temp/agent', },); // '/cats/home/temp/agent' where none of it exists
 ```
 */
async function realPathOf({ path, }: { readonly path: string; },): Promise<string> {
  // THE JOIN IS POSIX-ONLY: on Windows a drive-relative path such as `C:foo` is not absolute, yet the working
  // directory is not what it is relative to, so this join would put the working directory in front of the drive. That
  // cannot arise in this module: `searchReaches` takes both of its paths to share one filesystem root, as the paths
  // under a home directory on a POSIX system do, and compares no pair on two drives.
  /**
   The path as the lookup is asked for it, whole from the filesystem root.
   */
  const asked = isAbsolute(path,)
    ? path
    : `${process.cwd()}${sep}${path}`;
  try {
    return await realpath(asked,);
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    /**
     The parent directory, which has the same answer to give for the part that exists.
     */
    const parent = dirname(asked,);
    // THE STOP OF THE WALK, NOT A CASE: the parent of the filesystem root is the root itself; a working directory that
    // does not exist is no machine this runs on, so this return only keeps the recursion finite.
    if (parent === asked)
      return asked;
    return join(
      await realPathOf({ path: parent, },),
      basename(asked,),
    );
  }
}

/**
 Whether a search that starts at a root would find the runs directories that sit up to a number of levels below another
 directory.

 THE ROOT AND THE DIRECTORY ARE COMPARED BY REAL PATH, since the home directory of the process and its real place are
 both spellings of one directory. THE WALK STOPS `RUNS_SEARCH_DEPTH` LEVELS BELOW A ROOT, so a root holding the
 directory farther up than the runs' own depth allows leaves the deepest runs unread, and it enters no hidden directory
 and no dependency tree, so a directory behind one is not reached at all. THE ROOT AND THE DIRECTORY ARE TAKEN TO SHARE
 ONE FILESYSTEM ROOT, as the paths under a home directory on a POSIX system do; a pair of paths on two drives is not
 compared.

 @param root - directory the search starts at

 @param directory - directory whose runs are asked about

 @param runsLevels - how many levels below `directory` its runs directories sit, at the most

 @returns Whether `directory` is the root or lies below it, and the deepest runs of it are within the walk

 @throws What `realpath` raises for any failure but an absent path, such as a path through a file (`ENOTDIR`), a loop
 of links (`ELOOP`) or a directory nobody may enter (`EACCES`), which the caller decides what to make of; and, where
 `root` or `directory` is relative, what `process.cwd` raises when the working directory was removed (`ENOENT`)

 @example
 ```ts
 const reached = await searchReaches({
   root: '/cats/home',
   directory: '/cats/home/temp/agent',
   runsLevels: 2,
 },); // true
 ```
 */
export async function searchReaches(
  {
    root,
    directory,
    runsLevels,
  }: {
    readonly root: string;
    readonly directory: string;
    readonly runsLevels: number;
  },
): Promise<boolean> {
  /**
   The root by its real path.
   */
  const from = await realPathOf({ path: root, },);
  /**
   The directory by its real path.
   */
  const to = await realPathOf({ path: directory, },);
  if (from === to)
    return runsLevels <= RUNS_SEARCH_DEPTH;
  /**
   The way down from the root to the directory.
   */
  const way = relative(
    from,
    to,
  );
  if (way === '..')
    return false;
  if (way.startsWith(`..${sep}`,))
    return false;
  /**
   The names the search would have to enter on the way down.
   */
  const names = way.split(sep,);
  if (!names.every(function isEntered(name,): boolean {
    return searchEnters({ name, },);
  },))
    return false;
  return (names.length + runsLevels) <= RUNS_SEARCH_DEPTH;
}

/**
 How many levels below the runbook's runs root its runs directories sit, at the most: `<root>/<run>` and
 `<root>/<run>/runs`.
 */
export const RUNBOOK_RUNS_LEVELS = 2;

/**
 A searched root whose real path could not be resolved, and the filesystem's code.

 @example
 ```ts
 const unresolved: UnresolvedRoot = { root: '/cats/file/below', code: 'ENOTDIR', };
 ```
 */
export type UnresolvedRoot = {
  /**
   The root as it was given.
   */
  readonly root: string;

  /**
   Filesystem code, such as `ENOTDIR`, or the class name of a failure that carries none.
   */
  readonly code: string;
};

/**
 A searched root that reaches the runbook's runs root.

 @example
 ```ts
 const reached: ReachedRoot = { kind: 'reached', };
 ```
 */
type ReachedRoot = {
  /**
   Which reading this is.
   */
  readonly kind: 'reached';
};

/**
 A searched root that was resolved and does not reach the runbook's runs root.

 @example
 ```ts
 const notReached: NotReachedRoot = { kind: 'not-reached', };
 ```
 */
type NotReachedRoot = {
  /**
   Which reading this is.
   */
  readonly kind: 'not-reached';
};

/**
 A searched root that could not be resolved, so it counts as not reaching the runbook's runs root.

 @example
 ```ts
 const unresolved: UnresolvedRootReading = { kind: 'unresolved', root: '/cats/sealed/below', code: 'EACCES', };
 ```
 */
type UnresolvedRootReading = UnresolvedRoot & {
  /**
   Which reading this is.
   */
  readonly kind: 'unresolved';
};

/**
 What the check found for one searched root.

 @example
 ```ts
 const reading: RootReading = { kind: 'unresolved', root: '/cats/sealed/below', code: 'EACCES', };
 ```
 */
export type RootReading = NotReachedRoot | ReachedRoot | UnresolvedRootReading;

/**
 The runbook's runs root resolved.

 @example
 ```ts
 const resolved: ResolvedRunbookRoot = { kind: 'resolved', real: '/cats/home/temp/agent', };
 ```
 */
type ResolvedRunbookRoot = {
  /**
   Which reading this is.
   */
  readonly kind: 'resolved';

  /**
   Its real path.
   */
  readonly real: string;
};

/**
 The runbook's runs root could not be resolved.

 @example
 ```ts
 const unresolved: UnresolvedRunbookRoot = { kind: 'unresolved', code: 'ENOTDIR', };
 ```
 */
type UnresolvedRunbookRoot = {
  /**
   Which reading this is.
   */
  readonly kind: 'unresolved';

  /**
   Filesystem code of the failure, or its class name where it carries none.
   */
  readonly code: string;
};

/**
 What the check found for the runbook's runs root.

 @example
 ```ts
 const reading: RunbookRootReading = { kind: 'resolved', real: '/cats/home/temp/agent', };
 ```
 */
export type RunbookRootReading = ResolvedRunbookRoot | UnresolvedRunbookRoot;

/**
 Resolves the runbook's runs root once, so a failure of that root is named once.

 @param runbookRoot - where a launch that follows the runbook writes its runs

 @param l - logger the refusal is written to

 @returns Its real path, or that it could not be resolved and why

 @example
 ```ts
 const reading = await runbookRootReading({ runbookRoot: '/cats/home/temp/agent', l, },);
 ```
 */
export async function runbookRootReading(
  {
    runbookRoot,
    l,
  }: {
    readonly runbookRoot: string;
    readonly l: Logger;
  },
): Promise<RunbookRootReading> {
  /**
   Logger tagged with this function's name.
   */
  const rl = tagged({
    l,
    tag: runbookRootReading.name,
  },);
  try {
    return {
      kind: 'resolved',
      real: await realPathOf({ path: runbookRoot, },),
    };
  }
  catch (error) {
    /**
     The filesystem's code for the refusal, which the warning on stdout names.
     */
    const code = failureName({ error, },);
    // THE LOG LINE KEEPS THE FAILURE'S TEXT, as the line of a root that cannot be resolved does, and for the same
    // reason: the runbook's runs root is printed in the line, and the text names the path the lookup stopped at.
    rl.warn(
      `the runbook's runs root ${runbookRoot} could not be resolved (${code}); no searched root can be compared `
        + `with it. The failure was: ${caughtValueText(error,)}`,
    );
    return {
      kind: 'unresolved',
      code,
    };
  }
}

/**
 Whether one searched root reaches the runbook's runs root, a root the filesystem refuses to resolve counting as not.

 THE REFUSAL DOES NOT END THE AUDIT. The walk reports a directory it cannot list and goes on, and a root named on the
 command line can sit behind a file, a loop of links or a directory the process may not enter; the check logs the
 failure with the root and says so in the warning, so the newest record and the versions set after it are still
 printed.

 @param root - searched root

 @param runbookRoot - where a launch that follows the runbook writes its runs, as the warning names it

 @param runbookReal - real path of that root, resolved once by the caller

 @param l - logger the refusal is written to

 @returns That the root reaches it, that it does not, or that it could not be resolved and why

 @example
 ```ts
 const reading = await reachOfRoot({
   root: '/cats/runs',
   runbookRoot: '/cats/home/temp/agent',
   runbookReal: '/cats/home/temp/agent',
   l,
 },);
 ```
 */
export async function reachOfRoot(
  {
    root,
    runbookRoot,
    runbookReal,
    l,
  }: {
    readonly root: string;
    readonly runbookRoot: string;
    readonly runbookReal: string;
    readonly l: Logger;
  },
): Promise<RootReading> {
  /**
   Logger tagged with this function's name.
   */
  const rl = tagged({
    l,
    tag: reachOfRoot.name,
  },);
  try {
    if (await searchReaches({
      root,
      directory: runbookReal,
      runsLevels: RUNBOOK_RUNS_LEVELS,
    },))
      return { kind: 'reached', };
    return { kind: 'not-reached', };
  }
  catch (error) {
    /**
     The filesystem's code for the refusal, which the warning on stdout names.
     */
    const code = failureName({ error, },);
    // THE LOG LINE KEEPS THE FAILURE'S TEXT, which the warning on stdout leaves out: the text of a failed real-path
    // lookup names the path it stopped at, an ancestor of the root, and so tells which file or link on the way is to
    // blame. The line already prints the root and the runbook's runs root, so the text discloses no path the line
    // does not hold. The logger takes one string, so the caught value is rendered into it.
    rl.warn(
      `${root} could not be compared with the runbook's runs root ${runbookRoot} (${code}); `
        + `it counts as not reaching it. The failure was: ${caughtValueText(error,)}`,
    );
    return {
      kind: 'unresolved',
      root,
      code,
    };
  }
}

//endregion Search reach
