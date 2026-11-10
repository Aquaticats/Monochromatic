/**
 Unit tests for the live-importer declaration check.
 
 Builds a throwaway monorepo (root manifest plus a `package/grp/consumer`
 importer) so the check runs against a real `package/*\/*` layout and real
 `package.json` files.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isDeclaredByLiveImporter,
} from './declared.ts';

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
 Throws-away monorepo whose importer manifest is left as raw text, so a test
 can seed a manifest that is not valid JSON.
 
 @param consumerManifest - exact text written as `package/grp/consumer/package.json`
 
 @returns disposable holding the root path; disposal removes the tree
 
 @example
 ```ts
 await using repo = await makeRepoWithRawManifest('{ not json');
 ```
 */
async function makeRepoWithRawManifest(consumerManifest: string,): Promise<TempRepo> {
  /**
   Freshly created throwaway root path.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'catalog-tighten-declared-raw-',
  ),);
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
  /**
   Importer directory under the `package/*\/*` glob the check discovers.
   */
  const consumerDir = join(
    root,
    'package',
    'grp',
    'consumer',
  );
  await mkdir(
    consumerDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      consumerDir,
      'package.json',
    ),
    consumerManifest,
  );
  return {
    root,
    [Symbol.asyncDispose]: async function disposeRepo(): Promise<void> {
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
 Creates a throwaway monorepo with a root manifest and one importer under
 `package/grp/consumer` whose dependency fields are seeded from `deps`.
 
 @param deps - dependency-field maps to write into the consumer manifest
 
 @returns disposable holding the root path; disposal removes the tree
 
 @example
 ```ts
 await using repo = await makeRepo({ dependencies: { picomatch: 'catalog:' } });
 ```
 */
async function makeRepo(
  deps: {
    readonly dependencies?: Record<string, string>;
    readonly devDependencies?: Record<string, string>;
  },
): Promise<TempRepo> {
  /**
   Freshly created throwaway root path.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'catalog-tighten-declared-',
  ),);
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
  /**
   Importer directory under the `package/*\/*` glob the check discovers.
   */
  const consumerDir = join(
    root,
    'package',
    'grp',
    'consumer',
  );
  await mkdir(
    consumerDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      consumerDir,
      'package.json',
    ),
    JSON.stringify({
      name: 'consumer',
      ...deps,
    },),
  );
  return {
    root,
    [Symbol.asyncDispose]: async function disposeRepo(): Promise<void> {
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
 await caughtFailure(async function check() { return await isDeclaredByLiveImporter({ npmNames: [], monorepoRoot: '/repo', }); });
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

await describe({
  name: isDeclaredByLiveImporter.name,
  children: [
    it({
      name: 'is true when a workspace importer declares the name in dependencies',
      fn: async () => {
        await using repo = await makeRepo({ dependencies: { picomatch: 'catalog:', }, },);
        expect(await isDeclaredByLiveImporter({
          npmNames: ['picomatch',],
          monorepoRoot: repo.root,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'is true when the name is only in devDependencies',
      fn: async () => {
        await using repo = await makeRepo({ devDependencies: { '@types/mdx': 'catalog:', }, },);
        expect(await isDeclaredByLiveImporter({
          npmNames: ['@types/mdx',],
          monorepoRoot: repo.root,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'is true when a fallback alias candidate is declared',
      fn: async () => {
        await using repo = await makeRepo({ dependencies: { '@jsr/zod__zod': 'npm:...', }, },);
        expect(await isDeclaredByLiveImporter({
          npmNames: [
            'zod',
            '@jsr/zod__zod',
          ],
          monorepoRoot: repo.root,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'is false when no importer declares any candidate',
      fn: async () => {
        await using repo = await makeRepo({ dependencies: { picomatch: 'catalog:', }, },);
        expect(await isDeclaredByLiveImporter({
          npmNames: ['openai',],
          monorepoRoot: repo.root,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'throws instead of declaring nothing when an importer manifest is not valid JSON',
      fn: async () => {
        await using repo = await makeRepoWithRawManifest('{ "dependencies": {',);
        /**
         Whatever ended the check; folding this into "declares nothing" would
         label a declared package `UNDCL`.
         */
        const caught = await caughtFailure(async function check() {
          return await isDeclaredByLiveImporter({
            npmNames: ['picomatch',],
            monorepoRoot: repo.root,
          },);
        },);
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('package.json',);
      },
    },),
  ],
},);
