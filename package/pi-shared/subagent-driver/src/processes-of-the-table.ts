/**
 The processes of the process folder this account may read, and the process that started each.

 Everything the gate counts is read from here: a launcher is a process whose entry point is one of the launcher's names, and the
 unit a launcher is waiting on is proved by the `systemd-run` client it started, whose parent is the launcher. A count that
 trusted a unit's name alone would read two runs of one name as one agent.

 @module
 */

import { join, } from 'node:path';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a process that has ended.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 The number no process has, which a parent that could not be read answers with: a parent nobody can name proves nothing, and a
 unit it would have proved is counted as another agent.
 */
export const NO_PROCESS = 0;

/**
 One process of the folder: its number, and the words its own command line was split into.
 */
export type ProcessGoing = {
  /**
   The process's number.
   */
  readonly pid: number;

  /**
   The words the kernel split the process's command line into, which name the program and its options.
   */
  readonly words: readonly string[];
};

/**
 Reads one process's command line, or nothing when the process ended between the listing and the read.

 @param path - the process's own command line file

 @returns the words the kernel split the line into, or nothing at all

 @throws Error when the read failed for any reason other than a process that is gone

 @example
 ```ts
 const words = await commandLineOrNothingIn('/proc/123/cmdline',);
 ```
 */
async function commandLineOrNothingIn(path: string,): Promise<readonly string[]> {
  try {
    return (await readFile(
      path,
      'utf8',
    )).split('\0',);
  }
  catch (error) {
    /**
     The code the operating system gave the read.
     */
    const code = errorCodeOf(error,);
    if ((code === MISSING_PATH_CODE) || (code === NO_SUCH_PROCESS_CODE)) {
      return [];
    }
    throw error;
  }
}

/**
 Reads every process of the folder whose command line this account may read.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @returns one entry per process whose command line could be read, in the order the folder listed them

 @example
 ```ts
 const processes = await processesGoingIn({ processFolder: '/proc', },);
 ```
 */
export async function processesGoingIn({
  processFolder,
}: {
  readonly processFolder: string;
}): Promise<readonly ProcessGoing[]> {
  /**
   The entries of the process folder, which is one per process number.
   */
  const entries = await readdir(processFolder,);
  /**
   The processes found so far.
   */
  const processes: ProcessGoing[] = [];
  for (const entry of entries) {
    if (!isDigitsOnly(entry,)) {
      continue;
    }
    /**
     The process's command line, split where the kernel splits it, or nothing when the process ended between the listing and
     the read.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one process after another, and a line is only readable while its process lives; see doc/lint/sequential-awaits.md
    const words = await commandLineOrNothingIn(join(
      processFolder,
      entry,
      'cmdline',
    ),);
    if (words.length === 0) {
      continue;
    }
    processes.push({
      pid: Number(entry,),
      words,
    },);
  }
  return processes;
}

/**
 Reads the number of the process that started one.

 {@link NO_PROCESS} answers a process that ended between the listing and the read, and one whose record cannot be read: a parent
 nobody can name proves nothing, and the unit it would have proved is counted as another agent rather than folded into a launcher.

 @param processFolder - the folder holding one entry per process

 @param pid - the process whose parent is read

 @returns the process that started it, or {@link NO_PROCESS}

 @example
 ```ts
 const parent = await parentPidOf({ processFolder: '/proc', pid: 123, },);
 ```
 */
export async function parentPidOf({
  processFolder,
  pid,
}: {
  readonly processFolder: string;
  readonly pid: number;
}): Promise<number> {
  try {
    /**
     The process's own record.
     */
    const record = await readFile(
      join(
        processFolder,
        String(pid,),
        'stat',
      ),
      'utf8',
    );
    /**
     What follows the program's name in brackets, which may itself hold spaces and brackets.
     */
    const fields = record.slice(record.lastIndexOf(')',) + 1)
      .trim()
      .split(' ',);
    /**
     The parent's number, which is the second field there: the process's state comes first.
     */
    const parent = Number(fields[1],);
    return ((Number.isInteger(parent,)) && (parent > NO_PROCESS)) ? parent : NO_PROCESS;
  }
  catch (error) {
    /**
     The code the operating system gave the read.
     */
    const code = errorCodeOf(error,);
    if ((code === MISSING_PATH_CODE) || (code === NO_SUCH_PROCESS_CODE)) {
      return NO_PROCESS;
    }
    throw error;
  }
}
