import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  delimiter,
  dirname,
  join,
  resolve,
} from 'node:path';

import { loadPackageTasks, } from './load.ts';
import { usageEnvironment, } from './task-arguments.ts';
import { parseUsageSpec, } from './usage-declarations.ts';
import type {
  ResolvedTaskRequest,
  RunTask,
} from './task-invocation.ts';
import { BODY_PRELUDE, } from './task-script-prelude.ts';
import type {
  TaskDefinition,
  TaskFunctionContext,
  TaskStep,
} from './types.ts';

//region Task bodies and their process environment

/**
 Failed child tasks retain their individual errors for the caller's diagnostic.
 */
export class TaskChildrenFailedError extends AggregateError {
  constructor({
    name,
    failures,
  }: {
    readonly name: string;
    readonly failures: readonly unknown[]
  },) {
    super(
      failures,
      `Child tasks failed for ${name}`,
    );
    this.name = 'TaskChildrenFailedError';
  }
}

/**
 A task command did not complete successfully.
 */
export class TaskCommandFailedError extends Error {
  constructor({
    command,
    cwd,
    status,
  }: {
    readonly command: string;
    readonly cwd: string;
    readonly status: string
  },) {
    super(`Task command ${command} in ${cwd} failed: ${status}`,);
    this.name = 'TaskCommandFailedError';
  }
}

/**
 Resolves package-local tool bins before inherited system tools.

 @param root - Boundary of the repository-owned bin search.
 
 @param cwd - Working directory of this task body.
 
 @param inherited - Fixed invocation environment plus explicit inherited overrides.
 
 @returns PATH fragment for this task's command processes.
 */
function taskPathEnvironment(
  {
    root,
    cwd,
    inherited,
  }: {
    readonly root: string;
    readonly cwd: string;
    readonly inherited: Readonly<NodeJS.ProcessEnv>
  },
): NodeJS.ProcessEnv {
  const pathKey = Object.keys(inherited,)
    .find(key => key.toLowerCase() === 'path')
    ?? 'PATH';
  const directories: string[] = [];
  for (let directory = cwd; ;) {
    directories.push(join(
      directory,
      'node_modules',
      '.bin',
    ),);
    if (directory === root)
      break;
    const parent = dirname(directory,);
    if (parent === directory)
      break;
    directory = parent;
  }
  directories.push(inherited[pathKey] ?? '',);
  return { [pathKey]: directories.join(delimiter,), };
}

/**
 Distinguishes sequential step lists from individual function or command steps.
 */
function isStepList(value: TaskDefinition['run'],): value is readonly TaskStep[] {
  return Array.isArray(value,);
}

/**
 Executes a body step and waits for its process or direct async function.

 @param step - Body selected by the task definition.
 
 @param context - Invocation-owned working directory, environment, arguments and child dispatcher.
 
 @throws TaskCommandFailedError when a subprocess exits unsuccessfully.
 */
async function runStep({
  step,
  context,
}: {
  readonly step: TaskStep;
  readonly context: TaskFunctionContext
},): Promise<void> {
  if ((typeof step) === 'function') {
    await step(context,);
    return;
  }
  const [shell, rawBody,] = (typeof step) === 'string' ? [
    'bash',
    step,
  ] : [
    step.shell,
    step.run,
  ];
  const [command, ...shellArgs] = shell.split(' ',);
  if ((command === undefined) || (command === ''))
    throw new TaskCommandFailedError({
      command: shell,
      cwd: context.cwd,
      status: 'empty shell command',
    },);
  const isNodeInline = (command === 'node') && shellArgs.includes('--input-type=module-typescript',);
  // Braces keep body imports separate from prelude bindings; the final newline ends trailing comments.
  const body = isNodeInline ? `${BODY_PRELUDE}\n{\n${rawBody}\n\n}\n` : rawBody;
  const script = (!isNodeInline) && (context.argv
    .length
    > 0) ? `${body} "$@"` : body;
  const commandArgs = isNodeInline
    ? [
      ...shellArgs,
      script,
    ]
    : [
      ...shellArgs,
      '-c',
      script,
      'task-run',
      ...context.argv,
    ];
  const child = spawn(
    command,
    commandArgs,
    {
      cwd: context.cwd,
      env: context.env,
      stdio: 'inherit',
    },
  );
  const [code, signal,] = await once(
    child,
    'exit',
  );
  if (code !== 0)
    throw new TaskCommandFailedError({
      command,
      cwd: context.cwd,
      status: signal === null ? `exit ${code}` : `signal ${signal}`,
    },);
}

/**
 Runs immediate template children through the same task invocation.

 @param root - Repository containing the package task file.
 
 @param packagePath - Package whose child tasks the template selects.
 
 @param suffix - Parent task suffix, such as build:js.
 
 @param environment - Explicit values inherited by the child task calls.
 
 @param runTask - Shared invocation dispatcher.
 
 @throws TaskChildrenFailedError when a child fails.
 */
async function runChildren(
  {
    root,
    packagePath,
    suffix,
    environment,
    runTask,
  }: {
    readonly root: string;
    readonly packagePath: string;
    readonly suffix: string;
    readonly environment: Readonly<Record<string, string>>;
    readonly runTask: RunTask;
  },
): Promise<void> {
  const tasks = await loadPackageTasks(join(
    root,
    packagePath,
    'task.ts',
  ),);
  const prefix = `${suffix}:`;
  const depth = prefix.split(':',)
    .length;
  const children = Object.keys(tasks,)
    .filter(name => name.startsWith(prefix) && (name.split(':',)
      .length
      === depth))
    .toSorted();
  if (children.length === 0)
    throw new TaskCommandFailedError({
      command: `//${packagePath}:${suffix}`,
      cwd: root,
      status: 'no child tasks',
    },);
  const settled = await Promise.allSettled(children.map(async name => {
    await runTask({
      name: `//${packagePath}:${name}`,
      environment,
    },);
  }),);
  const failures = settled.flatMap(result => result.status === 'rejected' ? [result.reason,] : []);
  if (failures.length > 0)
    throw new TaskChildrenFailedError({
      name: `//${packagePath}:${suffix}`,
      failures,
    },);
}

/**
 Runs dependencies first, then body steps, with a stable invocation environment.

 @param root - Repository containing task definitions.
 
 @param cwd - Package or repository owning the task.
 
 @param definition - Resolved task body and dependencies.
 
 @param optionArgs - Arguments interpreted by the usage declaration.
 
 @param rest - Passthrough arguments after the second separator.
 
 @param request - Explicit call inputs used by the invocation cache.
 
 @param environment - Base environment captured when the CLI invocation began.
 
 @param runTask - Dispatcher shared with dependencies and template children.
 */
export async function executeTask(
  {
    root,
    cwd,
    definition,
    optionArgs,
    rest,
    request,
    environment,
    runTask,
  }: {
    readonly root: string;
    readonly cwd: string;
    readonly definition: TaskDefinition;
    readonly optionArgs: readonly string[];
    readonly rest: readonly string[];
    readonly request: ResolvedTaskRequest;
    readonly environment: Readonly<NodeJS.ProcessEnv>;
    readonly runTask: RunTask;
  },
): Promise<void> {
  const packagePath = resolve(
    root,
    cwd,
  )
    .slice(root.length + 1,);
  for (const dependency of definition.depends ?? []) {
    const name = dependency.startsWith('//')
      ? dependency
      : packagePath.startsWith('package/') ? `//${packagePath}:${dependency}` : dependency;
    await runTask({
      name,
      environment: request.environment,
    },);
  }
  const body = definition.run;
  if (body === undefined)
    return;
  const steps = isStepList(body,) ? body : [body,];
  const bodyCwd = definition.dir === undefined ? cwd : resolve(
    cwd,
    definition.dir,
  );
  const usageOptions = parseUsageSpec(definition.usage,);
  const usage = usageEnvironment({
    options: usageOptions,
    argv: optionArgs,
    rest,
  },);
  const inherited = {
    ...environment,
    ...request.environment,
  };
  const env: NodeJS.ProcessEnv = {
    ...inherited,
    ...taskPathEnvironment({
      root,
      cwd: bodyCwd,
      inherited,
    },),
    ...definition.env,
    TASK_REPO_ROOT: root,
    TASK_PACKAGE_PATH: packagePath,
    ...usage,
  };
  const context: TaskFunctionContext = {
    cwd: bodyCwd,
    env,
    argv: usageOptions.length === 0 ? [
      ...optionArgs,
      ...rest,
    ] : [],
    runChildren: async suffix => {
      await runChildren({
        root,
        packagePath,
        suffix,
        environment: {
          ...request.environment,
          ...definition.env,
          ...usage,
          TASK_CLI_CHILD: '1',
        },
        runTask,
      },);
    },
  };
  for (const step of steps)
    await runStep({
      step,
      context,
    },);
}

//endregion Task bodies and their process environment
