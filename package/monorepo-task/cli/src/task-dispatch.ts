import { availableParallelism, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  loadPackageTasks,
  loadRootTasks,
  resolveDefinition,
} from './load.ts';
import { splitPassthrough, } from './task-arguments.ts';
import {
  executeTask,
  TaskChildrenFailedError,
} from './task-execution.ts';
import type {
  ResolvedTaskRequest,
  RunTask,
} from './task-invocation.ts';
import {
  findPackageTaskFiles,
  packageDirectory,
} from './task-locations.ts';

//region Dispatch named tasks through an invocation-owned executor

/**
 Root commands that select the same task suffix in every package.
 */
const ROOT_FANOUT_TASKS: Readonly<Record<string, string>> = {
  build: 'build',
  test: 'test',
  'test:unit': 'test:unit',
  lint: 'lint',
};

/**
 A requested task does not exist or does not accept the supplied invocation.
 */
export class TaskSelectionError extends Error {
  /**
   Names the missing task or unsupported invocation.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'TaskSelectionError';
  }
}

/**
 Runs root package groups with the existing bounded concurrency policy.
 */
async function fanoutPackages(
  {
    root,
    suffix,
    request,
    runTask,
  }: {
    readonly root: string;
    readonly suffix: string;
    readonly request: ResolvedTaskRequest;
    readonly runTask: RunTask;
  },
): Promise<void> {
  /**
   Selected package identities preserve discovery order for bounded dispatch.
   */
  const names: string[] = [];
  for (const file of await findPackageTaskFiles(root,)) {
    /**
     Task definitions determine whether this package participates in the requested group.
     */
    const tasks = await loadPackageTasks(file,);
    if (Object.hasOwn(
      tasks,
      suffix,
    ))
      names.push(`//${dirname(file,)
        .slice(root.length + 1,)}:${suffix}`,);
  }
  if (names.length === 0)
    throw new TaskSelectionError(`task-cli: no package tasks matched ${suffix}`,);
  /**
   The existing root fanout policy bounds workers by available parallelism and selected tasks.
   */
  const limit = suffix.startsWith('watch') ? names.length : Math.max(
    1,
    Math.min(
      availableParallelism(),
      names.length,
    ),
  );
  /**
   All worker failures are retained until every active worker settles.
   */
  const failures: unknown[] = [];
  /**
   Workers claim task identities synchronously from one shared iterator.
   */
  const remaining = names.values();
  /**
   Consumes selected tasks while preserving errors from independent package builds.
   */
  async function worker(): Promise<void> {
    for (const name of remaining) {
      try {
        await runTask({
          name,
          environment: request.environment,
        },);
      }
      catch (error) {
        failures.push(error,);
      }
    }
  }
  await Promise.all(Array.from(
    { length: limit, },
    worker,
  ),);
  if (failures.length > 0)
    throw new TaskChildrenFailedError({
      name: request.name,
      failures,
    },);
}

/**
 Resolves a named task and executes it using the caller's shared dispatcher.

 @example
 ```ts
 await dispatchTask({ root, environment, request, runTask });
 ```
 */
export async function dispatchTask(
  {
    root,
    environment,
    request,
    runTask,
  }: {
    readonly root: string;
    readonly environment: Readonly<NodeJS.ProcessEnv>;
    readonly request: ResolvedTaskRequest;
    readonly runTask: RunTask;
  },
): Promise<void> {
  /**
   Task identity and argument words come from the normalized invocation request.
   */
  const {
    name,
    argv,
  } = request;
  /**
   Option words and literal passthrough are interpreted separately.
   */
  const {
    options: optionArgs,
    rest,
  } = splitPassthrough(argv,);
  if (name.startsWith('//')) {
    /**
     The first colon after the package prefix separates its task suffix.
     */
    const separator = name.indexOf(
      ':',
      2,
    );
    /**
     Package location excludes the task suffix.
     */
    const packagePath = name.slice(
      2,
      separator === (-1) ? undefined : separator,
    );
    /**
     An omitted task suffix stays empty and receives the normal missing-task diagnostic.
     */
    const taskName = separator === (-1) ? '' : name.slice(separator + 1,);
    /**
     Package manifests establish the working directory before loading its task definitions.
     */
    const cwd = await packageDirectory({
      root,
      packagePath,
    },);
    /**
     Task definitions determine whether this package participates in the requested group.
     */
    const tasks = await loadPackageTasks(join(
      cwd,
      'task.ts',
    ),);
    /**
     Own-property checks exclude prototype names from the task namespace.
     */
    const raw = Object.hasOwn(
      tasks,
      taskName,
    ) ? tasks[taskName] : undefined;
    if (raw === undefined)
      throw new TaskSelectionError(`task-cli: no task ${taskName} in ${packagePath}`,);
    /**
     Template defaults are resolved before the task enters its execution body.
     */
    const definition = await resolveDefinition({
      definition: raw,
      root,
    },);
    await executeTask({
      root,
      cwd,
      definition,
      optionArgs,
      rest,
      request,
      environment,
      runTask,
    },);
    return;
  }
  /**
   Only declared root fanout names receive package-wide dispatch.
   */
  const suffix = Object.hasOwn(
    ROOT_FANOUT_TASKS,
    name,
  ) ? ROOT_FANOUT_TASKS[name] : undefined;
  if (suffix !== undefined) {
    if ((optionArgs.length > 0) || (rest.length > 0))
      throw new TaskSelectionError(`task-cli: ${name} takes no arguments`,);
    await fanoutPackages({
      root,
      suffix,
      request,
      runTask,
    },);
    return;
  }
  /**
   Ordinary root tasks resolve from the same repository as package templates.
   */
  const rootTasks = await loadRootTasks(root,);
  /**
   Own-property checks exclude prototype names from the task namespace.
   */
  const raw = Object.hasOwn(
    rootTasks.tasks,
    name,
  ) ? rootTasks.tasks[name] : undefined;
  if (raw === undefined)
    throw new TaskSelectionError(`task-cli: unknown task: ${name}`,);
  /**
   Template defaults are resolved before the task enters its execution body.
   */
  const definition = await resolveDefinition({
    definition: raw,
    root,
  },);
  await executeTask({
    root,
    cwd: root,
    definition,
    optionArgs,
    rest,
    request,
    environment,
    runTask,
  },);
}

//endregion Dispatch named tasks through an invocation-owned executor
