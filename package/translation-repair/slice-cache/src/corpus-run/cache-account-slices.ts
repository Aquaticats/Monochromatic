import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  compareCodePoints,
  textsInCodePointOrder,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  namesIn,
  presentNamesOfKind,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  RUNS_DIR_NAME,
  SLICE_CACHE_DIR,
} from '@monochromatic-dev/translation-repair-run/ts';
import { isSliceFileName, } from './slice-cache-namespace.ts';

//region Slice cache account
// THE NEWEST SLICE-CACHE RECORD, AND HOW MANY RECORDS WERE READ, for the
// cache-account audit (ledger M57, M81). Each account of a cache version said
// whether the version was set after the newest record any run had written, and
// each found that record with a hand-written `find`: one named a runs root that
// does not exist, one's time filter failed, and each printed zero, which was
// read as an answer. The count of every record read is the control: a zero
// count says the roots hold nothing, which is the first thing to check before a
// newest record is trusted.

/**
 One slice-cache record and when it was last written.

 @example
 ```ts
 const record: SliceRecord = { path: '/runs/translation-repair-runs/slice-cache/Tabby/0-1-tabby.json', modifiedMs: 0, };
 ```
 */
export type SliceRecord = {
  /**
   Absolute path, so a report names the runs directory and entry it is in
   however many runs directories share a name.
   */
  readonly path: string;

  /**
   Last modification, in milliseconds since the epoch.
   */
  readonly modifiedMs: number;
};

/**
 Newest record among those read, or that none was found.

 @example
 ```ts
 const newest: NewestSliceRecord = { kind: 'none', };
 ```
 */
export type NewestSliceRecord =
  | {
    /**
     The roots held no record.
     */
    readonly kind: 'none';
  }
    | {
    /**
     The roots held one or more records.
     */
      readonly kind: 'found';

    /**
     The one written last.
     */
      readonly record: SliceRecord;
    };

/**
 What the slice caches under some runs directories hold.

 @example
 ```ts
 const account: SliceCacheAccount = { runsDirs: [], count: 0, newest: { kind: 'none', }, };
 ```
 */
export type SliceCacheAccount = {
  /**
   Runs directories read, in code-point order.
   */
  readonly runsDirs: readonly string[];

  /**
   Records read across all of them: the control.
   */
  readonly count: number;

  /**
   The record written last.
   */
  readonly newest: NewestSliceRecord;
};

/**
 Runs directories a parent holds by the default runs directory's name, which
 the default carries whole and a hand-set one beside it carries as a prefix.

 @param parent - directory the default runs directory sits in

 @returns Their paths, in code-point order, empty where the parent is absent

 @example
 ```ts
 const runsDirs = await runsDirsIn({ parent: join(root, 'node_modules', '.monochromatic',), },);
 ```
 */
export async function runsDirsIn({ parent, }: { readonly parent: string; },): Promise<readonly string[]> {
  /**
   Directories the parent holds.
   */
  const names = await presentNamesOfKind({
    dir: parent,
    kind: 'directory',
  },);
  return textsInCodePointOrder({ texts: names
    .filter(function isRunsDir(name,): boolean {
      return name.startsWith(RUNS_DIR_NAME,);
    },), },)
    .map(function pathOf(name,): string {
      return join(
        parent,
        name,
      );
    },);
}

/**
 How many levels down from a searched directory a runs directory may sit.
 The runs the accounts read sit one or two levels down from the scratch root
 (`<root>/vub-run1-20260821`, `<root>/fidelity-native-artifacts-c0HMDl/runs`);
 four leaves room and still bounds a walk over a large tree.
 */
export const RUNS_SEARCH_DEPTH = 4;

/**
 Whether a search for runs directories enters a directory: never a hidden
 one, where a worktree-copy hook left a payload of copied records (ledger
 M81), and never a dependency tree.

 @param name - directory's name

 @returns Whether to look inside it

 @example
 ```ts
 const entered = searchEnters({ name: 'node_modules', },); // false
 ```
 */
export function searchEnters({ name, }: { readonly name: string; },): boolean {
  return (!name.startsWith('.',)) && (name !== 'node_modules');
}

/**
 A directory a search could not list, and the filesystem's reason.

 @example
 ```ts
 const skipped: UnlistedDir = { dir: '/runs/locked', reason: 'EACCES', };
 ```
 */
export type UnlistedDir = {
  /**
   The directory.
   */
  readonly dir: string;

  /**
   Filesystem code, such as `EACCES`.
   */
  readonly reason: string;
};

/**
 What a search for runs directories found, and where it could not look.

 @example
 ```ts
 const search: RunsDirSearch = { found: [], unlisted: [], };
 ```
 */
export type RunsDirSearch = {
  /**
   Runs directories found.
   */
  readonly found: readonly string[];

  /**
   Directories the search could not list, so the count leaves out whatever
   runs they hold.
   */
  readonly unlisted: readonly UnlistedDir[];
};

/**
 Runs directories at or below a directory, to a bounded depth: each directory
 holding a slice cache is one, and the search does not look inside it.

 A BOUNDED STRUCTURAL WALK, recursing once per level under the directory, so
 the depth bound is also the stack bound. A directory it cannot list is
 reported rather than raised, since a tree named on the command line can
 hold one another user made (an `EACCES` stopped the first run over the
 agents' scratch root), and the count must say what it left out.

 @param dir - directory searched

 @param levelsLeft - levels under it still searched

 @returns Runs directories found, unsorted, and directories not listed

 @example
 ```ts
 const search = await runsDirsBelow({ dir, levelsLeft: RUNS_SEARCH_DEPTH, },);
 ```
 */
async function runsDirsBelow(
  {
    dir,
    levelsLeft,
  }: {
    readonly dir: string;
    readonly levelsLeft: number;
  },
): Promise<RunsDirSearch> {
  /**
   Directories it holds; links are neither kind, so none is followed.
   */
  const reading = await namesIn({
    dir,
    kind: 'directory',
  },);
  if (reading.kind === 'unreadable') {
    return {
      found: [],
      unlisted: [{
        dir,
        reason: reading.reason,
      },],
    };
  }
  /**
   Their names.
   */
  const { names, } = reading;
  if (names.includes(SLICE_CACHE_DIR,)) {
    return {
      found: [dir,],
      unlisted: [],
    };
  }
  if (levelsLeft === 0) {
    return {
      found: [],
      unlisted: [],
    };
  }
  /**
   The search under each directory it holds.
   */
  const nested = await Promise.all(names
    .filter(function entered(name,): boolean {
      return searchEnters({ name, },);
    },)
    .map(async function searchUnder(name,): Promise<RunsDirSearch> {
      return await runsDirsBelow({
        dir: join(
          dir,
          name,
        ),
        levelsLeft: levelsLeft - 1,
      },);
    },),);
  return {
    found: nested.flatMap(function foundIn({ found, },): readonly string[] {
      return found;
    },),
    unlisted: nested.flatMap(function unlistedIn({ unlisted, },): readonly UnlistedDir[] {
      return unlisted;
    },),
  };
}

/**
 Runs directories under a directory named on the command line: every
 directory holding a slice cache, up to four levels down, outside hidden
 directories and dependency trees, which is where the accounts' hand-written
 searches found the runs a pass under `TRANSLATION_REPAIR_RUNS_DIR` left.

 @param root - directory to search

 @returns Runs directories found and directories not listed, each in
 code-point order

 @example
 ```ts
 const { found, unlisted, } = await runsDirsUnder({ root: join(homedir(), 'temp', 'agent',), },);
 ```
 */
export async function runsDirsUnder({ root, }: { readonly root: string; },): Promise<RunsDirSearch> {
  /**
   The search, unsorted.
   */
  const {
    found,
    unlisted,
  } = await runsDirsBelow({
    dir: root,
    levelsLeft: RUNS_SEARCH_DEPTH,
  },);
  return {
    found: textsInCodePointOrder({ texts: found, },),
    unlisted: unlisted.toSorted(function byDir(
      left,
      right,
    ): number {
      return compareCodePoints({
        left: left.dir,
        right: right.dir,
      },);
    },),
  };
}

/**
 One record's modification time, or nothing where a discard removed it since
 its directory was listed.

 @param path - record's absolute path

 @returns Record, or none

 @throws What `stat` raises for any failure but an absent file

 @example
 ```ts
 const records = await recordAt({ path, },);
 ```
 */
async function recordAt({ path, }: { readonly path: string; },): Promise<readonly SliceRecord[]> {
  try {
    /**
     The file's status.
     */
    const { mtimeMs, } = await stat(path,);
    return [{
      path,
      modifiedMs: mtimeMs,
    },];
  } catch (error) {
    // A settled entry's discard can remove its records between the listing
    // and this read; any other fault is not an answer.
    rethrowUnlessMissingPath({ error, },);
    return [];
  }
}

/**
 Every record in one runs directory's slice cache: each entry's subdirectory,
 each file named as a slice, never a link or the namespace markers.

 @param runsDir - runs directory read

 @returns Its records, empty where it holds no slice cache

 @example
 ```ts
 const records = await recordsUnder({ runsDir, },);
 ```
 */
async function recordsUnder({ runsDir, }: { readonly runsDir: string; },): Promise<readonly SliceRecord[]> {
  /**
   The runs directory's slice cache.
   */
  const cacheDir = join(
    runsDir,
    SLICE_CACHE_DIR,
  );
  /**
   One subdirectory per entry.
   */
  const entryIds = await presentNamesOfKind({
    dir: cacheDir,
    kind: 'directory',
  },);
  /**
   Records of each entry.
   */
  const perEntry = await allInInputOrder({
    members: entryIds.map(async function recordsOf(entryId,): Promise<readonly SliceRecord[]> {
      /**
       The entry's cache directory.
       */
      const entryDir = join(
        cacheDir,
        entryId,
      );
      /**
       Files named as slices.
       */
      const names = (await presentNamesOfKind({
        dir: entryDir,
        kind: 'file',
      },)).filter(function isSlice(name,): boolean {
        return isSliceFileName({ name, },);
      },);
      /**
       Each with its time.
       */
      const records = await allInInputOrder({
        members: names.map(async function recordOf(name,): Promise<readonly SliceRecord[]> {
          return await recordAt({
            path: join(
              entryDir,
              name,
            ),
          },);
        },),
      },);
      return records.flat();
    },),
  },);
  return perEntry.flat();
}

/**
 Reads every slice-cache record under some runs directories, for the newest
 and the count.

 @param runsDirs - runs directories to read, each named in the report

 @returns Account

 @throws What a listing or `stat` raises for any failure but an absent path

 @example
 ```ts
 const account = await sliceCacheAccount({ runsDirs, },);
 ```
 */
export async function sliceCacheAccount(
  { runsDirs, }: { readonly runsDirs: readonly string[]; },
): Promise<SliceCacheAccount> {
  /**
   Every record, across the runs directories.
   */
  const records = (await allInInputOrder({
    members: runsDirs.map(async function read(runsDir,): Promise<readonly SliceRecord[]> {
      return await recordsUnder({ runsDir, },);
    },),
  },)).flat();
  /**
   The record written last; a tie goes to the path first in code-point order,
   so one tree always names one record.
   */
  const [newest,] = records.toSorted(function newestFirst(
    left,
    right,
  ): number {
    if (left.modifiedMs !== right.modifiedMs)
      return right.modifiedMs - left.modifiedMs;
    return compareCodePoints({
      left: left.path,
      right: right.path,
    },);
  },);
  return {
    runsDirs: textsInCodePointOrder({ texts: runsDirs, },),
    count: records.length,
    newest: (newest === undefined)
      ? { kind: 'none', }
      : {
        kind: 'found',
        record: newest,
      },
  };
}

//endregion Slice cache account
