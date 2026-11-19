/**
 Records when a launcher first found files at its worktree path without a proof of ownership.
 The record lives beside the worktree so retries can enforce the same grace period before clearing that path.

 @module
 */

import {
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 Filesystem diagnostic when no record exists at the requested path.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Hours in a day, which is the bound an unproven worktree is kept for.
 */
const HOURS_IN_A_DAY = 24;

/**
 Milliseconds in one hour.
 */
const MILLISECONDS_IN_AN_HOUR = 3_600_000;

/**
 The end of the file a start writes when it finds a worktree no proof says it made, which is the worktree a person made by hand or
 one made before this package wrote proofs. It carries the first sighting, so a bound can pass without removing a person's work at
 once and without keeping it for ever.
 */
const UNPROVEN_SIGHTING_FILE_END = '.unproven.json';

/**
 How long an unproven worktree is kept after the first start that wanted its path saw it.

 The decision of 2026-11-15 is that a worktree with no proof of who made it is removed after a bound. The bound here is a day: the
 longest run this machine starts asks for sixteen hours (`--wall=57600` in the coordinating session's batches), so a worktree nobody
 claimed a day after a start first wanted its path cannot be a run's own worktree, while a person who made one by hand still has a
 working day to say so by naming the run another way. Nothing is removed before it.
 */
export const UNPROVEN_WORKTREE_KEPT_MILLISECONDS: number = HOURS_IN_A_DAY * MILLISECONDS_IN_AN_HOUR;

/**
 What a start found at a path that holds a worktree no proof names: when a start first wanted that path, and which start saw it.
 */
export type UnprovenSighting = {
  /**
   The worktree's path this sighting is about.
   */
  readonly path: string;

  /**
   When a start first found it unproven, as an ISO 8601 instant.
   */
  readonly firstSeenAt: string;

  /**
   The run folder of the start that saw it.
   */
  readonly seenBy: string;
};

/**
 Reads the file that records the first sighting of a worktree no proof names.

 @param worktree - the worktree's path

 @returns the file's path

 @example
 ```ts
 const file = unprovenSightingFileOf({ worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export function unprovenSightingFileOf({ worktree, }: { readonly worktree: string },): string {
  return join(
    dirname(worktree,),
    `.${basename(worktree,)}${UNPROVEN_SIGHTING_FILE_END}`,
  );
}

/**
 Reads the record of the first start that wanted a path holding a worktree no proof names.

 @param worktree - the worktree's path

 @returns the sighting, or nothing when no start has recorded one, or the problem reading it gave

 @example
 ```ts
 const { sighting } = await readSighting({ worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function readSighting({ worktree, }: { readonly worktree: string },): Promise<{
  readonly sighting?: UnprovenSighting;
  readonly problem?: string
}> {
  /**
   The file the first sighting is written in.
   */
  const file = unprovenSightingFileOf({ worktree, },);
  try {
    /**
     What the file holds, of any shape.
     */
    const stored: unknown = JSON.parse(await readFile(
      file,
      'utf8',
    ),);
    if (((typeof stored) !== 'object') || (stored === null)) {
      return { problem: `${file} holds no record of a sighting`, };
    }
    /**
     The three fields a sighting carries.
     */
    const fields: Readonly<Record<string, unknown>> = { ...stored, };
    /**
     The path the launcher found, the time it found it, and the run that found it.
     */
    const {
      path,
      firstSeenAt,
      seenBy,
    } = fields;
    if (((typeof path) !== 'string') || ((typeof firstSeenAt) !== 'string')
      || ((typeof seenBy) !== 'string')) {
      return { problem: `${file} holds no record of a sighting`, };
    }
    return { sighting: {
      path,
      firstSeenAt,
      seenBy,
    }, };
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {};
    }
    return { problem: `${file} cannot be read as a record of a sighting: ${String(error,)}`, };
  }
}

/**
 Writes the first sighting of an unproven worktree, with one exclusive create, so the bound begins once.

 @param worktree - the worktree's path

 @param sighting - when this start saw it, and which start it was

 @returns whether this start wrote the first sighting

 @example
 ```ts
 const first = await writeSighting({ worktree: '/repo/.claude/worktrees/pi-a', sighting, },);
 ```
 */
export async function writeSighting({
  worktree,
  sighting,
}: {
  readonly worktree: string;
  readonly sighting: UnprovenSighting;
},): Promise<boolean> {
  /**
   The file the first sighting is written in.
   */
  const file = unprovenSightingFileOf({ worktree, },);
  await mkdir(
    dirname(file,),
    { recursive: true, },
  );
  try {
    await writeFile(
      file,
      `${JSON.stringify(
        sighting,
        undefined,
        2,
      )}\n`,
      {
        encoding: 'utf8',
        flag: 'wx',
      },
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === 'EEXIST') {
      return false;
    }
    throw error;
  }
}

/**
 Moves the sighting of an unproven worktree out of the way, so the making that removes it is the one that began the bound.

 @param worktree - the worktree's path

 @returns whether this caller moved it

 @example
 ```ts
 const moved = await moveTheSightingAway({ worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function moveTheSightingAway({ worktree, }: { readonly worktree: string },): Promise<boolean> {
  /**
   The file the first sighting is written in.
   */
  const file = unprovenSightingFileOf({ worktree, },);
  try {
    await rename(
      file,
      `${file}.${String(process.pid,)}.removed`,
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}
