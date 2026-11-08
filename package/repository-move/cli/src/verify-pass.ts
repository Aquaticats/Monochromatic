/**
 The verification pass: every copy of a pass against the read-only snapshot it was sent from, one `rsync --dry-run` each.
 Both sides are immutable, so it needs no quiet moment and runs before the cut-over window. By default it compares what
 `rsync` compares without reading data (path, type, size, time, mode, owner), which a copy built on a wrongly paired
 inode fails; `--checksum` reads every byte.

 The only differences excused are the empty placeholder folders a snapshot shows where its source held a nested
 subvolume: a send does not carry one, and nothing can be stored in one. Those of the nested subvolumes the inventory
 lists are known in advance. The others are found by their inode number: one is excused and named in the output only
 when the live tree holds the same placeholder at its path, or nothing (a placeholder a worktree kept from the
 snapshot it was made from, for a subvolume that is gone); a real folder or subvolume there stops the pass, since
 the inventory missed it and its files were not copied.

 @module
 */

import { lstat, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  createRecords,
  MISSING_PATH_CODE,
  readSnapshotEntries,
  type SnapshotEntry,
} from './completion-records.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  FINAL_PREFIX,
  FIRST_PREFIX,
} from './constants.ts';
import { MoveStopError, } from './errors.ts';
import { foundUnchanged, } from './final-diff-step.ts';
import type { MoveOptions, } from './move-options.ts';

/**
 How many differing lines a stop quotes.
 */
const QUOTED_DIFFERENCES = 20;

/**
 The characters that make `rsync` read a pattern as a wildcard pattern.
 */
const RSYNC_WILDCARD_CHARACTERS = '*?[';

/**
 Escapes the characters `rsync` reads as pattern syntax, so a path matches only itself. `rsync` reads a backslash as
 an escape only in a pattern that holds a wildcard character (`*`, `?` or `[`; "PATTERN MATCHING RULES" in
 `man rsync`), so a path without one is returned as it is: escaping it would make its backslashes literal.

 @param path - a path

 @returns the path itself without a wildcard character, else with a backslash before each of `\`, `*`, `?`, `[` and `]`

 @example
 ```ts
 import { escapeRsyncPattern, } from './verify-pass.ts';

 escapeRsyncPattern('a[1]',);
 // 'a\[1\]'
 escapeRsyncPattern(String.raw`back\slash`,);
 // 'back\slash'
 ```
 */
export function escapeRsyncPattern(path: string,): string {
  if (!Array.from(path,)
    .some(function isWildcard(character,) {
      return RSYNC_WILDCARD_CHARACTERS.includes(character,);
    },)) {
    return path;
  }
  return Array.from(path,)
    .map(function escapeOne(character,) {
      return String.raw`\*?[]`.includes(character,) ? `\\${character}` : character;
    },)
    .join('',);
}

/**
 The exclusions of one subvolume: exactly the nested subvolumes the inventory lists below it, anchored at the
 subvolume's root and matching folders only.

 @param entry - the subvolume

 @param everyone - every subvolume of the pass

 @returns `--exclude=` arguments

 @example
 ```ts
 import { nestedExclusions, } from './verify-pass.ts';

 const arguments_ = nestedExclusions({ entry, everyone, },);
 ```
 */
export function nestedExclusions(
  {
    entry,
    everyone,
  }: {
    readonly entry: SnapshotEntry;
    readonly everyone: readonly SnapshotEntry[];
  },
): readonly string[] {
  return nestedPaths({
    entry,
    everyone,
  },)
    .map(exclusionOf,);
}

/**
 The paths, relative to a subvolume's root, of the nested subvolumes the inventory lists below it.

 @param entry - the subvolume

 @param everyone - every subvolume of the pass

 @returns relative paths

 @example
 ```ts
 import { nestedPaths, } from './verify-pass.ts';

 const paths = nestedPaths({ entry, everyone, },);
 ```
 */
export function nestedPaths(
  {
    entry,
    everyone,
  }: {
    readonly entry: SnapshotEntry;
    readonly everyone: readonly SnapshotEntry[];
  },
): readonly string[] {
  return everyone.filter(function isNested(other,) {
    return other.path
      .startsWith(`${entry.path}/`,);
  },)
    .map(function relativePath(other,) {
      return other.path
        .slice(entry.path
          .length
          + 1,);
    },);
}

/**
 The `--exclude=` argument that skips one folder, anchored at the transfer root and matching folders only.

 @param path - the folder, relative to the transfer root

 @returns the argument

 @example
 ```ts
 import { exclusionOf, } from './verify-pass.ts';

 exclusionOf('wt/a',);
 // '--exclude=/wt/a/'
 ```
 */
export function exclusionOf(path: string,): string {
  return `--exclude=/${escapeRsyncPattern(path,)}/`;
}

/**
 Every empty placeholder folder of a subvolume inside one snapshot, found by its inode number.

 @param snapshot - the snapshot's folder

 @param inode - the placeholder inode number (`EMPTY_SUBVOLUME_PLACEHOLDER_INODE` on btrfs)

 @param runner - the runner; `find` runs with the transfer deadline because it walks the whole snapshot

 @returns paths relative to the snapshot's root

 @example
 ```ts
 import { placeholderFolders, } from './verify-pass.ts';

 const found = await placeholderFolders({ snapshot, inode: 2, runner, },);
 ```
 */
export async function placeholderFolders(
  {
    snapshot,
    inode,
    runner,
  }: {
    readonly snapshot: string;
    readonly inode: number;
    readonly runner: Runner;
  },
): Promise<readonly string[]> {
  /**
   The NUL-separated relative paths `find` printed.
   */
  const listing = await runner.exec({
    command: 'find',
    args: [
      `${snapshot}/`,
      '-xdev',
      '-inum',
      String(inode,),
      '-type',
      'd',
      '-printf',
      String.raw`%P\0`,
    ],
    transfer: true,
  },);
  return listing.split('\0',)
    .filter(function notEmpty(path,) {
      return path !== '';
    },);
}

/**
 What the live tree holds at the path of a placeholder a snapshot shows.
 */
type LiveKind = 'gone' | 'placeholder' | 'present';

/**
 A placeholder folder of a snapshot, relative to its root, with what the live tree holds at the same path.
 */
type PlaceholderOnLive = {
  readonly path: string;
  readonly kind: LiveKind;
};

/**
 What the live tree holds at a path: nothing, the same empty placeholder folder, or anything else (a folder or
 subvolume with files, a file, a link).

 @param path - the live path

 @param inode - the placeholder inode number

 @returns the kind

 @throws Error from `lstat` for anything but a missing path, so an unreadable path stops the pass

 @example
 ```ts
 import { liveKindOf, } from './verify-pass.ts';

 const kind = await liveKindOf({ path: '/var/home/user/Monochromatic/wt/x', inode: 2, },);
 ```
 */
export async function liveKindOf(
  {
    path,
    inode,
  }: {
    readonly path: string;
    readonly inode: number;
  },
): Promise<LiveKind> {
  try {
    /**
     The live entry, read without following a link.
     */
    const found = await lstat(
      path,
      { bigint: true, },
    );
    return found.isDirectory() && (found.ino === BigInt(inode,)) ? 'placeholder' : 'present';
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return 'gone';
    }
    throw error;
  }
}

/**
 Verifies every copy of a pass against its snapshot.

 @param options - the options; `pass` is `verify-first` or `verify-final`

 @param runner - the runner

 @throws MoveStopError naming the first copy that differs and the first differences, with every earlier verification recorded

 @example
 ```ts
 import { runVerifyPass, } from './verify-pass.ts';

 await runVerifyPass({ options, runner, },);
 ```
 */
export async function runVerifyPass(
  {
    options,
    runner,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
  },
): Promise<void> {
  /**
   The record prefix of the pass being verified.
   */
  const prefix = options.pass === 'verify-final' ? FINAL_PREFIX : FIRST_PREFIX;
  /**
   The record step: a quick check does not stand for a checksum check.
   */
  const step = options.checksum ? 'verified-checksum' : 'verified';
  /**
   Every subvolume the pass copied.
   */
  const everyone = await readSnapshotEntries({
    stateFolder: options.stateFolder,
    prefix,
  },);
  /**
   The run's record store.
   */
  const records = createRecords({
    stateFolder: options.stateFolder,
    writable: options.run,
  },);
  for (const entry of everyone) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- copies are compared one at a time: each rsync reads two trees from the same disks, and a record after each lets a rerun resume; see doc/lint/sequential-awaits.md
    if (await records.has({
      name: entry.name,
      step,
    },)) {
      continue;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- copies are compared one at a time: each rsync reads two trees from the same disks, and a record after each lets a rerun resume; see doc/lint/sequential-awaits.md
    if ((options.pass === 'verify-final') && await foundUnchanged({
      stateFolder: options.stateFolder,
      name: entry.name,
    },)) {
      runner.log(`${entry.name}: unchanged since the first pass, not compared`,);
      continue;
    }
    /**
     The nested subvolumes the inventory lists below this one, relative to its root.
     */
    const nested = nestedPaths({
      entry,
      everyone,
    },);
    /**
     Every placeholder folder the snapshot shows, listed or not.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- copies are compared one at a time: each rsync reads two trees from the same disks, and a record after each lets a rerun resume; see doc/lint/sequential-awaits.md
    const placeholders = await placeholderFolders({
      snapshot: join(
        options.snapshotFolder,
        entry.name,
      ),
      inode: options.placeholderInode,
      runner,
    },);
    /**
     The placeholders no listed subvolume accounts for, each with what the live tree holds at its path.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- copies are compared one at a time: each rsync reads two trees from the same disks, and a record after each lets a rerun resume; see doc/lint/sequential-awaits.md
    const unlistedKinds = await Promise.all(placeholders.filter(function notListed(path,) {
      return !nested.includes(path,);
    },)
      .map(async function withLiveKind(path,): Promise<PlaceholderOnLive> {
        return {
          path,
          kind: await liveKindOf({
            path: join(
              entry.path,
              path,
            ),
            inode: options.placeholderInode,
          },),
        };
      },),);
    /**
     Unlisted placeholders where the live tree holds a real folder or subvolume: its files were not copied.
     */
    const uncopied = unlistedKinds.filter(function holdsFiles(found,) {
      return found.kind === 'present';
    },);
    if (uncopied.length > 0) {
      throw new MoveStopError(
        `the snapshot ${entry.name} of ${entry.path} shows an empty placeholder folder where the live tree holds a folder or subvolume the inventory does not list, so its files were not copied: ${
          uncopied.map(function livePath(found,) {
            return join(
              entry.path,
              found.path,
            );
          },)
            .join(' | ',)
        }. List that subvolume (run the inventory stage again) or remove it, then run this pass again`,
      );
    }
    /**
     Unlisted placeholders that stand for nothing: the live tree holds the same empty placeholder there, or nothing.
     */
    const unlisted = unlistedKinds.map(function pathOf(found,) {
      return found.path;
    },);
    if (unlisted.length > 0) {
      runner.log(
        `${entry.name}: not compared, ${String(unlisted.length,)} empty placeholder folder(s) of a subvolume that is not in the inventory (a send never carries one, and nothing can be stored in one): ${
          unlisted.join(' | ',)
        }`,
      );
    }
    /**
     What rsync would change in the copy to make it equal to the snapshot.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- copies are compared one at a time: each rsync reads two trees from the same disks, and a record after each lets a rerun resume; see doc/lint/sequential-awaits.md
    const differences = (await runner.exec({
      command: 'rsync',
      args: [
        '--archive',
        '--dry-run',
        '--itemize-changes',
        '--delete',
        '--one-file-system',
        '--omit-dir-times',
        ...(options.checksum ? ['--checksum',] : []),
        ...nested.map(exclusionOf,),
        ...unlisted.map(exclusionOf,),
        `${join(
          options.snapshotFolder,
          entry.name,
        )}/`,
        `${join(
          options.receivedFolder,
          entry.name,
        )}/`,
      ],
      transfer: true,
    },)).split('\n',)
      .filter(function notBlank(line,) {
        return line !== '';
      },);
    if (differences.length > 0) {
      throw new MoveStopError(
        `the copy ${entry.name} of ${entry.path} differs from its snapshot (${String(differences.length,)} lines; the first ${
          String(QUOTED_DIFFERENCES,)
        } follow): ${differences.slice(
          0,
          QUOTED_DIFFERENCES,
        )
          .join(' | ',)}`,
      );
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- copies are compared one at a time: each rsync reads two trees from the same disks, and a record after each lets a rerun resume; see doc/lint/sequential-awaits.md
    await records.write({
      name: entry.name,
      step,
      data: {
        path: entry.path,
        uuid: entry.uuid,
      },
    },);
  }
  runner.log(`${options.pass} finished: ${String(everyone.length,)} copies match their snapshots`,);
}
