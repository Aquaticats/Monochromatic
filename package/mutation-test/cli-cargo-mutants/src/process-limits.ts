/**
 What the test run may start: the process limit it runs under, and how wide it can go inside that limit.

 A Linux control group counts threads as well as processes (`pids.max`), and a Node process holds about a dozen of them,
 so a limit such as systemd's `TasksMax=300` allows far fewer test processes than the CPU count suggests. A start that
 fails against the limit does not say so: Node reports `spawn ... EAGAIN` or aborts with a failed thread creation, and a
 test that only checks an exit status then reports a bare status 1. The shared test runner caps how many test files run
 at once from a per-file estimate that this package's `mise.toml` gives it, and tells each file the task budget of the run;
 this module sizes the campaign cases of one file from the same numbers, and names the limit when a program still fails
 against it. The numbers have one source, the `test:unit` task's environment in this package's `mise.toml`.
 */
import { readFile } from 'node:fs/promises';
import { dirname } from 'node:path';

import { isMissing } from './fs-guards.ts';

/**
 Root of the control group file system.
 */
const cgroupRoot = '/sys/fs/cgroup';

/**
 Name of the environment variable in which the runner tells a test file how many test files run at once.
 */
export const fileWorkersVariable = 'MONOCHROMATIC_TEST_FILE_WORKERS';

/**
 Name of the environment variable in which the runner tells a test file how many tasks the whole run may hold.
 */
export const taskBudgetVariable = 'MONOCHROMATIC_TEST_TASK_BUDGET';

/**
 Name of the environment variable, set by this package's `test:unit` task, for the tasks one test file holds at the least.
 */
export const fileTasksVariable = 'MONOCHROMATIC_TEST_FILE_TASKS';

/**
 Name of the environment variable, set by this package's `test:unit` task, for the tasks one campaign case holds at its peak.
 */
export const caseTasksVariable = 'MONOCHROMATIC_TEST_CASE_TASKS';

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
 Tell how many campaign cases one test file may run at once inside the task budget of the run.

 @param budget - tasks the whole run may hold, absent when the runner gave none

 @param workers - how many test files run at once

 @param fileTasks - tasks a test file holds at the least, which includes one case

 @param caseTasks - tasks one case holds at its peak

 @param wanted - how many cases a file runs at once when nothing limits it

 @returns the wanted width without a budget; otherwise the widest that keeps every file within its share, and at least one

 @example
 ```ts
 casesWithin({ budget: 240, workers: 5, fileTasks: 46, caseTasks: 30, wanted: 4 });
 ```
 */
export function casesWithin({
  budget,
  workers,
  fileTasks,
  caseTasks,
  wanted,
}: {
  readonly budget?: number;
  readonly workers: number;
  readonly fileTasks: number;
  readonly caseTasks: number;
  readonly wanted: number;
}): number {
  if (budget === undefined)
    return wanted;
  /**
   Tasks one file may hold.
   */
  const share = budget / workers;
  /**
   Cases that fit in a file's share beside the file's own tasks, which are the file's minimum less its one case.
   */
  const fitting = Math.floor((share - (fileTasks - caseTasks)) / caseTasks);
  return Math.max(
    1,
    Math.min(
      wanted,
      fitting,
    ),
  );
}

/**
 Read a positive integer from the environment.

 @param name - variable name

 @returns the number as the only element; no element when the variable is missing or not a positive integer

 @example
 ```ts
 positiveFromEnvironment('MONOCHROMATIC_TEST_FILE_WORKERS');
 ```
 */
function positiveFromEnvironment(name: string): readonly number[] {
  /**
   The variable as a number.
   */
  const value = Number(process.env[name]);
  return Number.isInteger(value) && (value > 0) ? [value] : [];
}

/**
 Tell how many campaign cases one test file of this run may run at once.

 @param wanted - how many cases a file runs at once when nothing limits it

 @returns the wanted width when the runner gave no budget (a file run by hand, or a run without a process limit); otherwise
 the width {@link casesWithin} computes from the runner's numbers and this package's estimates

 @example
 ```ts
 casesWithinRun(4);
 ```
 */
export function casesWithinRun(wanted: number): number {
  /**
   Tasks the whole run may hold.
   */
  const [budget] = positiveFromEnvironment(taskBudgetVariable);
  /**
   Test files running at once.
   */
  const [workers] = positiveFromEnvironment(fileWorkersVariable);
  /**
   Tasks a file holds at the least.
   */
  const [fileTasks] = positiveFromEnvironment(fileTasksVariable);
  /**
   Tasks one case holds at its peak.
   */
  const [caseTasks] = positiveFromEnvironment(caseTasksVariable);
  if (budget === undefined)
    return wanted;
  if (workers === undefined)
    return wanted;
  if (fileTasks === undefined)
    return wanted;
  if (caseTasks === undefined)
    return wanted;
  return casesWithin({
    budget,
    workers,
    fileTasks,
    caseTasks,
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
    + `the runner sizes the files from \`${fileTasksVariable}\` and the limit.`;
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
