/**
 Whether the process that took a review's folder still runs, so a folder left by a start that was killed can be taken again while
 a folder held by a live start is left alone.

 A process is known by what the kernel says of it: its number, the boot it runs in, and the moment it started. A number can be used
 again by a later process, so the number alone never proves the same process; the boot and the start time do. The package runs
 where `/proc` answers, and an entry that cannot be read is one whose process cannot be shown to run.

 @module
 */

import { readFileOrNothing, } from './finished-worktrees.ts';

/**
 What a process is on this machine: its number, the boot it runs in, and the moment it started, as the kernel counts it.
 */
export type ProcessIdentity = {
  /**
   The process's number.
   */
  readonly pid: number;

  /**
   The identifier of the boot the process runs in.
   */
  readonly bootId: string;

  /**
   The moment the process started, in the kernel's own ticks since that boot.
   */
  readonly startTicks: string;
};

/**
 The states the kernel reports for a process that has ended but has not been reaped.
 */
const ENDED_STATES: ReadonlySet<string> = new Set<string>([
  'Z',
  'X',
  'x',
]);

/**
 The place of a process's start time among the fields that follow its program's name: the state is the entry's third field, so the
 twenty-second field is the twentieth after the name.
 */
const START_TIME_AFTER_NAME = 19;

/**
 Reads the identity of the process that asks.

 @returns its number, its boot, and its start time

 @example
 ```ts
 const identity = await processIdentityOfThisProcess();
 ```
 */
export async function processIdentityOfThisProcess(): Promise<ProcessIdentity> {
  /**
   What the kernel says of this process.
   */
  const stat = await readFileOrNothing(`/proc/${String(process.pid,)}/stat`,);
  return {
    pid: process.pid,
    bootId: await bootIdNow(),
    startTicks: startTicksIn({ stat, },),
  };
}

/**
 Reads the identifier of the boot this machine runs in.

 @returns the identifier, or empty text when it cannot be read

 @example
 ```ts
 const bootId = await bootIdNow();
 ```
 */
async function bootIdNow(): Promise<string> {
  return (await readFileOrNothing('/proc/sys/kernel/random/boot_id',)).trim();
}

/**
 Reads the fields of a process's stat entry that follow the program's name: the state, then the numbers the kernel keeps.

 @param stat - the entry's text

 @returns its fields, the state first

 @example
 ```ts
 const fields = fieldsAfterTheName({ stat: '1 (a b) S 0 0 0', });
 ```
 */
function fieldsAfterTheName({ stat, }: { readonly stat: string },): readonly string[] {
  /**
   Where the program's name, which may hold spaces, ends.
   */
  const close = stat.lastIndexOf(')',);
  return (close === (-1))
    ? []
    : stat.slice(close + 1,)
      .trim()
      .split(' ',);
}

/**
 Reads the state a process's stat entry reports.

 @param stat - the entry's text

 @returns the state, or empty text when the entry is unreadable

 @example
 ```ts
 const state = stateIn({ stat: '1 (a) S 0 0', });
 ```
 */
function stateIn({ stat, }: { readonly stat: string },): string {
  return fieldsAfterTheName({ stat, },)[0] ?? '';
}

/**
 Reads the moment a process started, as its stat entry counts it.

 @param stat - the entry's text

 @returns the start time, or empty text when the entry is unreadable

 @example
 ```ts
 const ticks = startTicksIn({ stat: '1 (a) S 0 0', });
 ```
 */
function startTicksIn({ stat, }: { readonly stat: string },): string {
  return fieldsAfterTheName({ stat, },)[START_TIME_AFTER_NAME] ?? '';
}

/**
 Says whether the process a folder names still runs.

 @param identity - the number, boot and start time the folder recorded

 @returns whether that exact process still runs

 @example
 ```ts
 const runs = await identityStillRuns({ identity },);
 ```
 */
export async function identityStillRuns({ identity, }: {
  readonly identity: ProcessIdentity;
},): Promise<boolean> {
  if ((await bootIdNow()) !== identity.bootId) {
    return false;
  }
  /**
   What the kernel now says of that number.
   */
  const stat = await readFileOrNothing(`/proc/${String(identity.pid,)}/stat`,);
  if (stat === '') {
    return false;
  }
  if (ENDED_STATES.has(stateIn({ stat, },))) {
    return false;
  }
  return startTicksIn({ stat, },) === identity.startTicks;
}
