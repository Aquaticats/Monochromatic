import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  MissingCiBuildTaskError,
  planPackageBuild,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Repository-relative directory used in diagnostics.
 */
const DIRECTORY = 'package/example/core';

/**
 `task.ts` that declares `build`, shaped like the repository's package task files.
 */
const TASK_WITH_BUILD = 'export default { build: { extends: \'build\', }, lint: { extends: \'lint\', }, };\n';

/**
 `task.ts` that declares tasks other than `build`.
 */
const TASK_WITHOUT_BUILD = 'export default { lint: { extends: \'lint\', }, };\n';

/**
 `mise.toml` that declares `build`.
 */
const MISE_WITH_BUILD = '[tasks.build]\nextends = "build"\n';

/**
 `mise.toml` that declares tasks other than `build`.
 */
const MISE_WITHOUT_BUILD = '[tasks.lint]\nextends = "lint"\n';

/**
 `mise.toml` that declares `build` as a dotted key rather than a table header.
 */
const MISE_WITH_BUILD_DOTTED = 'tasks.build = { extends = "build" }\n';

/**
 `mise.toml` that declares `build` on a header line which also carries a comment.
 */
const MISE_WITH_BUILD_COMMENTED = '[tasks.build] # builds dist before packing\nrun = "node build.ts"\n';

/**
 `mise.toml` whose text names the build header only inside a multiline string.
 */
const MISE_NAMING_BUILD_IN_A_STRING = 'description = """\n[tasks.build]\n"""\n';

/**
 Disposable package directory; removed on scope exit.
 */
type DisposablePackage = {
  readonly packageDirectory: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable package directory holding the given files.

 @param files - File contents by file name; an absent name means the file does not exist.

 @returns Handle to use with `await using`.

 @example
 ```ts
 await using fixture = await makePackage({ files: { 'task.ts': TASK_WITH_BUILD } });
 ```
 */
async function makePackage(
  { files, }: { readonly files: Readonly<Record<string, string>>; },
): Promise<DisposablePackage> {
  /**
   Disposable package directory.
   */
  const packageDirectory = await mkdtemp(join(scratchRoot(), 'pnpr-build-plan-',),);
  await Promise.all(Object.entries(files,)
    .map(async function writeOne([name, text,],) {
      await writeFile(join(packageDirectory, name,), text, 'utf8',);
    },),);
  return {
    packageDirectory,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(packageDirectory, {
        recursive: true,
        force: true,
      },);
    },
  };
}

/**
 Runs a body in a disposable package directory holding the given files.

 @param files - File contents by file name; an absent name means the file does not exist.

 @param body - Receives the absolute package directory.

 @example
 ```ts
 await withPackage({ files: { 'task.ts': TASK_WITH_BUILD }, body: async (directory) => { void directory; } });
 ```
 */
async function withPackage(
  {
    files,
    body,
  }: {
    readonly files: Readonly<Record<string, string>>;
    readonly body: (packageDirectory: string,) => Promise<void>;
  },
): Promise<void> {
  await using fixture = await makePackage({ files, },);
  await body(fixture.packageDirectory,);
}

/**
 Captures the error an async call rejects with.

 @param call - Operation expected to reject.

 @returns The rejection value, or `undefined` when the call resolved.

 @example
 ```ts
 const caught = await caughtBy(async () => { throw new Error('x'); });
 ```
 */
async function caughtBy(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

//endregion Fixtures

await describe({
  name: planPackageBuild.name,
  children: [
    it({
      name: 'plans a task build when task.ts declares build',
      fn: async () => {
        await withPackage({
          files: { 'task.ts': TASK_WITH_BUILD, },
          body: async (packageDirectory,) => {
            expect(await planPackageBuild({
              packageDirectory,
              directory: DIRECTORY,
            },),).toEqual({ kind: 'task', },);
          },
        },);
      },
    },),
    it({
      name: 'plans a task build when task.ts and mise.toml both declare build',
      fn: async () => {
        await withPackage({
          files: {
            'task.ts': TASK_WITH_BUILD,
            'mise.toml': MISE_WITH_BUILD,
          },
          body: async (packageDirectory,) => {
            expect(await planPackageBuild({
              packageDirectory,
              directory: DIRECTORY,
            },),).toEqual({ kind: 'task', },);
          },
        },);
      },
    },),
    it({
      name: 'refuses a package whose build exists only in mise.toml and names every way out',
      fn: async () => {
        await withPackage({
          files: { 'mise.toml': MISE_WITH_BUILD, },
          body: async (packageDirectory,) => {
            const caught = await caughtBy(async () => {
              await planPackageBuild({
                packageDirectory,
                directory: DIRECTORY,
              },);
            },);
            expect(caught instanceof MissingCiBuildTaskError,).toBe(true,);
            if (!(caught instanceof MissingCiBuildTaskError))
              throw new Error('rejection was not a MissingCiBuildTaskError',);
            expect(caught.message,).toContain(`${DIRECTORY}/mise.toml declares a build task`,);
            expect(caught.message,).toContain('doc/decision/mise-free-ci.md',);
            expect(caught.message,).toContain(`add ${DIRECTORY}/task.ts`,);
            expect(caught.message,).toContain('PNPR_EXCLUDED_PACKAGES',);
          },
        },);
      },
    },),
    it({
      name: 'refuses a package whose task.ts lacks build while mise.toml declares it',
      fn: async () => {
        await withPackage({
          files: {
            'task.ts': TASK_WITHOUT_BUILD,
            'mise.toml': MISE_WITH_BUILD,
          },
          body: async (packageDirectory,) => {
            const caught = await caughtBy(async () => {
              await planPackageBuild({
                packageDirectory,
                directory: DIRECTORY,
              },);
            },);
            expect(caught instanceof MissingCiBuildTaskError,).toBe(true,);
          },
        },);
      },
    },),
    it({
      name: 'refuses a package whose build task is a dotted key',
      fn: async () => {
        await withPackage({
          files: { 'mise.toml': MISE_WITH_BUILD_DOTTED, },
          body: async (packageDirectory,) => {
            const caught = await caughtBy(async () => {
              await planPackageBuild({
                packageDirectory,
                directory: DIRECTORY,
              },);
            },);
            expect(caught instanceof MissingCiBuildTaskError,).toBe(true,);
          },
        },);
      },
    },),
    it({
      name: 'refuses a package whose build task header carries a comment',
      fn: async () => {
        await withPackage({
          files: { 'mise.toml': MISE_WITH_BUILD_COMMENTED, },
          body: async (packageDirectory,) => {
            const caught = await caughtBy(async () => {
              await planPackageBuild({
                packageDirectory,
                directory: DIRECTORY,
              },);
            },);
            expect(caught instanceof MissingCiBuildTaskError,).toBe(true,);
          },
        },);
      },
    },),
    it({
      name: 'plans no build when mise.toml names the build header only inside a string',
      fn: async () => {
        await withPackage({
          files: { 'mise.toml': MISE_NAMING_BUILD_IN_A_STRING, },
          body: async (packageDirectory,) => {
            expect(await planPackageBuild({
              packageDirectory,
              directory: DIRECTORY,
            },),).toEqual({ kind: 'none', },);
          },
        },);
      },
    },),
    it({
      name: 'plans no build when neither file declares build',
      fn: async () => {
        await withPackage({
          files: {
            'task.ts': TASK_WITHOUT_BUILD,
            'mise.toml': MISE_WITHOUT_BUILD,
          },
          body: async (packageDirectory,) => {
            expect(await planPackageBuild({
              packageDirectory,
              directory: DIRECTORY,
            },),).toEqual({ kind: 'none', },);
          },
        },);
      },
    },),
    it({
      name: 'plans no build for a package with no task files at all',
      fn: async () => {
        await withPackage({
          files: {},
          body: async (packageDirectory,) => {
            expect(await planPackageBuild({
              packageDirectory,
              directory: DIRECTORY,
            },),).toEqual({ kind: 'none', },);
          },
        },);
      },
    },),
  ],
},);
