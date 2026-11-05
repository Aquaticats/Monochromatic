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
  const names: string[] = [];
  for (const file of findPackageTaskFiles(root,)) {
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
  const limit = suffix.startsWith('watch') ? names.length : Math.max(
    1,
    Math.min(
      availableParallelism(),
      names.length,
    ),
  );
  const failures: unknown[] = [];
  const remaining = names.values();
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
  const {
    name,
    argv,
  } = request;
  const {
    options: optionArgs,
    rest,
  } = splitPassthrough(argv,);
  if (name.startsWith('//')) {
    const separator = name.indexOf(
      ':',
      2,
    );
    const packagePath = name.slice(
      2,
      separator === (-1) ? undefined : separator,
    );
    const taskName = separator === (-1) ? '' : name.slice(separator + 1,);
    const cwd = packageDirectory({
      root,
      packagePath,
    },);
    const tasks = await loadPackageTasks(join(
      cwd,
      'task.ts',
    ),);
    const raw = Object.hasOwn(
      tasks,
      taskName,
    ) ? tasks[taskName] : undefined;
    if (raw === undefined)
      throw new TaskSelectionError(`task-cli: no task ${taskName} in ${packagePath}`,);
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
  const rootTasks = await loadRootTasks(root,);
  const raw = Object.hasOwn(
    rootTasks.tasks,
    name,
  ) ? rootTasks.tasks[name] : undefined;
  if (raw === undefined)
    throw new TaskSelectionError(`task-cli: unknown task: ${name}`,);
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
