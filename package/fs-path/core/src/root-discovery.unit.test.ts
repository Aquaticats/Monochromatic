/**
 Walker and memo semantics, plus the few cases that must touch a real
 filesystem to prove the Node adapter reads real files and links.

 Marker shapes are covered as data tables in `root-marker.unit.test.ts`.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join as nodeJoin, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMemoryRootFilesystem,
  fileNamed,
  findRoot,
  findRootCached,
  GIT_REPOSITORY,
  isAbsolute,
  MISE_MONOREPO,
  packageNamed,
  PNPM_WORKSPACE,
  type RootMarker,
  type RootMatcherArgs,
  RootNotFoundError,
} from '@monochromatic-dev/fs-path-core';

//region Helpers

/**
 Marker that records every directory it is asked about and accepts the
 given one.

 @param name - marker name, distinct per test so memo keys never collide

 @param accept - directory to accept; absent means accept nothing

 @returns marker plus the list of directories probed so far

 @example
 ```ts
 const { marker, probed } = recordingMarker({ name: 'r1', accept: '/repo' });
 ```
 */
function recordingMarker({
  name,
  accept,
}: {
  readonly name: string;
  readonly accept?: string;
},): {
  readonly marker: RootMarker;
  readonly probed: string[];
} {
  /**
   Directories probed, in walk order.
   */
  const probed: string[] = [];
  return {
    marker: {
      matches: function record({ dir, }: RootMatcherArgs,): Promise<boolean> {
        probed.push(dir,);
        return Promise.resolve(dir === accept,);
      },
      name,
    },
    probed,
  };
}

/**
 Disposable directory tree under the temporary directory.
 */
type TempTree = {
  /**
   Fixture root.
   */
  readonly root: string;

  /**
   Nested child directory used as the walk start.
   */
  readonly nested: string;

  /**
   Removes the tree.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a temporary tree with a nested start directory.

 @param prefix - `mkdtemp` prefix

 @param parent - parent directory, defaulting to the OS temporary directory

 @returns disposable tree

 @example
 ```ts
 await using tree = await createTempTree({ prefix: 'fs-path-' });
 ```
 */
async function createTempTree({
  prefix,
  parent,
}: {
  readonly prefix: string;
  readonly parent?: string;
},): Promise<TempTree> {
  /**
   Fixture root.
   */
  const root = await mkdtemp(nodeJoin(
    parent ?? scratchRoot(),
    prefix,
  ),);
  /**
   Nested start directory.
   */
  const nested = nodeJoin(
    root,
    'child',
    'grandchild',
  );
  await mkdir(
    nested,
    { recursive: true, },
  );
  return {
    nested,
    root,
    [Symbol.asyncDispose](): Promise<void> {
      return rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Writes objects and refs directories under a Git administrative directory.

 @param gitDirectory - administrative directory

 @example
 ```ts
 await writeGitSignatures(nodeJoin(root, '.git'));
 ```
 */
async function writeGitSignatures(gitDirectory: string,): Promise<void> {
  await Promise.all([
    mkdir(
      nodeJoin(
        gitDirectory,
        'objects',
      ),
      { recursive: true, },
    ),
    mkdir(
      nodeJoin(
        gitDirectory,
        'refs',
      ),
      { recursive: true, },
    ),
  ],);
}

/**
 Name in this package's own manifest.
 */
const OWN_PACKAGE_NAME = '@monochromatic-dev/fs-path-core';

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: findRoot.name,
      children: [
        it({
          name: 'probes the start directory first, then each ancestor up to the root',
          fn: async () => {
            const { marker, probed, } = recordingMarker({ name: 'walk-order', },);
            /**
             Rejection captured after the full climb.
             */
            let caught: unknown;
            try {
              await findRoot({
                cwd: '/repo/a/b',
                fs: createMemoryRootFilesystem(),
                marker,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(RootNotFoundError,);
            expect(probed,).toEqual(['/repo/a/b', '/repo/a', '/repo', '/',],);
          },
        },),
        it({
          name: 'stops at the first accepted ancestor',
          fn: async () => {
            const { marker, probed, } = recordingMarker({
              accept: '/repo/a',
              name: 'walk-stop',
            },);
            expect(await findRoot({
              cwd: '/repo/a/b',
              fs: createMemoryRootFilesystem(),
              marker,
            },),).toBe('/repo/a',);
            expect(probed,).toEqual(['/repo/a/b', '/repo/a',],);
          },
        },),
        it({
          name: 'accepts the filesystem root itself',
          fn: async () => {
            expect(await findRoot({
              cwd: '/repo/a/b',
              fs: createMemoryRootFilesystem({ files: { '/deno.json': '{}', }, },),
              marker: fileNamed('deno.json',),
            },),).toBe('/',);
          },
        },),
        it({
          name: 'walks from a cwd spelled with a trailing slash',
          fn: async () => {
            expect(await findRoot({
              cwd: '/repo/a/b/',
              fs: createMemoryRootFilesystem({ files: { '/repo/pnpm-workspace.yaml': '', }, },),
              marker: PNPM_WORKSPACE,
            },),).toBe('/repo',);
          },
        },),
        it({
          name: 'rejects with RootNotFoundError carrying the marker name and start directory',
          fn: async () => {
            /**
             Rejection captured for inspection.
             */
            let caught: unknown;
            try {
              await findRoot({
                cwd: '/nowhere/deep',
                fs: createMemoryRootFilesystem(),
                marker: GIT_REPOSITORY,
              },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(RootNotFoundError,);
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as RootNotFoundError).name,).toBe('RootNotFoundError',);
            expect((caught as RootNotFoundError).marker,).toBe('git repository',);
            expect((caught as RootNotFoundError).startDir,).toBe('/nowhere/deep',);
            expect((caught as RootNotFoundError).message,).toBe('no git repository root found walking up from /nowhere/deep',);
          },
        },),
        it({
          name: 'defaults cwd to the process working directory',
          fn: async () => {
            const { marker, probed, } = recordingMarker({
              accept: process.cwd(),
              name: 'default-cwd',
            },);
            expect(await findRoot({
              fs: createMemoryRootFilesystem(),
              marker,
            },),).toBe(process.cwd(),);
            expect(probed,).toEqual([process.cwd(),],);
          },
        },),
        it({
          name: 'runs a custom marker written as one object literal',
          fn: async () => {
            /**
             Marker for a directory holding a `quarantine` directory.
             */
            const QUARANTINE_WARD: RootMarker = {
              matches: async function matchesWard({
                dir,
                fs,
              }: RootMatcherArgs,): Promise<boolean> {
                return await fs.isDirectory(`${dir}/quarantine`,);
              },
              name: 'quarantine ward',
            };
            expect(await findRoot({
              cwd: '/registry/north/3',
              fs: createMemoryRootFilesystem({ directories: ['/registry/north/3', '/registry/quarantine',], },),
              marker: QUARANTINE_WARD,
            },),).toBe('/registry',);
          },
        },),
      ],
    },),

    describe({
      name: findRootCached.name,
      children: [
        it({
          name: 'shares one walk across concurrent and later calls with the same marker and cwd',
          fn: async () => {
            const { marker, probed, } = recordingMarker({
              accept: '/memo/a',
              name: 'memo-share',
            },);
            /**
             Concurrent first callers.
             */
            const concurrent = await Promise.all([
              findRootCached({ cwd: '/memo/a', marker, },),
              findRootCached({ cwd: '/memo/a', marker, },),
              findRootCached({ cwd: '/memo/a', marker, },),
            ],);
            /**
             Caller after the walk settled.
             */
            const later = await findRootCached({ cwd: '/memo/a', marker, },);
            expect([...concurrent, later,],).toAllBe();
            expect(later,).toBe('/memo/a',);
            expect(probed,).toEqual(['/memo/a',],);
          },
        },),
        it({
          name: 'walks again for another start directory',
          fn: async () => {
            const { marker, probed, } = recordingMarker({
              accept: '/memo',
              name: 'memo-cwd',
            },);
            await findRootCached({ cwd: '/memo/x', marker, },);
            await findRootCached({ cwd: '/memo/y', marker, },);
            expect(probed,).toEqual(['/memo/x', '/memo', '/memo/y', '/memo',],);
          },
        },),
        it({
          name: 'walks again for another marker name at the same start directory',
          fn: async () => {
            const first = recordingMarker({
              accept: '/memo/z',
              name: 'memo-name-1',
            },);
            const second = recordingMarker({
              accept: '/memo/z',
              name: 'memo-name-2',
            },);
            await findRootCached({ cwd: '/memo/z', marker: first.marker, },);
            await findRootCached({ cwd: '/memo/z', marker: second.marker, },);
            expect(first.probed,).toEqual(['/memo/z',],);
            expect(second.probed,).toEqual(['/memo/z',],);
          },
        },),
        it({
          name: 'keeps a rejection for its key',
          fn: async () => {
            const { marker, } = recordingMarker({ name: 'memo-reject', },);
            /**
             Settled outcomes of two calls with the same key.
             */
            const [first, second,] = await Promise.allSettled([
              findRootCached({ cwd: '/memo/none', marker, },),
              findRootCached({ cwd: '/memo/none', marker, },),
            ],);
            expect(first.status,).toBe('rejected',);
            expect(second.status,).toBe('rejected',);
            expect((first as PromiseRejectedResult).reason,).toBeInstanceOf(RootNotFoundError,);
            expect((first as PromiseRejectedResult).reason,).toBe((second as PromiseRejectedResult).reason,);
          },
        },),
        it({
          name: 'resolves an omitted cwd to the process working directory at call time',
          fn: async () => {
            const { marker, probed, } = recordingMarker({
              accept: process.cwd(),
              name: 'memo-default-cwd',
            },);
            await findRootCached({ marker, },);
            await findRootCached({ cwd: process.cwd(), marker, },);
            expect(probed,).toEqual([process.cwd(),],);
          },
        },),
        it({
          name: 'matches findRoot for the same arguments',
          fn: async () => {
            expect(await findRootCached({ cwd: import.meta.dirname, marker: MISE_MONOREPO, },),)
              .toBe(await findRoot({ cwd: import.meta.dirname, marker: MISE_MONOREPO, },),);
          },
        },),
      ],
    },),

    describe({
      name: 'real filesystem',
      children: [
        it({
          name: 'mise, git, and pnpm markers resolve to one absolute directory from this package',
          fn: async () => {
            /**
             Roots by marker, walked from this test file's directory.
             */
            const roots = await Promise.all([
              findRoot({ cwd: import.meta.dirname, marker: MISE_MONOREPO, },),
              findRoot({ cwd: import.meta.dirname, marker: GIT_REPOSITORY, },),
              findRoot({ cwd: import.meta.dirname, marker: PNPM_WORKSPACE, },),
            ],);
            expect(roots,).toAllBe();
            expect(isAbsolute(roots[0] ?? '',),).toBe(true,);
          },
        },),
        it({
          name: 'the process working directory lies inside the monorepo root',
          fn: async () => {
            /**
             Root from the default start directory.
             */
            const root = await findRoot({ marker: MISE_MONOREPO, },);
            /**
             Runtime-native process directory.
             */
            const cwd = process.cwd();
            expect((cwd === root) || cwd.startsWith(`${root}/`,),).toBe(true,);
          },
        },),
        it({
          name: "packageNamed finds this package's directory from its src/",
          fn: async () => {
            /**
             Root of this package.
             */
            const root = await findRoot({
              cwd: import.meta.dirname,
              marker: packageNamed(OWN_PACKAGE_NAME,),
            },);
            expect(root.endsWith('/fs-path/core',),).toBe(true,);
          },
        },),
        it({
          name: 'reads a CRLF mise.toml from disk',
          fn: async () => {
            await using tree = await createTempTree({ prefix: 'fs-path-crlf-', },);
            await writeFile(
              nodeJoin(
                tree.root,
                'mise.toml',
              ),
              '[tools]\r\nnode = "24"\r\n\r\n[monorepo]\r\n',
            );
            expect(await findRoot({ cwd: tree.nested, marker: MISE_MONOREPO, },),).toBe(tree.root,);
          },
        },),
        it({
          name: 'reads a gitfile and a symbolic-link HEAD from disk',
          fn: async () => {
            await using tree = await createTempTree({ prefix: 'fs-path-gitfile-', },);
            /**
             Administrative directory targeted by the gitfile.
             */
            const gitDirectory = nodeJoin(
              tree.root,
              '.git-target',
            );
            await writeGitSignatures(gitDirectory,);
            await symlink(
              'refs/heads/missing',
              nodeJoin(
                gitDirectory,
                'HEAD',
              ),
            );
            await writeFile(
              nodeJoin(
                tree.root,
                '.git',
              ),
              'gitdir: .git-target\n',
            );
            expect(await findRoot({ cwd: tree.nested, marker: GIT_REPOSITORY, },),).toBe(tree.root,);
          },
        },),
        it({
          name: 'rejects from a markerless temporary tree',
          fn: async () => {
            await using tree = await createTempTree({ prefix: 'fs-path-markerless-', },);
            /**
             Rejection captured for inspection.
             */
            let caught: unknown;
            try {
              await findRoot({ cwd: tree.nested, marker: PNPM_WORKSPACE, },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(RootNotFoundError,);
            expect((caught as RootNotFoundError).startDir,).toBe(tree.nested,);
          },
        },),
        it({
          name: 'preserves logical home spelling instead of fabricating a var-home path',
          skip: !(process.env.HOME?.startsWith('/home/',) ?? false)
            ? 'requires POSIX /home alias fixture'
            : false,
          fn: async () => {
            /**
             Logical home path supplied by the environment.
             */
            const logicalHome = process.env.HOME;
            if (logicalHome === undefined)
              throw new Error('HOME disappeared after skip evaluation.',);
            await using tree = await createTempTree({
              parent: logicalHome,
              prefix: 'fs-path-home-alias-',
            },);
            /**
             Administrative directory at the fixture root.
             */
            const gitDirectory = nodeJoin(
              tree.root,
              '.git',
            );
            await writeGitSignatures(gitDirectory,);
            await writeFile(
              nodeJoin(
                gitDirectory,
                'HEAD',
              ),
              'ref: refs/heads/main\n',
            );
            expect(await findRoot({ cwd: tree.nested, marker: GIT_REPOSITORY, },),).toBe(tree.root,);
          },
        },),
      ],
    },),
  ],
},);
