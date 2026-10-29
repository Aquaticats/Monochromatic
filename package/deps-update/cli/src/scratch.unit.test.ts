/**
 Unit tests for the scratch workspace copy, against a throwaway monorepo.

 @module
 */

import {
  access,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createScratchWorkspace,
  ProjectOutsideRootError,
} from './scratch.ts';

/**
 Throwaway monorepo that removes itself when its `await using` scope ends.
 */
type TempRepo = AsyncDisposable & {
  /**
   Absolute path to the throwaway monorepo root.
   */
  readonly root: string;
};

/**
 Creates a monorepo with root manifests, a pnpmfile pair, one project, and
 unrelated files that must not be copied.

 @returns disposable holding the root path

 @example
 ```ts
 await using repo = await makeRepo();
 ```
 */
async function makeRepo(): Promise<TempRepo> {
  /**
   Fresh repository root.
   */
  const root = await mkdtemp(join(scratchRoot(), 'deps-update-test-',),);
  await mkdir(join(root, 'package', 'grp', 'app', 'src',), { recursive: true, },);
  await Promise.all([
    writeFile(join(root, 'package.json',), '{"name":"root"}',),
    writeFile(join(root, 'pnpm-workspace.yaml',), 'minimumReleaseAge: 1440\n',),
    writeFile(join(root, 'pnpm-lock.yaml',), "lockfileVersion: '9.0'\n",),
    writeFile(join(root, '.pnpmfile.mjs',), 'export default {};\n',),
    writeFile(join(root, '.pnpmfile.policies.json',), '{}',),
    writeFile(join(root, 'README.md',), 'not copied',),
    writeFile(join(root, 'package', 'grp', 'app', 'package.json',), '{"name":"app"}',),
    writeFile(join(root, 'package', 'grp', 'app', 'src', 'index.ts',), 'not copied',),
  ],);
  return {
    root,
    [Symbol.asyncDispose]: async function removeRepo(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Reports whether a path exists.

 @param path - path to probe

 @returns whether it exists

 @example
 ```ts
 await exists('/tmp');
 ```
 */
async function exists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (!Error.isError(error,))
      throw error;
    return false;
  }
}

await describe({
  name: createScratchWorkspace.name,
  children: [
    it({
      name: 'mirrors resolution inputs only',
      fn: async () => {
        await using repo = await makeRepo();
        await using scratch = await createScratchWorkspace({
          root: repo.root,
          projectDirs: [repo.root, join(repo.root, 'package', 'grp', 'app',),],
        },);
        expect(await readFile(join(scratch.dir, 'pnpm-workspace.yaml',), 'utf8',),).toBe('minimumReleaseAge: 1440\n',);
        expect(
          await exists(join(scratch.dir, 'pnpm-lock.yaml',),),
        ).toBe(true,);
        expect(
          await exists(join(scratch.dir, '.pnpmfile.mjs',),),
        ).toBe(true,);
        expect(
          await exists(join(scratch.dir, '.pnpmfile.policies.json',),),
        ).toBe(true,);
        expect(await readFile(join(scratch.dir, 'package', 'grp', 'app', 'package.json',), 'utf8',),).toBe(
          '{"name":"app"}',
        );
        expect(
          await exists(join(scratch.dir, '.npmrc',),),
        ).toBe(false,);
        expect(
          await exists(join(scratch.dir, 'README.md',),),
        ).toBe(false,);
        expect(
          await exists(join(scratch.dir, 'package', 'grp', 'app', 'src',),),
        ).toBe(false,);
      },
    },),
    it({
      name: 'removes itself on dispose',
      fn: async () => {
        await using repo = await makeRepo();
        /**
         Scratch path captured before disposal.
         */
        const dir = await (async function scoped(): Promise<string> {
          await using scratch = await createScratchWorkspace({ root: repo.root, projectDirs: [], },);
          return scratch.dir;
        })();
        expect(await exists(dir,),).toBe(false,);
      },
    },),
    it({
      name: 'rejects a project outside the root',
      fn: async () => {
        await using repo = await makeRepo();
        /**
         Rejection from the copy.
         */
        const error = await (async function attempt(): Promise<unknown> {
          try {
            await using _scratch = await createScratchWorkspace({
              root: repo.root,
              projectDirs: [scratchRoot(),],
            },);
            return 'resolved';
          }
          catch (caught) {
            return caught;
          }
        })();
        expect(error,).toBeInstanceOf(ProjectOutsideRootError,);
      },
    },),
  ],
},);
