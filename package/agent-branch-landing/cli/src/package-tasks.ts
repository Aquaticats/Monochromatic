/**
 The tasks one package's `mise.toml` declares, and which of them proves the least while still proving that the merge with `main`
 fits.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { CHEAPEST_CHECK_TASKS, } from './constants.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';

/**
 The text that opens a task header in a `mise.toml`.
 */
const TASK_HEADER_OPENING = '[tasks.';

/**
 Reads the task name one line of a `mise.toml` declares.

 A quoted header is `[tasks."lint:types"]`; a bare one is `[tasks.lint]` or `[tasks.lint:types]`. A bare name holding a dot declares
 a table below a task rather than a task, so it is not a task name.

 @param line - one line of the file

 @returns the task name it declares, or why the line declares none

 @example
 ```ts
 const name = taskNameOf('[tasks."lint:types"]',);
 ```
 */
function taskNameOf(line: string,): Looked<string> {
  /**
   The line without the spaces around it, which a configuration file may indent.
   */
  const trimmed = line.trim();
  if ((!trimmed.startsWith(TASK_HEADER_OPENING,)) || (!trimmed.endsWith(']',))) {
    return missing('a line that declares no task',);
  }
  /**
   What the header holds between its brackets.
   */
  const inner = trimmed.slice(
    TASK_HEADER_OPENING.length,
    -1,
  );
  if ((inner.length > 1) && inner.startsWith('"',)
    && inner.endsWith('"',)) {
    return found(inner.slice(
      1,
      -1,
    ),);
  }
  if (inner.includes('"',) || inner.includes('.',)) {
    return missing('a header naming a table below a task',);
  }
  return inner === '' ? missing('a header naming no task',) : found(inner,);
}

/**
 Reads the names of the tasks a `mise.toml` declares.

 @param miseToml - the file's text

 @returns the declared task names

 @example
 ```ts
 const declared = tasksDeclaredIn('[tasks.lint]\nextends = "lint"\n',);
 ```
 */
export function tasksDeclaredIn(miseToml: string,): ReadonlySet<string> {
  return new Set(miseToml.split('\n',)
    .flatMap(function nameOf(line: string,) {
      /**
       What this line declares.
       */
      const name = taskNameOf(line,);
      return name.found ? [name.value,] : [];
    },),);
}

/**
 Picks the task that proves the least while still proving the merged tree fits, from the names one package declares.

 Owner, 2026-11-10: "I don't think you'd need to rerun the same lints or tests another agent did, do you?" The agent ran its own
 suites; what it could not run is its work combined with what `main` gained since.

 @param declared - the task names the package declares

 @returns the cheapest declared task, or why none of them is declared

 @example
 ```ts
 const cheapest = cheapestCheckTaskAmong(new Set(['lint:types', 'test',],),);
 ```
 */
export function cheapestCheckTaskAmong(declared: ReadonlySet<string>,): Looked<string> {
  /**
   The first of the cheapest tasks the package declares.
   */
  const foundTask = CHEAPEST_CHECK_TASKS.find(function isDeclared(task: string,) {
    return declared.has(task,);
  },);
  return foundTask === undefined
    ? missing(`none of ${CHEAPEST_CHECK_TASKS.join(', ')} is declared there`,)
    : found(foundTask,);
}

/**
 Reads one package's declared tasks from its `mise.toml` in a worktree.

 @param worktree - the worktree the package sits in

 @param packagePath - the package's repository-relative path

 @returns the declared task names

 @throws LandingConfigurationError when the package has no readable `mise.toml`

 @example
 ```ts
 const declared = await declaredTasksOf({ worktree: '/repo/.claude/worktrees/a', packagePath: 'package/a/b', },);
 ```
 */
export async function declaredTasksOf({
  worktree,
  packagePath,
}: {
  readonly worktree: string;
  readonly packagePath: string;
},): Promise<ReadonlySet<string>> {
  try {
    return tasksDeclaredIn(await readFile(
      join(
        worktree,
        packagePath,
        'mise.toml',
      ),
      'utf8',
    ),);
  }
  catch (caught) {
    throw new LandingConfigurationError(
      `${packagePath} has no readable mise.toml (${caughtValueText(caught,)}), so no task there could check the branch`,
    );
  }
}

/**
 Names the tasks that would check the packages a branch changed, or none when no changed package declares one.

 @param worktree - the worktree the packages sit in

 @param packages - each changed package's repository-relative path

 @returns the repository-qualified task names to run, each once

 @example
 ```ts
 const tasks = await checkTasksFor({ worktree, packages: ['package/a/b',], },);
 ```
 */
export async function checkTasksFor({
  worktree,
  packages,
}: {
  readonly worktree: string;
  readonly packages: readonly string[];
},): Promise<readonly string[]> {
  /**
   The cheapest task of each changed package, as one list per package.
   */
  const chosen: readonly (readonly string[])[] = await Promise.all(packages.map(
    async function taskOf(packagePath: string,) {
      /**
       The task the package declares first among the cheapest ones.
       */
      const cheapest = cheapestCheckTaskAmong(await declaredTasksOf({
        worktree,
        packagePath,
      },),);
      return cheapest.found ? [`//${packagePath}:${cheapest.value}`,] : [];
    },
  ),);
  return [...new Set(chosen.flat(),),];
}
