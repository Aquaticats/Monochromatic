/**
 Task file loading for `@monochromatic-dev/monorepo-task-cli`: imports root
 and package `task.ts` modules and resolves `extends` onto root templates.
 */

import { execFileSync, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';
import type {
  PackageTaskFile,
  RootTaskFile,
  TaskDefinition,
} from './types.ts';

/**
 Monorepo root from this module's location inside `package/monorepo-task/cli`.
 */
const REPO_ROOT = resolve(import.meta.dirname, '..', '..', '..', '..');

/**
 Cached root task file for the current process.
 */
let rootTaskFile: RootTaskFile | undefined;

/**
 Returns the monorepo root this loader resolves task files against.

 @returns Absolute monorepo root path.
 */
export function repoRoot(): string {
  return REPO_ROOT;
}

/**
 Loads and returns the root `task.ts`.

 @returns Root templates and tasks.
 @throws When the root task file is missing or malformed.
 */
export async function loadRootTasks(): Promise<RootTaskFile> {
  if (rootTaskFile !== undefined) { return rootTaskFile; }
  const path = join(REPO_ROOT, 'task.ts');
  if (!existsSync(path)) { throw new Error(`task-cli: missing root task.ts`); }
  const module_ = await import(pathToFileURL(path).href) as { default: RootTaskFile; };
  rootTaskFile = module_.default;
  return rootTaskFile;
}

/**
 Loads one package `task.ts` file.

 @param path - Absolute path of the package task file.
 @returns Task definitions from the file.
 @throws When the file is missing or malformed.
 */
export async function loadPackageTasks(path: string): Promise<PackageTaskFile> {
  if (!existsSync(path)) { throw new Error(`task-cli: missing task file: ${path}`); }
  const module_ = await import(pathToFileURL(path).href) as { default: PackageTaskFile; };
  return module_.default;
}

/**
 Resolves a task definition through `extends` onto a root template.

 @param definition - Task definition possibly carrying `extends`.
 @returns The definition with template fields filled in.
 @throws When the named template does not exist.
 */
export async function resolveDefinition(definition: TaskDefinition,): Promise<TaskDefinition> {
  if (definition.extends === undefined) { return definition; }
  const root = await loadRootTasks();
  const template = root.templates[definition.extends];
  if (template === undefined)
    throw new Error(`task-cli: unknown task template: ${definition.extends}`);
  return {
    ...template,
    ...definition,
    depends: [...template.depends ?? [], ...definition.depends ?? [],],
  };
}

/**
 Runs this repository's task CLI for task bodies that call other tasks.

 @param name - Task name like `//package/config/oxlint:build:js:node`.
 @param args - Option and passthrough arguments.
 @throws When the child task fails.
 */
export function runTaskCli(name: string, args: readonly string[] = []): void {
  execFileSync(
    process.execPath,
    [join(REPO_ROOT, 'package', 'monorepo-task', 'cli', 'src', 'run.ts'), name, ...args,],
    { stdio: 'inherit', env: { ...process.env, TASK_CLI_CHILD: '1', }, },
  );
}
