import type {
  PackageTaskFile,
  RootTaskFile,
  TaskDefinition,
  TaskStep,
} from './types.ts';

//region Validate dynamic task module exports before dispatch

/**
 A loaded module does not provide the task declaration shape required by the runner.
 */
export class TaskFileError extends Error {
  /**
   @param path - Module whose default export cannot be dispatched.
   */
  constructor(path: string,) {
    super(`task-cli: invalid task definitions in ${path}`,);
    this.name = 'TaskFileError';
  }
}

/**
 Narrows objects before inspecting their properties.

 @param value - Unknown module export or declaration.
 
 @returns Whether property reads can be performed safely.
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (value !== null) && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

/**
 Checks a task's command or direct function body.

 @param value - Unknown body step from a dynamic module.
 
 @returns Whether execution can choose a supported body form.
 */
function isStep(value: unknown,): value is TaskStep {
  return ((typeof value) === 'string') || ((typeof value) === 'function')
    || (isRecord(value,) && ((typeof value.shell) === 'string')
      && ((typeof value.run) === 'string'));
}

/**
 Checks a list while keeping untyped array elements unknown.

 @param value - Possible ordered body sequence.
 
 @returns Whether every element can be executed as a task step.
 */
function isStepList(value: unknown,): value is readonly TaskStep[] {
  if (!Array.isArray(value,))
    return false;
  /**
   Array.isArray proves the container shape without proving its element types.
   */
  const steps: readonly unknown[] = value;
  return steps.every(isStep,);
}

/**
 Checks string lists used for dependency identities.

 @param value - Possible dependency list.
 
 @returns Whether each identity can be resolved as a task name.
 */
function isStringList(value: unknown,): value is readonly string[] {
  if (!Array.isArray(value,))
    return false;
  /**
   Dependencies must be strings before prefix and separator operations are used.
   */
  const entries: readonly unknown[] = value;
  return entries.every(function stringEntry(entry) {
    return (typeof entry) === 'string';
  },);
}

/**
 Validates the supported optional declaration fields without rejecting task metadata.

 @param value - Unknown task definition.
 
 @returns Whether dispatch can consume the definition.
 */
function isTaskDefinition(value: unknown,): value is TaskDefinition {
  if (!isRecord(value,))
    return false;
  for (const key of [
    'description',
    'extends',
    'dir',
    'usage',
  ] as const) {
    if ((value[key] !== undefined) && ((typeof value[key]) !== 'string'))
      return false;
  }
  if ((value.depends !== undefined) && (!isStringList(value.depends,)))
    return false;
  if (value.env !== undefined) {
    if (!isRecord(value.env,))
      return false;
    if (!Object.values(value.env,)
      .every(function stringValue(entry) { return (typeof entry) === 'string'; },))
      return false;
  }
  return (value.run === undefined) || isStep(value.run,)
    || isStepList(value.run,);
}

/**
 Checks every declaration before treating the export as a task dictionary.

 @param value - Unknown task collection.
 
 @returns Whether all named values are supported task declarations.
 */
function isTaskFile(value: unknown,): value is PackageTaskFile {
  return isRecord(value,)
    && Object.values(value,)
      .every(isTaskDefinition,);
}

/**
 Checks root task modules at the dynamic-import boundary.

 @param namespace - Dynamically imported namespace.
 
 @param path - Source path included in a failure diagnostic.
 
 @returns Validated root templates and task declarations.
 
 @throws TaskFileError when templates or tasks are malformed.
 
 @example
 ```ts
 const root = rootTasksFromModule({ namespace: { default: { templates: {}, tasks: {} } }, path: 'task.ts' });
 ```
 */
export function rootTasksFromModule({
  namespace,
  path,
}: {
  readonly namespace: unknown;
  readonly path: string;
},): RootTaskFile {
  if ((!isRecord(namespace,)) || (!isRecord(namespace.default,))
    || (!isTaskFile(namespace.default
      .templates,))
    || (!isTaskFile(namespace.default
      .tasks,)))
    throw new TaskFileError(path,);
  return {
    templates: namespace.default
      .templates,
    tasks: namespace.default
      .tasks,
  };
}

/**
 Checks a package module at the dynamic-import boundary.

 @param namespace - Dynamically imported namespace.
 
 @param path - Source path included in a failure diagnostic.
 
 @returns Validated package task dictionary.
 
 @throws TaskFileError when a task declaration is malformed.
 
 @example
 ```ts
 const tasks = packageTasksFromModule({ namespace: { default: {} }, path: 'task.ts' });
 ```
 */
export function packageTasksFromModule({
  namespace,
  path,
}: {
  readonly namespace: unknown;
  readonly path: string;
},): PackageTaskFile {
  if ((!isRecord(namespace,)) || (!isTaskFile(namespace.default,)))
    throw new TaskFileError(path,);
  return namespace.default;
}

//endregion Validate dynamic task module exports before dispatch
