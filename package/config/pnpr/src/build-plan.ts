import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  fieldOf,
  isJsonRecord,
} from './json-shape.ts';

//region How a package builds in a mise-free CI

/**
 Logger root for build planning.
 */
const moduleLogger = tagged({ tag: 'pnpr-build-plan', },);

/**
 How one package builds before `pnpm pack`.
 */
export type BuildPlan =
  | {
    /**
     The package declares `build` in its `task.ts`, so `task-run //<dir>:build` builds it.
     */
    readonly kind: 'task';
  }
    | {
    /**
     The package declares no build, so its sources pack as-is.
     */
      readonly kind: 'none';
    };

/**
 Error for a package whose build exists only in `mise.toml`, which the mise-free CI cannot run.

 Packing such a package without its build would publish a tarball whose `exports` point at a missing `dist`.
 */
export class MissingCiBuildTaskError extends Error {
  /**
   Creates the error naming the package directory and every way to resolve it.

   @param directory - Repository-relative package directory.

   @example
   ```ts
   throw new MissingCiBuildTaskError({ directory: 'package/zip-writer/core' });
   ```
   */
  constructor({ directory, }: { readonly directory: string; },) {
    super(
      `${directory}/mise.toml declares a build task, but pnpr-publish runs in CI without mise `
        + `(doc/decision/mise-free-ci.md) and ${directory}/task.ts does not declare build, `
        + 'so the build cannot run and packing the sources would publish a tarball without its dist. '
        + `Resolve it one way: add ${directory}/task.ts with the build tasks (package/logger/core/task.ts is a model), `
        + `remove the build task from ${directory}/mise.toml if the package needs none, `
        + 'or add the package to PNPR_EXCLUDED_PACKAGES in package/config/pnpr/src/generated-config-settings.ts.',
    );
    this.name = 'MissingCiBuildTaskError';
  }
}

/**
 Whether a path is absent, rethrowing every other filesystem failure.

 @param path - File to look for.

 @returns `true` when the file exists.

 @throws Error when the lookup fails for a reason other than the file being absent.

 @example
 ```ts
 await fileExists({ path: '/repo/package/logger/core/task.ts' });
 ```
 */
async function fileExists({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch (error) {
    if (isJsonRecord(error,) && (error.code === 'ENOENT'))
      return false;
    throw error;
  }
}

/**
 Whether the package's `task.ts` declares a `build` task.

 @param packageDirectory - Absolute package directory.

 @returns `true` when `task.ts` exists and its default export has a `build` key.

 @example
 ```ts
 await taskFileDeclaresBuild({ packageDirectory: '/repo/package/logger/core' });
 ```
 */
async function taskFileDeclaresBuild({ packageDirectory, }: { readonly packageDirectory: string; },): Promise<boolean> {
  /**
   The package's CI task file.
   */
  const taskFile = join(
    packageDirectory,
    'task.ts',
  );
  if (!(await fileExists({ path: taskFile, },)))
    return false;
  /**
   Task file module namespace.
   */
  const loaded: unknown = await import(pathToFileURL(taskFile,)
    .href);
  /**
   Task definitions by name.
   */
  const tasks = fieldOf({
    value: loaded,
    key: 'default',
  },);
  return isJsonRecord(tasks,) && Object.hasOwn(
    tasks,
    'build',
  );
}

/**
 Whether the package's `mise.toml` declares a `build` task.

 @param packageDirectory - Absolute package directory.

 @returns `true` when `mise.toml` has a `[tasks.build]` table.

 @example
 ```ts
 await miseFileDeclaresBuild({ packageDirectory: '/repo/package/logger/core' });
 ```
 */
async function miseFileDeclaresBuild({ packageDirectory, }: { readonly packageDirectory: string; },): Promise<boolean> {
  /**
   The package's local task file.
   */
  const miseFile = join(
    packageDirectory,
    'mise.toml',
  );
  if (!(await fileExists({ path: miseFile, },)))
    return false;
  /**
   Local task file text.
   */
  const miseToml = await readFile(
    miseFile,
    'utf8',
  );
  return miseToml.split('\n',)
    .some(function isBuildTask(line,) {
      return (line.trim() === '[tasks.build]') || (line.trim() === '[tasks."build"]');
    },);
}

/**
 Decides how a package builds before it is packed.

 The CI task file `task.ts` is the only build source CI can run; `mise.toml` is read only to refuse a silent skip.

 @param packageDirectory - Absolute package directory.

 @param directory - Repository-relative package directory, for diagnostics.

 @returns `task` when `task.ts` declares `build`, `none` when no file declares one.

 @throws {@link MissingCiBuildTaskError} when only `mise.toml` declares `build`.

 @example
 ```ts
 await planPackageBuild({ packageDirectory: '/repo/package/logger/core', directory: 'package/logger/core' });
 ```
 */
export async function planPackageBuild(
  {
    packageDirectory,
    directory,
  }: {
    readonly packageDirectory: string;
    readonly directory: string;
  },
): Promise<BuildPlan> {
  if (await taskFileDeclaresBuild({ packageDirectory, },)) {
    moduleLogger.debug(`${directory}: task.ts declares build`,);
    return { kind: 'task', };
  }
  if (await miseFileDeclaresBuild({ packageDirectory, },))
    throw new MissingCiBuildTaskError({ directory, },);
  moduleLogger.debug(`${directory}: no build task in task.ts or mise.toml`,);
  return { kind: 'none', };
}

//endregion How a package builds in a mise-free CI
