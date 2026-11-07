/**
 The verification pass: every copy of a pass against the read-only snapshot it was sent from, one `rsync --dry-run` each.
 Both sides are immutable, so it needs no quiet moment and runs before the cut-over window. By default it compares what
 `rsync` compares without reading data (path, type, size, time, mode, owner), which a copy built on a wrongly paired
 inode fails; `--checksum` reads every byte.

 The nested subvolumes the inventory lists below a subvolume are the only differences excused: a snapshot shows each as
 an empty folder and a send does not carry it.

 @module
 */

import { join, } from 'node:path';
import type { Runner, } from './command-runner.ts';
import {
  createRecords,
  readSnapshotEntries,
  type SnapshotEntry,
} from './completion-records.ts';
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
 Escapes the characters `rsync` reads as pattern syntax, so a path matches only itself.

 @param path - a path

 @returns the path with a backslash before each of `\`, `*`, `?`, `[` and `]`

 @example
 ```ts
 import { escapeRsyncPattern, } from './verify-pass.ts';

 escapeRsyncPattern('a[1]',);
 // 'a\[1\]'
 ```
 */
export function escapeRsyncPattern(path: string,): string {
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
  return everyone.filter(function isNested(other,) {
    return other.path
      .startsWith(`${entry.path}/`,);
  },)
    .map(function exclusion(other,) {
      return `--exclude=/${escapeRsyncPattern(other.path
        .slice(entry.path
          .length
          + 1,),)}/`;
    },);
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
        ...nestedExclusions({
          entry,
          everyone,
        },),
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
