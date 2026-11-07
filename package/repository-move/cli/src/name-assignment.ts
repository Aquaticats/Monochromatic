/**
 Gives every subvolume of a run its copy name, keyed by the subvolume's UUID plus its path, and states what changed
 since the names were first given.

 A name is never reused: a new subvolume takes the next number above every name that has a record or a folder in the
 snapshot folder, so a resume after worktrees were added or removed continues each subvolume's own records.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { pathExists, } from './completion-records.ts';
import type { SnapshotEntry, } from './completion-records.ts';
import { NAME_DIGITS, } from './constants.ts';
import type { Subvolume, } from './inventory.ts';

/**
 A subvolume with the copy name it was given.
 */
export type NamedSubvolume = Subvolume & {
  readonly name: string;
};

/**
 One change since the ledger was written, with the sentence that states it.
 */
export type Change = {
  readonly kind: 'added' | 'moved' | 'removed' | 'replaced';
  readonly message: string;
};

/**
 Numbers already used by entries named `<prefix><digits>` followed by a dot or nothing in a folder.

 @param folder - folder to read
 
 @param prefix - copy-name prefix
 
 @returns the numbers
 */
async function usedNumbers(
  {
    folder,
    prefix,
  }: {
    readonly folder: string;
    readonly prefix: string;
  },
): Promise<readonly number[]> {
  if (!await pathExists(folder,)) {
    return [];
  }
  const numbers: number[] = [];
  for (const entry of await readdir(folder,)) {
    const stem = entry.split('.',)[0] ?? '';
    const digits = stem.slice(prefix.length,);
    if (stem.startsWith(prefix,) && (digits.length >= NAME_DIGITS)
      && /^\d+$/v.test(digits,)) {
      numbers.push(Number(digits,),);
    }
  }
  return numbers;
}

/**
 Tells whether a ledger entry is for exactly this subvolume.

 @param entry - the ledger entry
 
 @param subvolume - the subvolume
 
 @returns true when UUID and path both match
 */
function isSame(
  {
    entry,
    subvolume,
  }: {
    readonly entry: SnapshotEntry;
    readonly subvolume: Subvolume;
  },
): boolean {
  return (entry.uuid === subvolume.uuid) && (entry.path === subvolume.path);
}

/**
 States that an old copy stays untouched.

 @param entry - the ledger entry of the old copy
 
 @returns the sentence
 */
function stays(entry: SnapshotEntry,): string {
  return `its copy ${entry.name} and records stay as they are and nothing was deleted`;
}

/**
 Describes how a current subvolume that has no ledger entry came to be.

 @param subvolume - the subvolume
 
 @param ledger - the ledger
 
 @returns the change
 */
function describeNew(
  {
    subvolume,
    ledger,
  }: {
    readonly subvolume: Subvolume;
    readonly ledger: readonly SnapshotEntry[];
  },
): Change {
  const samePath = ledger.find(function atPath(entry,) {
    return entry.path === subvolume.path;
  },);
  if (samePath !== undefined) {
    return {
      kind: 'replaced',
      message: `replaced: ${subvolume.path} now has UUID ${subvolume.uuid}, it had ${samePath.uuid}; `
        + `it is copied as a new subvolume, and ${stays(samePath,)}`,
    };
  }
  const sameUuid = ledger.find(function withUuid(entry,) {
    return entry.uuid === subvolume.uuid;
  },);
  if (sameUuid !== undefined) {
    return {
      kind: 'moved',
      message: `moved: UUID ${subvolume.uuid} was at ${sameUuid.path} and is now at ${subvolume.path}; `
        + `it is copied again under its new path, and ${stays(sameUuid,)}`,
    };
  }
  return {
    kind: 'added',
    message: `added since the pass started: ${subvolume.path} (${subvolume.uuid})`,
  };
}

/**
 Assigns names to the current subvolumes.

 @param current - the subvolumes this run covers, main first
 
 @param ledger - the finished snapshot records of this prefix
 
 @param prefix - `s` or `f`
 
 @param folders - folders whose entries also reserve numbers (the snapshot folder and the state folder)
 
 @returns the named subvolumes in the same order, and the changes since the ledger was written
 */
export async function assignNames(
  {
    current,
    ledger,
    prefix,
    folders,
  }: {
    readonly current: readonly Subvolume[];
    readonly ledger: readonly SnapshotEntry[];
    readonly prefix: string;
    readonly folders: readonly string[];
  },
): Promise<{
  readonly named: readonly NamedSubvolume[];
  readonly changes: readonly Change[];
}> {
  const reserved = [
    ...ledger.map(function numberOf(entry,) {
      return Number(entry.name
        .slice(prefix.length,),);
    },),
    ...(await Promise.all(folders.map(async function numbersIn(folder,) {
      return await usedNumbers({
        folder,
        prefix,
      },);
    },),)).flat(),
  ];
  let next = Math.max(
    -1,
    ...reserved,
  ) + 1;
  const named: NamedSubvolume[] = [];
  for (const subvolume of current) {
    const existing = ledger.find(function matches(entry,) {
      return isSame({
        entry,
        subvolume,
      },);
    },);
    if (existing === undefined) {
      named.push({
        ...subvolume,
        name: `${prefix}${String(next,)
          .padStart(
            NAME_DIGITS,
            '0',
          )}`,
      },);
      next += 1;
    }
    else {
      named.push({
        ...subvolume,
        name: existing.name,
      },);
    }
  }
  if (ledger.length === 0) {
    return {
      named,
      changes: [],
    };
  }
  const added = current.filter(function hasNoEntry(subvolume,) {
    return !ledger.some(function matches(entry,) {
      return isSame({
        entry,
        subvolume,
      },);
    },);
  },)
    .map(function toChange(subvolume,) {
      return describeNew({
        subvolume,
        ledger,
      },);
    },);
  const removed = ledger.filter(function isGone(entry,) {
    return !current.some(function sharesIdentity(subvolume,) {
      return (subvolume.path === entry.path) || (subvolume.uuid === entry.uuid);
    },);
  },)
    .map(function toChange(entry,): Change {
      return {
        kind: 'removed',
        message: `no longer present: ${entry.path} (${entry.uuid}); ${stays(entry,)}`,
      };
    },);
  return {
    named,
    changes: [
      ...added,
      ...removed,
    ],
  };
}
