/**
 Completion records: one JSON file per finished step in the state folder, written to a temporary name and renamed, so
 a record exists only for a step that finished.

 @module
 */

import {
  access,
  readdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { isDigits, } from './text-checks.ts';

/**
 Suffix of the snapshot records, which double as the ledger of copy names.
 */
/**
 Error code of a path that does not exist.
 */
export const MISSING_PATH_CODE = 'ENOENT';

/**
 Suffix of the snapshot records, which double as the ledger of copy names.
 */
const SNAPSHOT_SUFFIX = '.snapshot.json';

/**
 A run's record store; in print mode writing does nothing.
 */
export type Records = {
  /**
   Tells whether the step of a copy has a completion record.
   */
  readonly has: (key: RecordKey,) => Promise<boolean>;

  /**
   Writes the completion record of a step.
   */
  readonly write: (entry: RecordEntry,) => Promise<void>;
};

/**
 Names the record of one step of one copy.
 */
export type RecordKey = {
  readonly name: string;
  readonly step: string;
};

/**
 A record to write: its key and the data it carries.
 */
export type RecordEntry = RecordKey & {
  readonly data: object;
};

/**
 One finished snapshot: the copy name it got and the subvolume it is a snapshot of.
 */
export type SnapshotEntry = {
  readonly name: string;
  readonly uuid: string;
  readonly path: string;
};

/**
 Tells whether a path exists, without throwing for a missing one.

 @param path - the path
 
 @returns true when it exists
 */
export async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if ((error as NodeJS.ErrnoException).code === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Writes a text file through a temporary name and a rename, so a reader sees all of it or none of it.

 @param path - the file
 
 @param text - its content
 */
export async function writeTextAtomically(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): Promise<void> {
  await writeFile(
    `${path}.tmp`,
    text,
  );
  await rename(
    `${path}.tmp`,
    path,
  );
}

/**
 Makes the record store of a state folder.

 @param stateFolder - the state folder
 
 @param writable - false in print mode, where nothing is written
 
 @returns the store
 */
export function createRecords(
  {
    stateFolder,
    writable,
  }: {
    readonly stateFolder: string;
    readonly writable: boolean;
  },
): Records {
  /**
   Path of one record file.

   @param name - the copy name
   
   @param step - the step name
   
   @returns the path
   */
  function recordPath({
    name,
    step,
  }: RecordKey,): string {
    return join(
      stateFolder,
      `${name}.${step}.json`,
    );
  }

  /**
   Tells whether a record exists.

   @param name - the copy name
   
   @param step - the step name
   
   @returns true when it exists
   */
  async function has({
    name,
    step,
  }: RecordKey,): Promise<boolean> {
    return await pathExists(recordPath({
      name,
      step,
    },),);
  }

  /**
   Writes a record unless this is print mode.

   @param name - the copy name
   
   @param step - the step name
   
   @param data - what the record carries
   */
  async function write({
    name,
    step,
    data,
  }: RecordEntry,): Promise<void> {
    if (writable) {
      await writeTextAtomically({
        path: recordPath({
          name,
          step,
        },),
        text: JSON.stringify({
          at: new Date().toISOString(),
          ...data,
        },),
      },);
    }
  }

  return {
    has,
    write,
  };
}

/**
 Reads every snapshot record of a prefix: the ledger that maps (UUID, path) to a copy name.

 @param stateFolder - the state folder
 
 @param prefix - `s` for the first pass, `f` for the final pass
 
 @returns the entries, sorted by name
 */
export async function readSnapshotEntries(
  {
    stateFolder,
    prefix,
  }: {
    readonly stateFolder: string;
    readonly prefix: string;
  },
): Promise<readonly SnapshotEntry[]> {
  if (!await pathExists(stateFolder,)) {
    return [];
  }
  /**
   The files this step reads.
   */
  const files = (await readdir(stateFolder,)).filter(function isSnapshotRecord(file,) {
    /**
     The digits part of a copy name.
     */
    const digits = file.slice(
      prefix.length,
      -SNAPSHOT_SUFFIX.length,
    );
    return file.startsWith(prefix,) && file.endsWith(SNAPSHOT_SUFFIX,)
      && isDigits(digits,);
  },)
    .toSorted();
  return await Promise.all(files.map(async function readEntry(file,) {
    /**
     The parsed content of a record file.
     */
    const data = JSON.parse(await readFile(
      join(
        stateFolder,
        file,
      ),
      'utf8',
    ),) as {
      readonly uuid?: string;
      readonly path?: string;
    };
    return {
      name: file.slice(
        0,
        -SNAPSHOT_SUFFIX.length,
      ),
      uuid: data.uuid ?? '',
      path: data.path ?? '',
    };
  },),);
}
