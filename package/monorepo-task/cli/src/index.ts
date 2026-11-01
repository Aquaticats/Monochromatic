/**
 Library entry for `@monochromatic-dev/monorepo-task-cli`: the task shapes
 and the loader helpers that resolve root and package `task.ts` files.
 */

export type {
  PackageTaskFile,
  RootTaskFile,
  TaskDefinition,
  TaskStep,
} from './types.ts';

export {
  loadPackageTasks,
  loadRootTasks,
  resolveDefinition,
} from './load.ts';
