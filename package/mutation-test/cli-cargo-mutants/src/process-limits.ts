/**
 What the test run may start: the process limit it runs under, and how wide it can go inside that limit.

 A Linux control group counts threads as well as processes (`pids.max`), and a Node process holds about a dozen of them,
 so a limit such as systemd's `TasksMax=300` allows far fewer test processes than the CPU count suggests. A start that
 fails against the limit does not say so: Node reports `spawn ... EAGAIN` or aborts with a failed thread creation, and a
 test that only checks an exit status then reports a bare status 1. The test run therefore sizes itself to the limit and
 names the limit when a program still fails against it.
 */
import { readFileSync } from 'node:fs';
import { availableParallelism } from 'node:os';
import { dirname } from 'node:path';

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
 readSetting('/sys/fs/cgroup/pids.max');
 ```
 */
function readSetting(path: string): readonly string[] {
  try {
    return [readFileSync(
      path,
      'utf8',
    )
      .trim()];
  }
  catch (error) {
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
 nearestPidsLimit('/sys/fs/cgroup/user.slice/run-p1.scope');
 ```
 */
export function nearestPidsLimit(directory: string): readonly PidsLimit[] {
  if (!directory.startsWith(cgroupRoot))
    return [];
  const [text] = readSetting(`${directory}/pids.max`);
  const limit = Number(text);
  if (Number.isInteger(limit) && (limit > 0))
    return [{
      directory,
      limit,
    }];
  return directory === cgroupRoot ? [] : nearestPidsLimit(dirname(directory));
}

/**
 Find the process limit of the running process.

 @returns the limit as the only element; no element when none applies

 @example
 ```ts
 ownPidsLimit();
 ```
 */
export function ownPidsLimit(): readonly PidsLimit[] {
  const [membership] = readSetting('/proc/self/cgroup');
  /**
   The control group line of the unified hierarchy.
   */
  const path = membership?.split('\n')
    .find((line) => line.startsWith('0::'))
    ?.slice('0::'.length);
  return path === undefined ? [] : nearestPidsLimit(`${cgroupRoot}${path}`);
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
  return Math.max(
    1,
    Math.min(
      processors,
      Math.floor(limit * plannedShare / (testFileTasks + campaignCaseTasks)),
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
  return Math.max(
    1,
    Math.min(
      wanted,
      Math.floor((limit * plannedShare / workers - testFileTasks) / campaignCaseTasks),
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
  const told = Number(process.env[fileWorkersVariable]);
  return Number.isInteger(told) && (told > 0) ? told : availableParallelism();
}

/**
 Tell how many campaign cases one test file of this run may run at once.

 @param wanted - how many cases a file runs at once when nothing limits it

 @returns the width inside the process limit the running process is under, as {@link casesWithin} computes it

 @example
 ```ts
 casesWithinOwnLimit(4);
 ```
 */
export function casesWithinOwnLimit(wanted: number): number {
  const [found] = ownPidsLimit();
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
 processLimitMessage('node');
 ```
 */
export function processLimitMessage(program: string): string {
  const [found] = ownPidsLimit();
  const where = found === undefined
    ? 'no control group sets a limit, so the per-user limit (`ulimit -u`) or the host is the cause'
    : `${found.directory}/pids.max is ${String(found.limit)} tasks, threads included`;
  return `${program} could not start a process or a thread, so the process limit was reached: ${where}. `
    + `Raise the limit (systemd \`TasksMax=\`), or lower the number of test files and cases that run at once; `
    + `the runner sizes both from the limit, and ${fileWorkersVariable} overrides the number of files.`;
}
