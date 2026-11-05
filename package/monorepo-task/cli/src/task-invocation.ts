//region One owner for each task execution within a CLI invocation

/**
 Task identity and explicit inputs, excluding the invocation's fixed base environment.
 */
export type TaskRequest = {
  readonly name: string;
  readonly argv?: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
};

/**
 Normalized request delivered to the task executor.
 */
export type ResolvedTaskRequest = {
  readonly name: string;
  readonly argv: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Runs a task and waits for its dependencies and body to settle.
 */
export type RunTask = (request: TaskRequest) => Promise<void>;

/**
 Executor seam shared by the repository dispatcher and isolated graph tests.
 */
export type TaskExecutor = (context: {
  readonly request: ResolvedTaskRequest;
  readonly runTask: RunTask;
}) => Promise<void>;

/**
 Terminal progress output belongs to the CLI adapter, not the execution cache.
 */
export type TaskEvent = {
  readonly name: string;
  readonly state: 'started' | 'reused' | 'completed' | 'failed' | 'rejected';
};

/**
 Dependency path that would otherwise await its own unfinished execution.
 */
export class TaskDependencyCycleError extends Error {
  /**
   Names the cycle without exposing task arguments or environment values.

   @param names - Dependency path returning to a task already on that path.
   */
  constructor(names: readonly string[],) {
    super(`Task dependency cycle: ${names.join(' -> ',)}`,);
    this.name = 'TaskDependencyCycleError';
  }
}

/**
 A dependency callback outlived the task that owned it.
 */
export class TaskInvocationClosedError extends Error {
  constructor(name: string,) {
    super(`Task ${name} requested dependent work after completing`,);
    this.name = 'TaskInvocationClosedError';
  }
}

/**
 Shared completion and currently awaited dependencies for one task.
 */
type TaskEntry = {
  readonly name: string;
  readonly completion: Promise<void>;
  readonly waits: Set<TaskEntry>;
};

/**
 Encodes all explicit inputs with environment keys in a stable order.

 @param request - Inputs whose execution may be shared.
 @returns Collision-free JSON encoding of the input tuple.
 */
function requestKey(request: ResolvedTaskRequest,): string {
  const environment = Object.entries(request.environment,).toSorted(([left,], [right,]) =>
    left < right ? -1 : left > right ? 1 : 0);
  return JSON.stringify([request.name, request.argv, environment,],);
}

/**
 Checks the pending wait graph, including cycles between concurrently started branches.

 @param parent - Task adding a dependency.
 @param child - Requested dependency.
 @param pending - Executions whose bodies have not settled.
 @throws TaskDependencyCycleError when the new dependency closes a pending cycle.
 */
function assertNoPendingCycle(
  { parent, child, pending, }: {
    readonly parent: TaskEntry;
    readonly child: TaskEntry;
    readonly pending: ReadonlySet<TaskEntry>;
  },
): void {
  const work = [child,];
  const visited = new Set([child,],);
  const previous = new Map<TaskEntry, TaskEntry>();
  for (let current = work.pop(); current !== undefined; current = work.pop()) {
    if (!pending.has(current,))
      continue;
    if (current === parent) {
      const path = [current.name,];
      for (let predecessor = previous.get(current,); predecessor !== undefined; predecessor = previous.get(predecessor,))
        path.push(predecessor.name,);
      throw new TaskDependencyCycleError([parent.name, ...path.toReversed(),],);
    }
    for (const dependency of current.waits) {
      if (visited.has(dependency,))
        continue;
      visited.add(dependency,);
      previous.set(dependency, current,);
      work.push(dependency,);
    }
  }
}

/**
 Creates an invocation-local execution cache. Concurrent callers share the same
 completion or failure; a new invocation always owns a fresh cache.

 @param execute - Task implementation that uses runTask for dependent work.
 @returns Entry point for the invocation's task graph.
 @throws TaskDependencyCycleError when dependent tasks would await each other.
 @throws TaskInvocationClosedError when a completed task requests more dependent work.
 */
export function createTaskInvocation(
  { execute, report, }: { readonly execute: TaskExecutor; readonly report?: (event: TaskEvent) => void; },
): RunTask {
  const executions = new Map<string, TaskEntry>();
  const pending = new Set<TaskEntry>();

  /**
   Shares completion while tracking only dependencies that callers are still awaiting.
   */
  async function visit(
    { request: input, parent, }: { readonly request: TaskRequest; readonly parent?: TaskEntry; },
  ): Promise<void> {
    if (parent !== undefined && !pending.has(parent,)) {
      report?.({ name: input.name, state: 'rejected', },);
      throw new TaskInvocationClosedError(parent.name,);
    }
    const request: ResolvedTaskRequest = {
      name: input.name,
      argv: [...input.argv ?? [],],
      environment: { ...input.environment, },
    };
    const key = requestKey(request,);
    let entry = executions.get(key,);
    if (entry === undefined) {
      /**
       Yields once so the cache owns the execution before dependencies can request it.
       */
      async function executeOnce(): Promise<void> {
        await Promise.resolve();
        try {
          report?.({ name: request.name, state: 'started', },);
          await execute({
            request,
            runTask: async function runDependency(child,) {
              await visit({ request: child, parent: created, },);
            },
          },);
          report?.({ name: request.name, state: 'completed', },);
        }
        catch (error) {
          report?.({ name: request.name, state: 'failed', },);
          throw error;
        }
        finally {
          pending.delete(created,);
        }
      }
      const created: TaskEntry = { name: request.name, waits: new Set(), completion: executeOnce(), };
      executions.set(key, created,);
      pending.add(created,);
      entry = created;
    }
    else {
      report?.({ name: request.name, state: 'reused', },);
    }
    parent?.waits.add(entry,);
    try {
      if (parent !== undefined)
        assertNoPendingCycle({ parent, child: entry, pending, },);
      await entry.completion;
    }
    finally {
      parent?.waits.delete(entry,);
    }
  }

  return async function runTask(request,) {
    await visit({ request, },);
  };
}

//endregion One owner for each task execution within a CLI invocation
