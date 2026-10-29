import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import {
  createMemoryRootFilesystem,
  findRoot,
  GIT_REPOSITORY,
  type RootFilesystem,
  RootNotFoundError,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  GIT_MARKER_DIRECTORIES,
  GIT_MARKER_FILES,
  MANIFEST_MOUNT,
  materialiseGitMarker,
  RSYNC_EXCLUDES,
  rsyncSourceArgs,
  SOURCE_FILE_LIST_NAME,
  SOURCE_MOUNT,
  WORK_MOUNT,
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

/**
 Throwaway directories standing in for the source, work, and manifest mounts.
 */
type MountFixture = {
  readonly source: string;
  readonly work: string;
  readonly list: string;
};

/**
 Points an rsync argument list at fixture directories instead of the fixed
 container mounts.
 
 @param options - Argument list and the fixture directories.
 
 @returns Arguments with every mount path replaced.
 
 @example
 ```ts
 retarget({ args: rsyncSourceArgs(), mounts });
 ```
 */
function retarget(options: {
  readonly args: readonly string[];
  readonly mounts: MountFixture;
},): readonly string[] {
  return options.args.map(function swap(arg: string,): string {
    if (arg === `${SOURCE_MOUNT}/`)
      return `${options.mounts.source}/`;

    if (arg === `${WORK_MOUNT}/`)
      return `${options.mounts.work}/`;

    if (arg === `--files-from=${MANIFEST_MOUNT}/${SOURCE_FILE_LIST_NAME}`)
      return `--files-from=${options.mounts.list}`;

    return arg;
  },);
}

/**
 Lists every file and symlink path under a directory, relative to it.
 
 @param root - Directory to walk.
 
 @returns Sorted relative paths of non-directory entries.
 
 @example
 ```ts
 expect(await filesUnder(work)).toEqual(['a.ts']);
 ```
 */
async function filesUnder(root: string,): Promise<readonly string[]> {
  /**
   Recursive directory entries.
   */
  const entries = await readdir(
    root,
    {
      recursive: true,
      withFileTypes: true,
    },
  );
  return entries
    .filter(function isNotDirectory(entry,): boolean {
      return !entry.isDirectory();
    },)
    .map(function toRelative(entry,): string {
      return join(
        entry.parentPath,
        entry.name,
      ).slice(root.length + 1,);
    },)
    .toSorted();
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
  name: rsyncSourceArgs.name,
  children: [
    it({
      name: 'reads a NUL-separated file list, tolerates missing entries, and never deletes',
      fn: async function argumentShape(): Promise<void> {
        /**
         Argument list under test.
         */
        const args = rsyncSourceArgs();
        expect(args,).toContain('--archive',);
        expect(args,).toContain('--from0',);
        expect(args,).toContain(`--files-from=${MANIFEST_MOUNT}/${SOURCE_FILE_LIST_NAME}`,);
        expect(args,).toContain('--ignore-missing-args',);
        expect(args,).not.toContain('--delete',);
        expect(args.slice(-2,),).toEqual([
          `${SOURCE_MOUNT}/`,
          `${WORK_MOUNT}/`,
        ],);
      },
    },),
    ...RSYNC_EXCLUDES.map(function excludeCase(pattern: string,) {
      return it({
        name: `excludes ${pattern} both as a listed entry and as a parent of listed files`,
        fn: async function excludePair(): Promise<void> {
          /**
           Argument list under test.
           */
          const args = rsyncSourceArgs();
          /**
           Values that follow an `--exclude` flag.
           */
          const excludes = args
            .filter(function followsExclude(_arg: string, index: number,): boolean {
              return args[index - 1] === '--exclude';
            },);
          expect(excludes,).toContain(pattern,);
          expect(excludes,).toContain(`${pattern}/***`,);
        },
      },);
    },),
    it({
      name: 'real rsync copies only listed existing files, with spaces and newlines intact, and honors the excludes',
      fn: async function realCopy(): Promise<void> {
        /**
         Throwaway parent holding the three fixture directories.
         */
        const parent = await mkdtemp(join(
          scratchRoot(),
          'mutation-rsync-',
        ),);
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(
              parent,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Fixture directories replacing the container mounts.
         */
        const mounts: MountFixture = {
          source: join(
            parent,
            'source',
          ),
          work: join(
            parent,
            'work',
          ),
          list: join(
            parent,
            'list',
          ),
        };
        /**
         Relative paths present in the fixture source.
         */
        const present = [
          'a.ts',
          'dir with space/b.ts',
          'new\nline.ts',
          'not-listed.bin',
          'pkg/node_modules/dep.js',
          'pkg/dist/out.js',
          'pkg/src/c.ts',
        ];
        await mkdir(
          mounts.work,
          { recursive: true, },
        );
        await Promise.all(present.map(async function writeSource(
          relative: string,
        ): Promise<void> {
          await mkdir(
            join(
              mounts.source,
              relative,
              '..',
            ),
            { recursive: true, },
          );
          await writeFile(
            join(
              mounts.source,
              relative,
            ),
            relative,
          );
        },),);
        await writeFile(
          mounts.list,
          [
            'a.ts',
            'dir with space/b.ts',
            'new\nline.ts',
            // Listed but absent: a tracked file deleted in the working tree.
            'deleted.ts',
            'pkg/node_modules/dep.js',
            'pkg/dist/out.js',
            'pkg/src/c.ts',
          ].map(function terminate(path: string,): string {
            return `${path}\0`;
          },).join(''),
        );

        await spawn(
          'rsync',
          retarget({
            args: rsyncSourceArgs(),
            mounts,
          },),
        );

        expect(await filesUnder(mounts.work,),).toEqual([
          'a.ts',
          'dir with space/b.ts',
          'new\nline.ts',
          'pkg/src/c.ts',
        ].toSorted(),);
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
          scratchRoot(),
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
