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
import {
  runTaskProcess,
  TaskCommandFailedError,
} from './task-process.ts';
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
  /**
   Reports the parent task while retaining the original failures.
   */
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
  /**
   Windows environment names can retain mixed-case PATH spelling.
   */
  const pathKey = Object.keys(inherited,)
    .find(function pathKey(key) { return key.toLowerCase() === 'path'; })
    ?? 'PATH';
  /**
   Local tool bins precede ancestor and system bins in lookup order.
   */
  const directories: string[] = [];
  for (let directory = cwd; ;) {
    directories.push(join(
      directory,
      'node_modules',
      '.bin',
    ),);
    if (directory === root)
      break;
    /**
     Stop at the filesystem root even if the configured repository boundary is outside this path.
     */
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

 @param value - Declared body inspected before sequencing.
 
 @returns Whether the body contains a sequence of steps.
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
  /**
   String tasks use Bash; explicit shell steps preserve their selected interpreter.
   */
  const [shell, rawBody,] = (typeof step) === 'string' ? [
    'bash',
    step,
  ] : [
    step.shell,
    step.run,
  ];
  /**
   Interpreter options remain separate from the body text.
   */
  const [command, ...shellArgs] = shell.split(' ',);
  if ((command === undefined) || (command === ''))
    throw new TaskCommandFailedError({
      command: shell,
      cwd: context.cwd,
      status: 'empty shell command',
    },);
  /**
   Only native TypeScript inline bodies receive the repository helper prelude.
   */
  const isNodeInline = (command === 'node') && shellArgs.includes('--input-type=module-typescript',);
  // Braces keep body imports separate from prelude bindings; the final newline ends trailing comments.
  /**
   A separate block prevents body imports from colliding with prelude bindings.
   */
  const body = isNodeInline ? `${BODY_PRELUDE}\n{\n${rawBody}\n\n}\n` : rawBody;
  /**
   Shell passthrough uses positional parameters instead of interpolating argument text.
   */
  const script = (!isNodeInline) && (context.argv
    .length
    > 0) ? `${body} "$@"` : body;
  /**
   The child receives body text and literal arguments through distinct argv entries.
   */
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
  await runTaskProcess({
    command,
    args: commandArgs,
    cwd: context.cwd,
    env: context.env,
  });
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
  /**
   Template fanout selects immediate children from the package's declared tasks.
   */
  const tasks = await loadPackageTasks(join(
    root,
    packagePath,
    'task.ts',
  ),);
  /**
   A colon boundary prevents unrelated task prefixes from matching.
   */
  const prefix = `${suffix}:`;
  /**
   Immediate child names contain exactly one additional colon-separated segment.
   */
  const depth = prefix.split(':',)
    .length;
  /**
   Stable child ordering gives deterministic dispatch without serializing completion.
   */
  const children = Object.keys(tasks,)
    .filter(function immediateChild(name) { return name.startsWith(prefix) && (name.split(':',)
      .length
      === depth); })
    .toSorted(function byName(
      left,
      right
    ) { return left.localeCompare(right,); });
  if (children.length === 0)
    throw new TaskCommandFailedError({
      command: `//${packagePath}:${suffix}`,
      cwd: root,
      status: 'no child tasks',
    },);
  /**
   All concurrent children settle before their parent reports aggregate failure.
   */
  const settled = await Promise.allSettled(children.map(async function startChild(name) {
    await runTask({
      name: `//${packagePath}:${name}`,
      environment,
    },);
  }),);
  /**
   Rejections keep their original errors for the parent diagnostic.
   */
  const failures: unknown[] = [];
  for (const result of settled) {
    if (result.status === 'rejected')
      failures.push(result.reason,);
  }
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

 @example
 ```ts
 await executeTask({ root, cwd, definition, optionArgs: [], rest: [], request, environment, runTask });
 ```
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
  /**
   Relative package identity anchors local dependency names and task-body context.
   */
  const packagePath = resolve(
    root,
    cwd,
  )
    .slice(root.length + 1,);
  for (const dependency of definition.depends ?? []) {
    /**
     Explicit repository task names remain absolute; local suffixes inherit their package.
     */
    const name = dependency.startsWith('//')
      ? dependency
      : packagePath.startsWith('package/') ? `//${packagePath}:${dependency}` : dependency;
    await runTask({
      name,
      environment: request.environment,
    },);
  }
  /**
   A separate block prevents body imports from colliding with prelude bindings.
   */
  /**
   Dependency-only tasks finish after their prerequisites settle.
   */
  const body = definition.run;
  if (body === undefined)
    return;
  /**
   A body array is an ordered sequence, while a function or command is a single step.
   */
  const steps = isStepList(body,) ? body : [body,];
  /**
   A declared directory changes only the task body, leaving process-global cwd untouched.
   */
  const bodyCwd = definition.dir === undefined ? cwd : resolve(
    cwd,
    definition.dir,
  );
  /**
   Usage declarations determine whether words become environment options or literal argv.
   */
  const usageOptions = parseUsageSpec(definition.usage,);
  /**
   Parsed option values are scoped to this task's process environment.
   */
  const usage = usageEnvironment({
    options: usageOptions,
    argv: optionArgs,
    rest,
  },);
  /**
   Explicit invocation overrides precede task-specific definitions and usage variables.
   */
  const inherited = {
    ...environment,
    ...request.environment,
  };
  /**
   Every child gets a fresh environment including task identity and package tool bins.
   */
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
  /**
   Direct function bodies receive the same explicit inputs as command bodies.
   */
  const context: TaskFunctionContext = {
    cwd: bodyCwd,
    env,
    argv: usageOptions.length === 0 ? [
      ...optionArgs,
      ...rest,
    ] : [],
    runChildren: async function dispatchChildren(suffix) {
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
