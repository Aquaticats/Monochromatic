/**
 Unit tests for pnpm virtual-store classification probing.
 
 Builds a throwaway `.pnpm` store on disk so the probe runs against a real
 directory layout, including the scope-mangled directory names pnpm emits.
 
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
  NOT_IN_STORE,
  readStoreVersions,
} from './store-probe.ts';

/**
 Throwaway monorepo root that removes itself when its `await using` scope ends,
 so tests never leak temp directories and never reach for a banned `finally`.
 */
type TempRoot = AsyncDisposable & {
  /**
   Absolute path to the throwaway monorepo root.
   */
  readonly root: string;
};

/**
 Creates a disposable throwaway monorepo root under the OS temp directory.
 
 @returns disposable holding the root path; disposal removes the tree
 
 @example
 ```ts
 await using temp = await makeTempRoot();
 // use temp.root ...
 ```
 */
async function makeTempRoot(): Promise<TempRoot> {
  /**
   Freshly created throwaway root path.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'catalog-tighten-store-',
  ),);
  return {
    root,
    [Symbol.asyncDispose]: async function disposeTempRoot(): Promise<void> {
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
 Writes one store copy at
 `<root>/node_modules/.pnpm/<mangled>@<version>/node_modules/<name>/package.json`.
 
 @param root - throwaway monorepo root
 
 @param mangled - scope-mangled store directory prefix, e.g. `\@types+mdx`
 
 @param name - real npm name (unmangled) for the inner directory, e.g. `\@types/mdx`
 
 @param version - version to write into the manifest
 
 @example
 ```ts
 await writeStoreCopy({ root, mangled: "openai", name: "openai", version: "6.26.0" });
 ```
 */
async function writeStoreCopy(
  {
    root,
    mangled,
    name,
    version,
  }: {
    readonly root: string;
    readonly mangled: string;
    readonly name: string;
    readonly version: string;
  },
): Promise<void> {
  /**
   Inner package directory holding the manifest, under the mangled store entry.
   */
  const pkgDir = join(
    root,
    'node_modules',
    '.pnpm',
    `${mangled}@${version}`,
    'node_modules',
    name,
  );
  await mkdir(
    pkgDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      pkgDir,
      'package.json',
    ),
    JSON.stringify({
      name,
      version,
    },),
  );
}

await describe({
  name: readStoreVersions.name,
  children: [
    it({
      name: 'finds a scope-mangled transitive package by its real name',
      fn: async () => {
        await using temp = await makeTempRoot();
        await writeStoreCopy({
          root: temp.root,
          mangled: '@types+mdx',
          name: '@types/mdx',
          version: '2.0.14',
        },);
        expect(await readStoreVersions({
          npmName: '@types/mdx',
          monorepoRoot: temp.root,
          modulesDir: 'node_modules',
        },),).toEqual(['2.0.14',],);
      },
    },),

    it({
      name: 'returns distinct sorted versions when the store holds several copies',
      fn: async () => {
        await using temp = await makeTempRoot();
        await writeStoreCopy({
          root: temp.root,
          mangled: 'micromark',
          name: 'micromark',
          version: '4.0.0',
        },);
        await writeStoreCopy({
          root: temp.root,
          mangled: 'micromark',
          name: 'micromark',
          version: '3.2.0',
        },);
        expect(await readStoreVersions({
          npmName: 'micromark',
          monorepoRoot: temp.root,
          modulesDir: 'node_modules',
        },),).toEqual([
          '3.2.0',
          '4.0.0',
        ],);
      },
    },),

    it({
      name: 'does not match a sibling whose name shares a prefix',
      fn: async () => {
        await using temp = await makeTempRoot();
        await writeStoreCopy({
          root: temp.root,
          mangled: 'micromark-core-commonmark',
          name: 'micromark-core-commonmark',
          version: '2.0.0',
        },);
        expect(await readStoreVersions({
          npmName: 'micromark',
          monorepoRoot: temp.root,
          modulesDir: 'node_modules',
        },),).toBe(NOT_IN_STORE,);
      },
    },),

    it({
      name: 'returns the not-in-store sentinel when there is no .pnpm directory',
      fn: async () => {
        await using temp = await makeTempRoot();
        expect(await readStoreVersions({
          npmName: 'preact',
          monorepoRoot: temp.root,
          modulesDir: 'node_modules',
        },),).toBe(NOT_IN_STORE,);
      },
    },),
  ],
},);
