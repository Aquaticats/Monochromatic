/**
 Build-task detection for the buildless-package exemption.
 
 A package defining no build task ships no artifact, so requiring its tests to
 import one would be vacuous rather than merely inconvenient. Keying the
 exemption on the task rather than on file layout makes it self-healing: adding
 a build task re-arms the rule with no change here.
 
 @module
 */

import { readFileSync, } from 'node:fs';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger for build-task probing.
 */
const l = tagged({ tag: 'build-task', },);

/**
 Task file every package declares its mise tasks in.
 */
const TASK_FILE = 'mise.toml';

/**
 Prefix of a TOML table header naming a task.
 */
const TASK_TABLE_PREFIX = 'tasks.';

/**
 Task name that produces the package's artifact.
 */
const BUILD_TASK = 'build';

/**
 Prefix of narrower build task names such as `build:js:node`.
 */
const BUILD_TASK_PREFIX = 'build:';

/**
 Extracts the task name from one TOML table header line.
 
 Recognizes both `[tasks.build]` and the quoted `[tasks."build:js"]` spelling
 that names containing colons require.
 
 @param line - already-trimmed line from the task file
 
 @returns task name, or an empty string when the line is not a task header
 
 @example
 ```ts
 taskNameOfHeader({ line: '[tasks."build:js"]' });
 ```
 
 @internal
 */
export function taskNameOfHeader({ line, }: {
  /**
   Already-trimmed line from the task file.
   */
  readonly line: string;
},): string {
  if ((!line.startsWith('[',)) || (!line.endsWith(']',)))
    return '';
  /**
   Header contents between the brackets.
   */
  const header = line.slice(
    1,
    -1,
  );
  if (!header.startsWith(TASK_TABLE_PREFIX,))
    return '';
  /**
   Task name, still carrying quotes when the name needed them.
   */
  const quoted = header.slice(TASK_TABLE_PREFIX.length,);
  if (quoted.startsWith('"',)
    && quoted.endsWith('"',)
    && (quoted.length >= 2))
    {
    return quoted.slice(
      1,
      -1,
    );
  }
  return quoted;
}

/**
 Tests whether a task name produces build output.
 
 @param name - task name from a table header
 
 @returns true for `build` and for any `build:` scoped task
 
 @example
 ```ts
 isBuildTaskName({ name: 'build:js:node' });
 ```
 
 @internal
 */
export function isBuildTaskName({ name, }: {
  /**
   Task name from a table header.
   */
  readonly name: string;
},): boolean {
  return (name === BUILD_TASK) || name.startsWith(BUILD_TASK_PREFIX,);
}

/**
 Reads one package's task file, treating an unreadable file as empty.
 
 A package without a task file declares no tasks at all, which is exactly the
 buildless case the exemption targets, so absence needs no separate signal.
 
 @param taskFile - absolute path of the task file to read
 
 @returns file contents, or an empty string when the file cannot be read
 
 @example
 ```ts
 readTaskFile({ taskFile: '/repo/package/module/x/mise.toml' });
 ```
 */
function readTaskFile({ taskFile, }: {
  /**
   Absolute path of the task file to read.
   */
  readonly taskFile: string;
},): string {
  try {
    /* oxlint-disable no-restricted-syntax/no-sync -- Oxlint rule visitors run synchronously and expose no async hook to await a read from. */
    return readFileSync(
      taskFile,
      'utf8',
    );
    /* oxlint-enable no-restricted-syntax/no-sync */
  }
  catch (error: unknown) {
    l.debug(`no task file at ${taskFile}: ${String(error,)}`,);
    return '';
  }
}

/**
 Reads a package's task file and reports whether it declares any build task.
 
 @param packageRoot - absolute package root to probe
 
 @returns true when the task file declares `build` or any `build:` task
 
 @example
 ```ts
 declaresBuildTask({ packageRoot: '/repo/package/module/x' });
 ```
 
 @internal
 */
export function declaresBuildTask({ packageRoot, }: {
  /**
   Absolute package root to probe.
   */
  readonly packageRoot: string;
},): boolean {
  /**
   Task file contents for this package, empty when the package declares no tasks.
   */
  const text = readTaskFile({
    taskFile: join(
      packageRoot,
      TASK_FILE,
    ),
  },);

  return text.split('\n',)
    .some(function declaresBuild(rawLine,): boolean {
      return isBuildTaskName({
        name: taskNameOfHeader({ line: rawLine.trim(), },),
      },);
    },);
}
