/**
 Shared task shapes for `@monochromatic-dev/dev-script-task-cli`.

 Kept in one module so root and package `task.ts` files and the CLI entry agree
 on the same structural types without importing runtime code.
 */

/**
 One executable step: a plain shell string, or a step plus its shell.
 */
export type TaskStep = string | { readonly shell: string; readonly run: string; };

/**
 One task definition in a root or package `task.ts` file.
 */
export type TaskDefinition = {
  readonly description?: string;
  readonly extends?: string;
  readonly depends?: readonly string[];
  readonly run?: TaskStep | readonly TaskStep[];
  readonly usage?: string;
  readonly env?: Readonly<Record<string, string>>;
};

/**
 Root `task.ts` default export: templates plus root task definitions.
 */
export type RootTaskFile = {
  readonly templates: Readonly<Record<string, TaskDefinition>>;
  readonly tasks: Readonly<Record<string, TaskDefinition>>;
};

/**
 Package `task.ts` default export: task definitions only.
 */
export type PackageTaskFile = Readonly<Record<string, TaskDefinition>>;
