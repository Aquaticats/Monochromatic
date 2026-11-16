/**
 What one process's number says about a claim's holder: whether the process that wrote a claim is still the process that holds its number.

 A number alone cannot answer that. The kernel gives a number to another process once its own holder is gone, and this machine's runs are
 hours long, so a claim that named a process a boot ago may name anything now. The moment a process started is what tells them apart, and the
 kernel reports it for every process in its own statistics.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import type { AClaimOnAnEntry, } from './queue-of-runs.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives one process's number when no process of the account has it.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 The code the operating system gives a file this account may not read.
 */
const NOT_ALLOWED_CODE = 'EACCES';

/**
 The folder a Linux machine keeps one folder per process in, which is where a process's start moment is read.
 */
const PROCESS_FOLDER = '/proc';

/**
 Which field of a process's own statistics holds the moment it started, counted in clock ticks since the machine came up. The first
 field after the process's name is its state, which is the third field of the statistics, so this counts from there.
 */
const START_MOMENT_FIELD = 22;

/**
 The number of the first field of a process's statistics after its name.
 */
const FIRST_FIELD_AFTER_THE_NAME = 3;

/**
 The column a process's statistics separate its name from its other fields with, which the name itself may hold.
 */
const PROCESS_STAT_NAME_END = ')';

/**
 What one process's number says about a claim's holder.
 */
export type HolderOfAClaim =
  | {
    readonly kind: 'there';
    readonly startedAt: string
  }
    | { readonly kind: 'gone' }
    | {
      readonly kind: 'another-process';
      readonly startedAt: string
    }
    | {
      readonly kind: 'unreadable';
      readonly reason: string
    };

/**
 Reads when one process started, from the moment the kernel counted.

 @param pid - the process's number

 @returns the start moment, or nothing when no process of that number is there or its statistics cannot be read

 @example
 ```ts
 const startedAt = await startedAtOf({ pid: process.pid });
 ```
 */
export async function startedAtOf({ pid, }: { readonly pid: number },): Promise<HolderOfAClaim> {
  try {
    if (!processIsThere({ pid, },)) {
      return { kind: 'gone', };
    }
  }
  catch (error) {
    return {
      kind: 'unreadable',
      reason: String(error,),
    };
  }
  /**
   The process's statistics, of which the twenty-second field is the moment it started, or what kept them from being read.
   */
  const statistics = await theStatisticsOf({ pid, },);
  if ('problem' in statistics) {
    return statistics.problem;
  }
  /**
   The text of the statistics.
   */
  const { shown, } = statistics;
  /**
   Where the process's name ends, which is the last closing bracket: the name itself may hold brackets and spaces.
   */
  const nameEnd = shown.lastIndexOf(PROCESS_STAT_NAME_END,);
  if (nameEnd === (-1)) {
    return {
      kind: 'unreadable',
      reason: `the statistics of process ${String(pid,)} hold no name`,
    };
  }
  /**
   The start moment, which is at the field the kernel counts it in.
   */
  const moment = shown.slice(nameEnd + 1,)
    .trim()
    .split(' ',)[START_MOMENT_FIELD - FIRST_FIELD_AFTER_THE_NAME];
  return ((moment === undefined) || (moment === ''))
    ? {
      kind: 'unreadable',
      reason: `the statistics of process ${String(pid,)} name no start moment`,
    }
    : {
      kind: 'there',
      startedAt: moment,
    };
}

/**
 Reads one process's own statistics, or answers what kept them from being read.

 @param pid - the process's number

 @returns the text of the statistics, or what the number says instead

 @throws the error a read gave, other than a missing or unreadable process

 @example
 ```ts
 const statistics = await theStatisticsOf({ pid: process.pid });
 ```
 */
async function theStatisticsOf({ pid, }: { readonly pid: number },): Promise<
  | { readonly shown: string }
    | { readonly problem: HolderOfAClaim }
  > {
  try {
    return {
      shown: await readFile(
        `${PROCESS_FOLDER}/${String(pid,)}/stat`,
        'utf8',
      ),
    };
  }
  catch (error) {
    // A process this account may not read is one that is there: the claim is not this caller's to take from it.
    if (errorCodeOf(error,) === NOT_ALLOWED_CODE) {
      return {
        problem: {
          kind: 'unreadable',
          reason: `process ${String(pid,)} is there and this account may not read it`,
        },
      };
    }
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return { problem: { kind: 'gone', }, };
    }
    throw error;
  }
}

/**
 Says whether the process a claim names is still there.

 @param pid - the process number the claim names

 @returns whether it is there, which a process this account may not signal counts as

 @example
 ```ts
 const there = processIsThere({ pid: 1 });
 ```
 */
export function processIsThere({ pid, }: { readonly pid: number },): boolean {
  // A number this account may not signal still names a live process, so only "no such process" says the holder is gone.
  return !refusedAsGone({ pid, },);
}

/**
 Says whether the kernel answers that no process holds one number.

 @param pid - the process number

 @returns whether the kernel said there is no such process

 @throws the error the kernel gave, other than "no such process"

 @example
 ```ts
 const gone = refusedAsGone({ pid: 1 });
 ```
 */
function refusedAsGone({ pid, }: { readonly pid: number },): boolean {
  try {
    process.kill(
      pid,
      0,
    );
    return false;
  }
  catch (error) {
    return errorCodeOf(error,) === NO_SUCH_PROCESS_CODE;
  }
}

/**
 Reads whether a claim's holder is the process the claim was written by.

 @param claim - the claim, whose process number and start moment are what say which process wrote it

 @returns whether that process still holds it, has gone, or whether its number now belongs to another process

 @example
 ```ts
 const holder = await theHolderOfAClaim({ claim });
 ```
 */
export async function theHolderOfAClaim({
  claim,
}: {
  readonly claim: AClaimOnAnEntry;
},): Promise<HolderOfAClaim> {
  /**
   What the process number says now.
   */
  const now = await startedAtOf({ pid: claim.claim
    .pid, },);
  if (now.kind === 'gone') {
    return { kind: 'gone', };
  }
  if (now.kind === 'unreadable') {
    return now;
  }
  if (claim.claim
    .startedAt
    === undefined) {
    // A claim written before a start moment was recorded names a process this caller cannot tell from a reused number, so it is treated
    // as a holder that may be alive: the entry waits for a person rather than being started a second time under two fillings.
    return {
      kind: 'unreadable',
      reason: `the claim names process ${String(claim.claim
        .pid,)} but no start moment, so it cannot be told from a number reused`,
    };
  }
  return (now.startedAt
    === claim.claim
      .startedAt)
    ? {
      kind: 'there',
      startedAt: now.startedAt,
    }
    : {
      kind: 'another-process',
      startedAt: now.startedAt,
    };
}
