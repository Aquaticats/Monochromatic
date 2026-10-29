import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  createMemoryRootFilesystem,
  findRoot,
  GIT_REPOSITORY,
  type RootFilesystem,
  RootNotFoundError,
} from '@monochromatic-dev/module-fs-path/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  GIT_MARKER_DIRECTORIES,
  GIT_MARKER_FILES,
  materialiseGitMarker,
  RSYNC_EXCLUDES,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Work tree root used by the in-memory cases.
 */
const TREE = '/w';

/**
 Nested directory a package test would start its walk from.
 */
const NESTED = `${TREE}/package/module/example/src`;

/**
 In-memory tree carrying the declared marker shape under {@link TREE}.
 */
const declaredShape = createMemoryRootFilesystem({
  directories: [
    NESTED,
    ...GIT_MARKER_DIRECTORIES.map(function underTree(relative: string,): string {
      return `${TREE}/${relative}`;
    },),
  ],
  files: Object.fromEntries(Object.entries(GIT_MARKER_FILES,).map(function underTreeFile(
    [relative, content,]: readonly [string, string,],
  ): readonly [string, string,] {
    return [`${TREE}/${relative}`, content,];
  },),),
},);

/**
 In-memory tree with only an empty `.git` directory, the shape the work tree
 had before this marker existed.
 */
const emptyMarker = createMemoryRootFilesystem({
  directories: [
    NESTED,
    `${TREE}/.git`,
  ],
},);

/**
 Rejection of a GIT_REPOSITORY walk from {@link NESTED} over `fs`, or
 `undefined` when the walk fulfils.

 @param fs - in-memory tree to walk

 @returns caught rejection, or nothing

 @example
 ```ts
 expect(await walkRejection(emptyMarker)).toBeInstanceOf(RootNotFoundError);
 ```
 */
async function walkRejection(fs: RootFilesystem,): Promise<unknown> {
  try {
    await findRoot({
      cwd: NESTED,
      fs,
      marker: GIT_REPOSITORY,
    },);
  }
  catch (error: unknown) {
    return error;
  }
  return undefined;
}

//endregion Fixtures

await describe({
  name: 'shard copy exclusions',
  children: [
    it({
      name: 'omits generated caches from both rsync and the baked image',
      fn: async () => {
        /**
         Image build exclusions resolved from the checked-in runtime recipe.
         */
        const ignore = await readFile(
          new URL('../../runtime/containerignore', import.meta.url),
          'utf8',
        );
        expect(RSYNC_EXCLUDES,).toContain('**/.cache',);
        expect(ignore.split('\n',),).toContain('**/.cache',);
      },
    },),
  ],
},);

await describe({
  name: materialiseGitMarker.name,
  children: [
    it({
      name: 'declared shape satisfies the GIT_REPOSITORY marker from a nested directory',
      fn: async function declaredShapeAccepted(): Promise<void> {
        expect(await findRoot({
          cwd: NESTED,
          fs: declaredShape,
          marker: GIT_REPOSITORY,
        },),).toBe(TREE,);
      },
    },),
    it({
      name: 'an empty .git directory alone is rejected, which is why the shape exists',
      fn: async function emptyMarkerRejected(): Promise<void> {
        expect(await walkRejection(emptyMarker,),).toBeInstanceOf(RootNotFoundError,);
      },
    },),
    it({
      name: 'materialised marker on disk is found from a nested directory, twice over',
      fn: async function onDiskFound(): Promise<void> {
        /**
         Throwaway work tree.
         */
        const dir = await mkdtemp(join(
          tmpdir(),
          'mutation-test-marker-',
        ),);
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(
              dir,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Nested directory inside the throwaway tree.
         */
        const nested = join(
          dir,
          'package',
          'module',
          'example',
          'src',
        );
        await mkdir(
          nested,
          { recursive: true, },
        );
        await materialiseGitMarker({ dir, },);
        // A second run must be a no-op rather than a failure.
        await materialiseGitMarker({ dir, },);
        expect(await findRoot({
          cwd: nested,
          marker: GIT_REPOSITORY,
        },),).toBe(dir,);
      },
    },),
  ],
},);
