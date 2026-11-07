/**
 Which checks the daily update runs for which packages.

 Rationale: `doc/planning/toolchain-always-latest.md` ("Checking every package after the update").

 @module
 */

import {
  RUST_CHECK_TIMEOUT_MS,
  TYPESCRIPT_CHECK_TIMEOUT_MS,
} from './daily-update-check-bounds.ts';
import type {
  CheckKind,
  PackageCheck,
} from './daily-update-package-check-types.ts';

/**
 The `mise` task that compiles a Rust crate and runs Clippy over it.
 */
export const RUST_CHECK_TASK = 'lint:clippy';

/**
 The `mise` task that type-checks a TypeScript package.
 */
export const TYPESCRIPT_CHECK_TASK = 'lint:types';

/**
 What a task counts as and how long it may run.
 */
type CheckTask = {
  /**
   Kind of package the task checks.
   */
  readonly kind: CheckKind;
  /**
   Longest the task may run, in milliseconds.
   */
  readonly timeoutMs: number;
};

/**
 Kind and time bound of each task that counts as a check.
 */
const CHECK_TASKS: ReadonlyMap<string, CheckTask> = new Map([
  [
    RUST_CHECK_TASK,
    {
      kind: 'rust',
      timeoutMs: RUST_CHECK_TIMEOUT_MS,
    },
  ],
  [
    TYPESCRIPT_CHECK_TASK,
    {
      kind: 'typescript',
      timeoutMs: TYPESCRIPT_CHECK_TIMEOUT_MS,
    },
  ],
],);

/**
 Every package task name starts with this: two slashes and the `package` folder.
 */
const PACKAGE_TASK_PREFIX = '//package/';

/**
 Segments of a package directory: the folder `package`, a category and a name.
 */
const PACKAGE_PATH_SEGMENTS = 3;

/**
 Separates a package directory from its task in a task name.
 */
const TASK_SEPARATOR = ':';

/**
 Picks the checks out of the task names `mise tasks ls --all` prints.

 @param taskNames - names such as `//package/a/b:lint:clippy`; root tasks and tasks that are no check are ignored

 @returns one check per package and check task, ordered by package and then task

 @example
 ```ts
 checksFromTaskNames({ taskNames: ['//package/a/b:lint:clippy', '//package/a/b:build',], },); // one Rust check
 ```
 */
export function checksFromTaskNames({
  taskNames,
}: {
  readonly taskNames: readonly string[];
},): readonly PackageCheck[] {
  /**
   Checks found so far.
   */
  const checks: PackageCheck[] = [];
  for (const taskName of taskNames) {
    if (!taskName.startsWith(PACKAGE_TASK_PREFIX,))
      continue;
    /**
     Position of the colon that ends the package directory.
     */
    const separator = taskName.indexOf(TASK_SEPARATOR,);
    if (separator === (-1))
      continue;
    /**
     Package directory and task name.
     */
    const directory = taskName.slice(
      '//'.length,
      separator,
    );
    /**
     Task name after the separator.
     */
    const task = taskName.slice(separator + TASK_SEPARATOR.length,);
    /**
     What the task counts as; absent for a task that is no check.
     */
    const checkTask = CHECK_TASKS.get(task,);
    if ((checkTask !== undefined) && (directory.split('/',)
      .length
      === PACKAGE_PATH_SEGMENTS)) {
      checks.push({
        package: directory,
        kind: checkTask.kind,
        task,
        timeoutMs: checkTask.timeoutMs,
      },);
    }
  }
  return checks.toSorted(function byPackageThenTask(
    left,
    right,
  ) {
    return `${left.package}:${left.task}`.localeCompare(`${right.package}:${right.task}`,);
  },);
}

/**
 The command that runs a check, as a person types it at the repository root.

 @param check - the check

 @returns the command line

 @example
 ```ts
 reproduceCommand({ package: 'package/a/b', kind: 'rust', task: 'lint:clippy', timeoutMs: 1, },); // 'mise run //package/a/b:lint:clippy'
 ```
 */
export function reproduceCommand(check: PackageCheck,): string {
  return `mise run //${check.package}:${check.task}`;
}
