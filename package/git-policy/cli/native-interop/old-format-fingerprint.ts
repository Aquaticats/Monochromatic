/**
 A cheap, comparable description of the stored repository state a refusing wrapper must leave alone.
 */
import {
  lstat,
  readdir,
} from 'node:fs/promises';
import { join } from 'node:path';

import { isMissingPath } from './support.ts';

/**
 Directories below `.git` that hold objects, references and transaction state.
 Index and log files are left out: a read command refreshes the index and may write diagnostics.
 */
export const STORED_STATE_DIRECTORIES: readonly string[] = [
  'objects',
  'refs',
  'cli-git-transactions',
  'cli-git/shadow',
];

/**
 Path part that marks the record of a lock directory, which a wrapper retires when its holder is dead.
 */
const LOCK_DIRECTORY_PART = '/landing.lock/';
/**
 Suffix of the marker a landing leaves beside a pack so that nothing collects it before the ref moves.
 */
const KEEP_SUFFIX = '.keep';

/**
 Files that a wrapper may retire on its own, whatever else it decides: a dead holder's lock record and a pack's keep marker.

 @param line - a line of a fingerprint
 
 @returns whether losing the file is housekeeping
 */
function isHousekeeping(line: string): boolean {
  /**
   The path part of the line, before the size.
   */
  const path = line.slice(
    0,
    line.lastIndexOf(':'),
  );
  return path.includes(LOCK_DIRECTORY_PART) || path.endsWith(KEEP_SUFFIX);
}

/**
 The stored files that a later fingerprint no longer shows, apart from housekeeping.

 @param before - fingerprint taken before

 @param after - fingerprint taken after

 @returns lines of files that were lost or resized

 @example
 ```ts
 const lost = lostEntries({ before, after });
 ```
 */
export function lostEntries({
  before,
  after,
}: {
  readonly before: readonly string[];
  readonly after: readonly string[];
}): string[] {
  return before.filter(function wasLost(line) {
    return (!after.includes(line)) && (!isHousekeeping(line));
  });
}

/**
 Names and sizes of every file below the given directories of a repository's `.git`.

 @param repository - repository to describe

 @param subdirectories - the directories below `.git` to describe

 @returns one `directory/name:size` line per file, in no particular order, none for a directory that is absent

 @example
 ```ts
 const lines = await fingerprintOf({ repository, subdirectories: STORED_STATE_DIRECTORIES });
 ```
 */
export async function fingerprintOf({
  repository,
  subdirectories,
}: {
  readonly repository: string;
  readonly subdirectories: readonly string[];
}): Promise<string[]> {
  /**
   One list of lines per described directory.
   */
  const groups = await Promise.all(subdirectories.map(async function describeDirectory(subdirectory) {
    /**
     The directory below `.git`.
     */
    const base = join(
      repository,
      '.git',
      subdirectory,
    );
    try {
      /**
       Every path below the directory, files and directories alike.
       */
      const names = await readdir(
        base,
        { recursive: true },
      );
      /**
       One line per file and none per directory, each path described from its status.
       */
      const lines = await Promise.all(names.map(async function describeEntry(name): Promise<readonly string[]> {
        /**
         Status of the path, read without following a symbolic link.
         */
        const entry = await lstat(join(
          base,
          name,
        ));
        if (entry.isDirectory())
          return [];
        return [`${subdirectory}/${name}:${String(entry.size)}`];
      }));
      return lines.flat();
    }
    catch (error) {
      if (isMissingPath(error))
        return [];
      throw error;
    }
  }));
  return groups.flat();
}
