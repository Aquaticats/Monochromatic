/**
 Library entry for `@monochromatic-dev/monorepo-task-cli`: the task shapes
 and the loader helpers that resolve root and package `task.ts` files.

 `src/run.ts` owns the command line and runs it on load, so the package root
 exports nothing from there and importing the package never dispatches a task.

 @module
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
  runTaskCli,
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
