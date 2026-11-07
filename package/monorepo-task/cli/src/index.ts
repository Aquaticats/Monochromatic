/**
 Library entry for `@monochromatic-dev/monorepo-task-cli`: the task shapes
 and the loader helpers that resolve root and package `task.ts` files.
 */

export type {
  PackageTaskFile,
  RootTaskFile,
  TaskDefinition,
  TaskStep,
  TaskFunction,
  TaskFunctionContext,
} from './types.ts';

export {
  loadPackageTasks,
  loadRootTasks,
  resolveDefinition,
} from './load.ts';

export {
  createTaskInvocation,
  TaskDependencyCycleError,
  TaskInvocationClosedError,
} from './task-invocation.ts';

export type {
  ResolvedTaskRequest,
  RunTask,
  TaskEvent,
  TaskExecutor,
  TaskRequest,
} from './task-invocation.ts';
