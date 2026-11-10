/**
 Unit tests for workspace root discovery and safe catalog alias candidate resolution.
 
 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  scratchRoot,
} from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  discoverWorkspaceRoots,
  resolveNpmNames,
} from './version-resolve.ts';

//region Fixtures

/**
 Mode a directory needs for its own removal, restored before disposal so the
 scratch tree can be cleaned up after a permission test.
 */
const READABLE_DIRECTORY_MODE = 0o755;

/**
 Mode that denies every read, search, and list right to the owning process.
 */
const UNREADABLE_DIRECTORY_MODE = 0o000;

/**
 Throwaway monorepo that removes itself when its `await using` scope ends.
 */
type TempRepo = AsyncDisposable & {
  /**
   Absolute path to the throwaway monorepo root.
   */
  readonly root: string;
  /**
   Absolute path to the throwaway `package/` directory, whose mode the caller may change.
   */
  readonly packagesDir: string;
};

/**
 Creates a throwaway monorepo holding one workspace package under
 `package/grp/consumer`.
 
 @returns disposable holding the root and `package/` paths; disposal restores
 the directory mode and removes the tree
 
 @example
 ```ts
 await using repo = await makeRepo();
 ```
 */
async function makeRepo(): Promise<TempRepo> {
  /**
   Freshly created throwaway root path.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'catalog-tighten-roots-',
  ),);
  /**
   `package/` directory holding the single category the scan walks.
   */
  const packagesDir = join(
    root,
    'package',
  );
  /**
   One workspace package under the category, so discovery has a hit to report.
   */
  const consumerDir = join(
    packagesDir,
    'grp',
    'consumer',
  );
  await mkdir(
    consumerDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      root,
      'package.json',
    ),
    JSON.stringify({
      name: 'root',
      private: true,
    },),
  );
  await writeFile(
    join(
      consumerDir,
      'package.json',
    ),
    JSON.stringify({ name: 'consumer', }),
  );
  return {
    root,
    packagesDir,
    [Symbol.asyncDispose]: async function disposeRepo(): Promise<void> {
      // A permission test leaves `package/` unreadable, which blocks its own removal.
      await chmod(
        packagesDir,
        READABLE_DIRECTORY_MODE,
      );
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Runs an operation that is expected to fail and keeps whatever ended it.
 
 @param operation - work to run
 
 @returns thrown value, or the settled result when the operation did not throw
 
 @example
 ```ts
 await caughtFailure(async function discover() { return await discoverWorkspaceRoots('/repo'); });
 ```
 */
async function caughtFailure(operation: () => Promise<unknown>,): Promise<unknown> {
  try {
    return await operation();
  }
  catch (error: unknown) {
    return error;
  }
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: discoverWorkspaceRoots.name,
      children: [
        it({
          name: 'lists the workspace package directories under package/',
          fn: async () => {
            await using repo = await makeRepo();
            expect(await discoverWorkspaceRoots(repo.root,),).toEqual([join(
              repo.packagesDir,
              'grp',
              'consumer',
            ),],);
          },
        },),

        it({
          name: 'throws instead of reporting no workspace packages when package/ cannot be read',
          fn: async () => {
            await using repo = await makeRepo();
            await chmod(
              repo.packagesDir,
              UNREADABLE_DIRECTORY_MODE,
            );
            /**
             Whatever ended the scan; a silent empty list would classify every
             catalog entry as missing.
             */
            const caught = await caughtFailure(async function discover() {
              return await discoverWorkspaceRoots(repo.root,);
            },);
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('package',);
          },
        },),
      ],
    },),
    describe({
      name: resolveNpmNames.name,
      children: [
        it({
          name: 'returns the catalog key before a valid scoped alias target',
          fn: async () => {
            expect(resolveNpmNames({
              catalogKey: 'zod',
              catalogValue: 'npm:@jsr/zod__zod@>=4.1.8',
            },),).toEqual([
              'zod',
              '@jsr/zod__zod',
            ],);
          },
        },),

        it({
          name: 'returns only the key when an alias target is invalid',
          fn: async () => {
            expect(resolveNpmNames({
              catalogKey: 'zod',
              catalogValue: 'npm:../../outside@>=1.0.0',
            },),).toEqual(['zod',],);
          },
        },),

        it({
          name: 'returns no candidates when the catalog key is invalid',
          fn: async () => {
            expect(resolveNpmNames({
              catalogKey: '../../outside',
              catalogValue: '>=1.0.0',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
