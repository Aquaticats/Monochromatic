/**
 Whether one process is descended from another: the chain of processes that started it, read from the process folder's own record
 of each.

 The driver keeps a run only for work the run itself started, so it has to tell a process its own Pi started from one another
 session, another run or the person started. On 2026-11-19 two runs that worked in the main checkout waited out their whole wall
 clock on another session's `codex`, `claude` and `ssh`, on other runs' `pi`, and on `systemd-run` scopes of both, because the
 driver counted every process working in the run's folder.

 Descent is what the process table holds about who started what: a command the run's own `process` tool starts is a child of the
 run's Pi, and a `systemd-run --user --scope` the agent makes keeps the parent that started it while it leaves the run's own
 control group. A process another session started descends from that session, and nothing of it reaches this run.

 A parent link is read while the run's own Pi still runs, which is when the driver asks. A process whose parent has ended belongs to
 whatever took it in when that parent went, so no chain then says where it came from: such a chain ends without reaching the root,
 and the caller reads that as no evidence rather than as evidence for either answer.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Where the parent's process number sits in the fields that follow the program's name on a `/proc/<pid>/stat` line: the state comes
 first, and the parent after it.
 */
const PARENT_FIELD = 1;

/**
 The process number the kernel writes for a process nothing started, which is where a chain of parents ends.
 */
const NO_PARENT = 0;

/**
 The failures that mean a folder holds no record this account may read: the process ended, its entry is not a folder, or this
 account may not read it.
 */
const UNREADABLE_RECORD_CODES: ReadonlySet<string> = new Set<string>([
  MISSING_PATH_CODE,
  'ENOTDIR',
  'EACCES',
  'EPERM',
],);

/**
 A record that names no parent, as a symbol so no process number can be mistaken for it: a process that ended, one this account may
 not read, the first process, and a record that is no record at all all name none.
 */
export const NO_PARENT_RECORD: unique symbol = Symbol('no PPid record names the program that started this one',);

/**
 One record of the process folder, or the fact that there is none this account may read.
 */
type RecordOfAProcess =
  | { readonly text: string }
    | { readonly unreadable: true };

/**
 Reads one record the process folder keeps of a process.

 @param processFolder - folder holding one entry per process, `/proc` here

 @param pid - process number whose record is wanted

 @param name - name of the record, as the folder writes it

 @returns the record's text, or that there is none this account may read

 @example
 ```ts
 const record = await recordOf({ processFolder: '/proc', pid: 486667, name: 'stat', },);
 ```
 */
async function recordOf({
  processFolder,
  pid,
  name,
}: {
  readonly processFolder: string;
  readonly pid: number;
  readonly name: string;
},): Promise<RecordOfAProcess> {
  try {
    return {
      text: await readFile(
        join(
          processFolder,
          String(pid,),
          name,
        ),
        'utf8',
      ),
    };
  }
  catch (error) {
    // A process that ended, one whose folder this account may not read, and one whose entry is not a folder have no record. The
    // caller reads that as no evidence either way, which is what a process list always holds for the processes that ended while
    // it was read.
    if (UNREADABLE_RECORD_CODES.has(errorCodeOf(error,),)) {
      return { unreadable: true, };
    }
    throw error;
  }
}

/**
 Reads the process number a record names as the process that started the process recorded.

 @param record - the process folder's own record of one process

 @returns the parent's process number, or {@link NO_PARENT_RECORD} when the record names none

 @example
 ```ts
 const parent = parentIn({ record: '486667 (node) S 486601 486667 486667 0 -1 4194560', },);
 ```
 */
function parentIn({ record, }: { readonly record: string },): number | typeof NO_PARENT_RECORD {
  /**
   The fields that follow the program's name, which the kernel writes in parentheses and which may hold a space or a closing
   parenthesis of their own.
   */
  const afterName = record.slice(record.lastIndexOf(')',) + 1,)
    .trim();
  /**
   The number the parent's field holds, read as a number so a record that names none is told from one that does.
   */
  const parent = Number(afterName.split(' ',)[PARENT_FIELD],);
  // A process nothing started has no parent by design, and a record that holds no whole number where the parent goes is no record
  // this reader can use: both end a chain of parents rather than pointing it at a process that never started anything.
  if ((!Number.isSafeInteger(parent,)) || (parent <= NO_PARENT)) {
    return NO_PARENT_RECORD;
  }
  return parent;
}

/**
 Reads the process number of the process that started one process, from the process folder's own record of it.

 A `/proc/<pid>/stat` line starts with the process number, the program's name in parentheses, the process's state and then the
 process number of the one that started it. A program may name itself with a space or a closing parenthesis inside that
 parentheses, so the fields are read from the last `)` rather than the first.

 @param processFolder - folder holding one entry per process, `/proc` here

 @param pid - process number whose parent is wanted

 @returns process number of the process that started it, or {@link NO_PARENT_RECORD} when no record this account can read names one

 @example
 ```ts
 const parent = await parentOfProcess({ processFolder: '/proc', pid: 486667, },);
 ```
 */
export async function parentOfProcess({
  processFolder,
  pid,
}: {
  readonly processFolder: string;
  readonly pid: number;
},): Promise<number | typeof NO_PARENT_RECORD> {
  /**
   The process folder's own record of that process.
   */
  const record = await recordOf({
    processFolder,
    pid,
    name: 'stat',
  },);
  if ('unreadable' in record) {
    return NO_PARENT_RECORD;
  }
  return parentIn({ record: record.text, },);
}

/**
 Reads whether one process is descended from another, following the chain of processes that started it.

 The chain is followed one link at a time, and each link is the record of the process the link before it named, so a process
 another session started is read as that session's however many processes deep it sits.

 @param processFolder - folder holding one entry per process, `/proc` here

 @param pid - process the chain is followed from, which is no descendant of itself

 @param rootPid - process a chain must reach: the run's own, which every process the run started descends from

 @returns whether the chain reaches the root, which is false when it ends first, when one of its records cannot be read, and when a
 record names a process as its own ancestor

 @example
 ```ts
 const own = await descendsFrom({ processFolder: '/proc', pid: 486667, rootPid: 486601, },);
 ```
 */
export async function descendsFrom({
  processFolder,
  pid,
  rootPid,
}: {
  readonly processFolder: string;
  readonly pid: number;
  readonly rootPid: number;
},): Promise<boolean> {
  // A process is no descendant of itself, which is what keeps the run's own process out of the work the run left running.
  if (pid === rootPid) {
    return false;
  }
  /**
   The chain read so far, whose last entry is the process whose parent is read next.
   */
  const chain: number[] = [pid,];
  /**
   The process numbers already followed, so a record that names a process as its own ancestor, or two as each other's, is read
   once rather than followed for ever.
   */
  const followed = new Set<number>([pid,],);
  for (;;) {
    // One link after another by nature: the record read here names the process whose record the next read is of, so these reads
    // cannot be asked at once.
    // oxlint-disable-next-line eslint/no-await-in-loop -- each record names the process whose record is read next; see doc/lint/sequential-awaits.md
    const parent = await parentOfProcess({
      processFolder,
      pid: chain.at(-1,) ?? pid,
    },);
    if ((typeof parent) === 'symbol') {
      return false;
    }
    if (parent === rootPid) {
      return true;
    }
    if (followed.has(parent,)) {
      return false;
    }
    followed.add(parent,);
    chain.push(parent,);
  }
}
