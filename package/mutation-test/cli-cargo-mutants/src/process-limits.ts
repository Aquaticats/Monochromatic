/**
 What the test run may start: the process limit it runs under, and how wide it can go inside that limit.

 A Linux control group counts threads as well as processes (`pids.max`), and a Node process holds about a dozen of them,
 so a limit such as systemd's `TasksMax=300` allows far fewer test processes than the CPU count suggests. A start that
 fails against the limit does not say so: Node reports `spawn ... EAGAIN` or aborts with a failed thread creation, and a
 test that only checks an exit status then reports a bare status 1. The test run therefore sizes itself to the limit and
 names the limit when a program still fails against it.
 */
import { readFile } from 'node:fs/promises';
import { availableParallelism } from 'node:os';
import { dirname } from 'node:path';

import { isMissing } from './fs-guards.ts';

/**
 Root of the control group file system.
 */
const cgroupRoot = '/sys/fs/cgroup';

/**
 Share of the limit the run plans to use, which leaves the rest for the runner, the task launcher and the shell around it.
 */
export const plannedShare = 0.8;

/**
 Tasks one test file's own Node process holds, measured as the peak of a file that starts nothing.
 */
export const testFileTasks = 16;

/**
 Tasks one campaign case holds at its peak: the campaign script, the stand-in program it is waiting on and their helpers.
 */
export const campaignCaseTasks = 30;

/**
 Name of the environment variable in which the test runner tells the test files how many run at once.
 */
export const fileWorkersVariable = 'MONOCHROMATIC_TEST_FILE_WORKERS';

/**
 The process limit that applies to this process.
 */
export type PidsLimit = {
  /**
   Control group directory that sets the limit.
   */
  readonly directory: string;
  /**
   Most tasks the group may hold.
   */
  readonly limit: number;
};

/**
 Read a file as text.

 @param path - the file

 @returns its text with the final newline removed, as the only element; no element when it cannot be read

 @example
 ```ts
 await readSetting('/sys/fs/cgroup/pids.max');
 ```
 */
async function readSetting(path: string): Promise<readonly string[]> {
  try {
    return [(await readFile(
      path,
      'utf8',
    ))
      .trim()];
  }
  catch (error) {
    // A group without the setting (the root group has no `pids.max`) or a host without control groups is not an error.
    if (!isMissing(error))
      console.error(`Cannot read ${path}: ${String(error)}`);
    return [];
  }
}

/**
 Find the nearest process limit at or above a control group.

 The nearest one is used because the limits above it (the user's slice, the whole session) are shared with other work and
 say nothing about what this run may use.

 @param directory - the control group directory to start from

 @returns the limit as the only element; no element when no group up to the root sets one

 @example
 ```ts
 await nearestPidsLimit('/sys/fs/cgroup/user.slice/run-p1.scope');
 ```
 */
export async function nearestPidsLimit(directory: string): Promise<readonly PidsLimit[]> {
  if (!directory.startsWith(cgroupRoot))
    return [];
  /**
   The group's setting, which is `max` when it sets no limit.
   */
  const [text] = await readSetting(`${directory}/pids.max`);
  /**
   The setting as a number, which is not an integer for `max`.
   */
  const limit = Number(text);
  if (Number.isInteger(limit) && (limit > 0))
    return [{
      directory,
      limit,
    }];
  return directory === cgroupRoot ? [] : await nearestPidsLimit(dirname(directory));
}

/**
 Find the process limit of the running process.

 @returns the limit as the only element; no element when none applies

 @example
 ```ts
 await ownPidsLimit();
 ```
 */
export async function ownPidsLimit(): Promise<readonly PidsLimit[]> {
  /**
   The control group membership lines of this process.
   */
  const [membership] = await readSetting('/proc/self/cgroup');
  /**
   The control group line of the unified hierarchy.
   */
  const line = membership?.split('\n')
    .find(function isUnified(entry) {
      return entry.startsWith('0::');
    });
  /**
   The group's path below the root.
   */
  const path = line?.slice('0::'.length);
  return path === undefined ? [] : await nearestPidsLimit(`${cgroupRoot}${path}`);
}

/**
 Tell how many test files may run at once inside a process limit.

 @param limit - the limit in tasks, absent when there is none

 @param processors - how many CPUs the run may use

 @returns the processor count without a limit; otherwise as many files as fit, with their cases, in the planned share

 @example
 ```ts
 fileWorkersWithin({ limit: 300, processors: 28 });
 ```
 */
export function fileWorkersWithin({
  limit,
  processors,
}: {
  readonly limit?: number;
  readonly processors: number;
}): number {
  if (limit === undefined)
    return processors;
  /**
   Files that fit, each with one case.
   */
  const fitting = Math.floor((limit * plannedShare) / (testFileTasks + campaignCaseTasks));
  return Math.max(
    1,
    Math.min(
      processors,
      fitting,
    ),
  );
}

/**
 Tell how many campaign cases one test file may run at once inside a process limit.

 @param limit - the limit in tasks, absent when there is none

 @param workers - how many test files run at once

 @param wanted - how many cases a file runs at once when nothing limits it

 @returns the wanted width without a limit; otherwise the widest that keeps every file within its share, and at least one

 @example
 ```ts
 casesWithin({ limit: 300, workers: 4, wanted: 4 });
 ```
 */
export function casesWithin({
  limit,
  workers,
  wanted,
}: {
  readonly limit?: number;
  readonly workers: number;
  readonly wanted: number;
}): number {
  if (limit === undefined)
    return wanted;
  /**
   Tasks one file may hold.
   */
  const share = (limit * plannedShare) / workers;
  /**
   Cases that fit in a file's share beside the file itself.
   */
  const fitting = Math.floor((share - testFileTasks) / campaignCaseTasks);
  return Math.max(
    1,
    Math.min(
      wanted,
      fitting,
    ),
  );
}

/**
 Tell how many test files the runner started at once.

 @returns the runner's number from the environment, or the processors this run may use when the runner did not say

 @example
 ```ts
 fileWorkers();
 ```
 */
export function fileWorkers(): number {
  /**
   The runner's number.
   */
  const told = Number(process.env[fileWorkersVariable]);
  return Number.isInteger(told) && (told > 0) ? told : availableParallelism();
}

/**
 Tell how many campaign cases one test file of this run may run at once.

 @param wanted - how many cases a file runs at once when nothing limits it

 @returns the width inside the process limit the running process is under, as {@link casesWithin} computes it

 @example
 ```ts
 await casesWithinOwnLimit(4);
 ```
 */
export async function casesWithinOwnLimit(wanted: number): Promise<number> {
  /**
   The limit that applies, as the only element.
   */
  const [found] = await ownPidsLimit();
  return found === undefined
    ? wanted
    : casesWithin({
      limit: found.limit,
      workers: fileWorkers(),
      wanted,
    });
}

/**
 Explain a failure to start a process or a thread.

 @param program - the program that failed

 @returns a message that names the process limit and what to do about it

 @example
 ```ts
 await processLimitMessage('node');
 ```
 */
export async function processLimitMessage(program: string): Promise<string> {
  /**
   The limit that applies, as the only element.
   */
  const [found] = await ownPidsLimit();
  /**
   Where the limit is set, or why it is not known.
   */
  const where = found === undefined
    ? 'no control group sets a limit, so the per-user limit (`ulimit -u`) or the host is the cause'
    : `${found.directory}/pids.max is ${String(found.limit)} tasks, threads included`;
  return `${program} could not start a process or a thread, so the process limit was reached: ${where}. `
    + `Raise the limit (systemd \`TasksMax=\`), or lower the number of test files and cases that run at once; `
    + `the runner sizes both from the limit, and ${fileWorkersVariable} overrides the number of files.`;
}

/**
 Words in the output of a program that failed to start a process or a thread: Node's failed `spawn`, its failed thread
 creation, and the C library's reason.
 */
const processLimitWords: readonly string[] = [
  'EAGAIN',
  'uv_thread_create',
  'pthread_create',
  'Resource temporarily unavailable',
];

/**
 Tell whether a program's output shows that it could not start a process or a thread.

 @param text - the program's standard error, or the text of an error

 @returns true when any of the telltale words is in it

 @example
 ```ts
 mentionsProcessLimit('Error: spawn podman EAGAIN');
 ```
 */
export function mentionsProcessLimit(text: string): boolean {
  return processLimitWords.some(function appearsIn(word) {
    return text.includes(word);
  });
}
