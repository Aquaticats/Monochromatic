import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { listSourceFiles, } from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Path holding a space, tracked in the fixture.
 */
const SPACED_PATH = 'dir with space/a b.ts';

/**
 Path holding a newline, untracked in the fixture.
 */
const NEWLINE_PATH = 'new\nline.ts';

/**
 Runs git inside a fixture repository with hooks and signing disabled.
 
 @param options - Fixture root and git arguments.
 
 @example
 ```ts
 await git({ root, args: ['add', '--', 'a.ts'] });
 ```
 */
async function git(options: {
  readonly root: string;
  readonly args: readonly string[];
},): Promise<void> {
  await spawn(
    'git',
    [
      '-C',
      options.root,
      '-c',
      'core.hooksPath=/dev/null',
      '-c',
      'commit.gpgsign=false',
      '-c',
      'user.name=fixture',
      '-c',
      'user.email=fixture@example.invalid',
      ...options.args,
    ],
    { cwd: options.root, },
  );
}

/**
 Writes one file under a root, creating parent directories.
 
 @param options - Root, relative path, and content.
 
 @example
 ```ts
 await put({ root, relative: 'src/a.ts', content: 'x' });
 ```
 */
async function put(options: {
  readonly root: string;
  readonly relative: string;
  readonly content: string;
},): Promise<void> {
  /**
   Absolute destination path.
   */
  const target = join(
    options.root,
    options.relative,
  );
  await mkdir(
    join(
      target,
      '..',
    ),
    { recursive: true, },
  );
  await writeFile(
    target,
    options.content,
  );
}

/**
 Rejection of a listing over `root`, or `undefined` when it fulfils.
 
 @param root - Repository root to list.
 
 @returns Caught rejection, or nothing.
 
 @example
 ```ts
 expect(await listingRejection(plainDirectory)).toBeInstanceOf(Error);
 ```
 */
async function listingRejection(root: string,): Promise<unknown> {
  try {
    await listSourceFiles({ repoRoot: root, },);
  }
  catch (error: unknown) {
    return error;
  }
  return undefined;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: listSourceFiles.name,
      children: [
        it({
          name: 'lists tracked and untracked-not-ignored files, omits ignored files, keeps deleted tracked files',
          fn: async function listsSourceOnly(): Promise<void> {
            /**
             Throwaway repository root.
             */
            const root = await mkdtemp(join(
              scratchRoot(),
              'mutation-list-',
            ),);
            await using _cleanup = {
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
            await git({
              root,
              args: [
                'init',
                '--quiet',
              ],
            },);
            await put({
              root,
              relative: '.gitignore',
              content: 'ignored-dir/\n*.log\n',
            },);
            await put({
              root,
              relative: 'tracked.ts',
              content: 'a',
            },);
            await put({
              root,
              relative: 'deleted.ts',
              content: 'b',
            },);
            await put({
              root,
              relative: SPACED_PATH,
              content: 'c',
            },);
            await git({
              root,
              args: [
                'add',
                '--',
                '.gitignore',
                'tracked.ts',
                'deleted.ts',
                SPACED_PATH,
              ],
            },);
            await git({
              root,
              args: [
                'commit',
                '--quiet',
                '--message',
                'fixture',
                // The git shim requires an explicit pathspec on commit.
                '--',
                '.gitignore',
                'tracked.ts',
                'deleted.ts',
                SPACED_PATH,
              ],
            },);
            // Working-tree states after the commit.
            await rm(join(
              root,
              'deleted.ts',
            ),);
            await put({
              root,
              relative: 'untracked.ts',
              content: 'd',
            },);
            await put({
              root,
              relative: NEWLINE_PATH,
              content: 'e',
            },);
            await put({
              root,
              relative: 'ignored-dir/big.bin',
              content: 'f',
            },);
            await put({
              root,
              relative: 'debug.log',
              content: 'g',
            },);

            /**
             Raw NUL-separated list.
             */
            const list = await listSourceFiles({ repoRoot: root, },);
            /**
             Paths with the trailing empty segment after the last NUL removed.
             */
            const paths = list.split('\0',).slice(
              0,
              -1,
            );

            expect(list.endsWith('\0',),).toBe(true,);
            expect(paths.toSorted(),).toEqual([
              '.gitignore',
              'deleted.ts',
              SPACED_PATH,
              NEWLINE_PATH,
              'tracked.ts',
              'untracked.ts',
            ].toSorted(),);
          },
        },),
        it({
          name: 'returns an empty list for a repository without source files',
          fn: async function emptyRepository(): Promise<void> {
            /**
             Throwaway empty repository root.
             */
            const root = await mkdtemp(join(
              scratchRoot(),
              'mutation-list-empty-',
            ),);
            await using _cleanup = {
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
            await git({
              root,
              args: [
                'init',
                '--quiet',
              ],
            },);
            expect(await listSourceFiles({ repoRoot: root, },),).toBe('',);
          },
        },),
        it({
          name: 'rejects when the root is not a git repository',
          fn: async function notARepository(): Promise<void> {
            /**
             Throwaway directory outside any repository.
             */
            const root = await mkdtemp(join(
              scratchRoot(),
              'mutation-list-plain-',
            ),);
            await using _cleanup = {
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
            expect(await listingRejection(root,),).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),
  ],
},);
