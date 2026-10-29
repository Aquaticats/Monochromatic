/**
 A cheap, comparable description of the stored repository state a refusing wrapper must leave alone.
 */
/// <reference types="node" />
import {
  lstat,
  readdir,
} from 'node:fs/promises';
import { join } from 'node:path';

import { isMissingPath } from './support.mjs';

/**
 Directories below `.git` that hold objects, references and transaction state.
 Index and log files are left out: a read command refreshes the index and may write diagnostics.
 */
export const STORED_STATE_DIRECTORIES = [
  'objects',
  'refs',
  'cli-git-transactions',
  'cli-git/shadow',
];

/** Path part that marks the record of a lock directory, which a wrapper retires when its holder is dead. */
const LOCK_DIRECTORY_PART = '/landing.lock/';
/** Suffix of the marker a landing leaves beside a pack so that nothing collects it before the ref moves. */
const KEEP_SUFFIX = '.keep';

/**
 Files that a wrapper may retire on its own, whatever else it decides: a dead holder's lock record and a pack's keep marker.

 @param {string} line - a line of a fingerprint
 @returns {boolean} whether losing the file is housekeeping
 */
function isHousekeeping(line) {
  const path = line.slice(
    0,
    line.lastIndexOf(':'),
  );
  return path.includes(LOCK_DIRECTORY_PART) || path.endsWith(KEEP_SUFFIX);
}

/**
 The stored files that a later fingerprint no longer shows, apart from housekeeping.

 @param {{ before: readonly string[], after: readonly string[] }} request - fingerprints taken before and after
 @returns {string[]} lines of files that were lost or resized
 */
export function lostEntries({
  before,
  after,
}) {
  return before.filter(function wasLost(line) {
    return (!after.includes(line)) && (!isHousekeeping(line));
  });
}

/**
 Names and sizes of every file below the given directories of a repository's `.git`.

 @param {{ repository: string, subdirectories: readonly string[] }} request - repository and the directories below
   `.git` to describe
 @returns {Promise<string[]>} one `directory/name:size` line per file, in no particular order, none for a directory that is absent
 */
export async function fingerprintOf({
  repository,
  subdirectories,
}) {
  const groups = await Promise.all(subdirectories.map(async function describeDirectory(subdirectory) {
    const base = join(
      repository,
      '.git',
      subdirectory,
    );
    try {
      const names = await readdir(
        base,
        { recursive: true },
      );
      const entries = await Promise.all(names.map(async function describeEntry(name) {
        const entry = await lstat(join(
          base,
          name,
        ));
        return {
          name,
          entry,
        };
      }));
      return entries
        .filter(function isFile(described) {
          return !described.entry
            .isDirectory();
        })
        .map(function describeFile(described) {
          return `${subdirectory}/${described.name}:${String(described.entry
            .size)}`;
        });
    }
    catch (error) {
      if (isMissingPath(error))
        return [];
      throw error;
    }
  }));
  return groups.flat();
}
