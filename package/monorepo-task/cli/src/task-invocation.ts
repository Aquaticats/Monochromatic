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
  /** Identifies the completed owner without exposing its inputs. */
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
  /** Canonical key order makes object insertion order irrelevant to execution identity. */
  const environment = Object.entries(request.environment,)
    .toSorted(function compareKeys(
      [left,]: readonly [string, string],
      [right,]: readonly [string, string],
    ) {
      return left < right ? -1 : left > right ? 1 : 0;
    });
  return JSON.stringify([
    request.name,
    request.argv,
    environment,
  ],);
}

/**
 Checks the pending wait graph, including cycles between concurrently started branches.

 @param parent - Task adding a dependency.
 
 @param child - Requested dependency.
 
 @param pending - Executions whose bodies have not settled.
 
 @throws TaskDependencyCycleError when the new dependency closes a pending cycle.
 */
function assertNoPendingCycle(
  {
    parent,
    child,
    pending,
  }: {
    readonly parent: TaskEntry;
    readonly child: TaskEntry;
    readonly pending: ReadonlySet<TaskEntry>;
  },
): void {
  /** Explicit work stack avoids recursion through long dependency chains. */
  const work = [child,];
  /** Each pending execution is examined at most once per cycle check. */
  const visited = new Set([child,],);
  /** Predecessors reconstruct only the cycle that is actually reported. */
  const previous = new Map<TaskEntry, TaskEntry>();
  for (let current = work.pop(); current !== undefined; current = work.pop()) {
    if (!pending.has(current,))
      continue;
    if (current === parent) {
      /** Human-readable identities deliberately omit argument and environment values. */
      const path = [current.name,];
      for (let predecessor = previous.get(current,); predecessor !== undefined; predecessor = previous.get(predecessor,))
        path.push(predecessor.name,);
      throw new TaskDependencyCycleError([
        parent.name,
        ...path.toReversed(),
      ],);
    }
    for (const dependency of current.waits) {
      if (visited.has(dependency,))
        continue;
      visited.add(dependency,);
      previous.set(
        dependency,
        current,
      );
      work.push(dependency,);
    }
  }
}

/**
 Creates an invocation-local execution cache. Concurrent callers share the same
 completion or failure; a new invocation always owns a fresh cache.

 @param execute - Task implementation that uses runTask for dependent work.
 
 @param report - CLI progress observer, receiving identities and states without argument or environment values.
 
 @returns Entry point for the invocation's task graph.
 
 @throws TaskDependencyCycleError when dependent tasks would await each other.
 
 @throws TaskInvocationClosedError when a completed task requests more dependent work.

 @example
 ```ts
 const run = createTaskInvocation({ execute: dispatch });
 await run({ name: 'build' });
 ```
 */
export function createTaskInvocation(
  {
    execute,
    report,
  }: {
    readonly execute: TaskExecutor;
    readonly report?: (event: TaskEvent) => void
  },
): RunTask {
  /** Settled executions remain reusable until this invocation is released. */
  const executions = new Map<string, TaskEntry>();
  /** Only unfinished bodies can participate in a dependency deadlock. */
  const pending = new Set<TaskEntry>();

  /**
   Shares completion while tracking only dependencies that callers are still awaiting.
   */
  async function visit(
    {
      request: input,
      parent,
    }: {
      readonly request: TaskRequest;
      readonly parent?: TaskEntry
    },
  ): Promise<void> {
    if ((parent !== undefined) && (!pending.has(parent,))) {
      report?.({
        name: input.name,
        state: 'rejected',
      },);
      throw new TaskInvocationClosedError(parent.name,);
    }
    /** Snapshot caller-owned inputs before the executor gets a chance to run. */
    const request: ResolvedTaskRequest = {
      name: input.name,
      argv: [...input.argv ?? [],],
      environment: { ...input.environment, },
    };
    /** Full input identity distinguishes calls that must run separately. */
    const key = requestKey(request,);
    /** The creator owns setup; every subsequent caller observes the same completion. */
    const entry = (function entryForRequest(): TaskEntry {
      /** Previously started work includes both fulfilled and rejected completions. */
      const cached = executions.get(key,);
      if (cached !== undefined) {
        report?.({ name: request.name, state: 'reused', });
        return cached;
      }
      /**
       Yields once so the cache owns the execution before dependencies can request it.
       */
      async function executeOnce(): Promise<void> {
        await Promise.resolve();
        /** Closing the body prevents retained dependency callbacks from starting new work. */
        using completion = {
          [Symbol.dispose](): void { pending.delete(created,); },
        };
        try {
          report?.({
            name: request.name,
            state: 'started',
          },);
          await execute({
            request,
            runTask: async function runDependency(child,) {
              await visit({
                request: child,
                parent: created,
              },);
            },
          },);
          report?.({
            name: request.name,
            state: 'completed',
          },);
        }
        catch (error) {
          report?.({
            name: request.name,
            state: 'failed',
          },);
          throw error;
        }
      }
      /** The initial yield lets this record enter both indexes before execution proceeds. */
      const created: TaskEntry = {
        name: request.name,
        waits: new Set(),
        completion: executeOnce(),
      };
      executions.set(
        key,
        created,
      );
      pending.add(created,);
      return created;
    })();
    /** The wait edge exists only until this caller observes completion or a cycle error. */
    using waiting = {
      [Symbol.dispose](): void { parent?.waits.delete(entry,); },
    };
    parent?.waits
      .add(entry,);
    if (parent !== undefined)
      assertNoPendingCycle({ parent, child: entry, pending, },);
    await entry.completion;
  }

  return async function runTask(request,) {
    await visit({ request, },);
  };
}

//endregion One owner for each task execution within a CLI invocation
