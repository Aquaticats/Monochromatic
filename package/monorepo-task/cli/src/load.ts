/**
 Task file loading for `@monochromatic-dev/monorepo-task-cli`: imports root
 and package `task.ts` modules and resolves `extends` onto root templates.
 */

import { stat, } from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';
import { runTaskProcess, } from './task-process.ts';
import {
  packageTasksFromModule,
  rootTasksFromModule,
} from './task-file-validation.ts';
import type {
  PackageTaskFile,
  RootTaskFile,
  TaskDefinition,
} from './types.ts';

/**
 Monorepo root from this module's location inside `package/monorepo-task/cli`.
 */
const REPO_ROOT = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..'
);

/**
 Cached root task file for the current process.
 */
const rootTaskFiles = new Map<string, RootTaskFile>();

/**
 Returns the monorepo root this loader resolves task files against.

 @returns Absolute monorepo root path.

 @example
 ```ts
 const root = repoRoot();
 ```
 */
export function repoRoot(): string {
  return REPO_ROOT;
}

/**
 Loads and returns the root `task.ts`.

 @param root - Repository whose templates must be loaded.

 @example
 ```ts
 const tasks = await loadRootTasks(root);
 ```

 @returns Root templates and tasks.
 
 @throws When the root task file is missing or malformed.
 */
export async function loadRootTasks(root: string = REPO_ROOT,): Promise<RootTaskFile> {
  /**
   Root templates are loaded once per repository in this process.
   */
  const cached = rootTaskFiles.get(root,);
  if (cached !== undefined)
    return cached;
  /**
   The task file belongs to the caller-selected repository root.
   */
  const path = join(
    root,
    'task.ts'
  );
  if (await stat(
    path,
    { throwIfNoEntry: false }
  ) === undefined)
    throw new Error(`task-cli: missing root task.ts`);
  /**
   Repository-owned task modules supply their declared default export.
   */
  const namespace: unknown = await import(pathToFileURL(path)
    .href);
  /**
   Validate dynamically loaded declarations before exposing them to task dispatch.
   */
  const rootTasks = rootTasksFromModule({
    namespace,
    path
  });
  rootTaskFiles.set(
    root,
    rootTasks,
  );
  return rootTasks;
}

/**
 Loads one package `task.ts` file.

 @example
 ```ts
 const tasks = await loadPackageTasks(path);
 ```

 @param path - Absolute path of the package task file.
 
 @returns Task definitions from the file.
 
 @throws When the file is missing or malformed.
 */
export async function loadPackageTasks(path: string): Promise<PackageTaskFile> {
  if (await stat(
    path,
    { throwIfNoEntry: false }
  ) === undefined)
    throw new Error(`task-cli: missing task file: ${path}`);
  /**
   Repository-owned task modules supply their declared default export.
   */
  const namespace: unknown = await import(pathToFileURL(path)
    .href);
  return packageTasksFromModule({
    namespace,
    path
  });
}

/**
 Resolves a task definition through `extends` onto a root template.

 @example
 ```ts
 const effective = await resolveDefinition({ definition, root });
 ```

 @param definition - Task definition possibly carrying `extends`.
 
 @returns The definition with template fields filled in.
 
 @throws When the named template does not exist.
 */
export async function resolveDefinition(
  {
    definition,
    root: repositoryRoot = REPO_ROOT,
  }: {
    readonly definition: TaskDefinition;
    readonly root?: string
  },
): Promise<TaskDefinition> {
  if (definition.extends === undefined)
    return definition;
  /**
   Templates come from the same repository as the requested task.
   */
  const root = await loadRootTasks(repositoryRoot,);
  /**
   Missing template names receive a task-specific diagnostic before execution.
   */
  const template = root.templates[definition.extends];
  if (template === undefined)
    throw new Error(`task-cli: unknown task template: ${definition.extends}`);
  return {
    ...template,
    ...definition,
    depends: [
      ...template.depends ?? [],
      ...definition.depends ?? [],
    ],
  };
}

/**
 Runs this repository's task CLI for task bodies that call other tasks.

 @example
 ```ts
 await runTaskCli({ name: '//package/config/oxlint:build:js:node' });
 ```

 @param name - Task name like `//package/config/oxlint:build:js:node`.
 
 @param args - Option and passthrough arguments.
 
 @throws When the child task fails.
 */
export async function runTaskCli({
  name,
  args = [],
}: {
  readonly name: string;
  readonly args?: readonly string[];
}): Promise<void> {
  await runTaskProcess({
    command: process.execPath,
    args: [
      join(
        REPO_ROOT,
        'package',
        'monorepo-task',
        'cli',
        'src',
        'run.ts'
      ),
      name,
      ...args,
    ],
    cwd: REPO_ROOT,
    env: {
      ...process.env,
      TASK_CLI_CHILD: '1',
    },
  });
}
