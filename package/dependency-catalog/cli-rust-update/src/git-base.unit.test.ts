/**
 Tests for the base-revision lockfile reader, on a disposable git repository.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BaseRevisionError,
  createGitBaseLockReader,
} from './git-base.ts';

//region Fixtures

/**
 Runs git in a repository with a fixed identity.

 @param root - repository root

 @param args - git arguments

 @example
 ```ts
 await git({ root, args: ['status',], });
 ```
 */
async function git({ root, args, }: { readonly root: string; readonly args: readonly string[]; },): Promise<void> {
  await spawn('git', ['-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', '-c', 'commit.gpgsign=false', ...args,], { cwd: root, },);
}

/**
 Repository with one committed lockfile and a later working-tree edit.

 @returns repository root, removed on dispose

 @example
 ```ts
 await using repo = await makeGitRepo();
 ```
 */
async function makeGitRepo(): Promise<AsyncDisposable & { readonly root: string; }> {
  /**
   Fresh directory.
   */
  const root = await mkdtemp(join(scratchRoot(), 'rust-catalog-git-base-',),);
  await mkdir(join(root, 'package/cli/a',), { recursive: true, },);
  await writeFile(join(root, 'package/cli/a/Cargo.lock',), 'committed\n',);
  await git({ root, args: ['init', '--quiet', '--initial-branch=main',], },);
  await git({ root, args: ['add', '--', 'package/cli/a/Cargo.lock',], },);
  await git({ root, args: ['commit', '--quiet', '--message=base', '--', 'package/cli/a/Cargo.lock',], },);
  await writeFile(join(root, 'package/cli/a/Cargo.lock',), 'edited\n',);
  return {
    root,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

//endregion Fixtures

await describe({
  name: createGitBaseLockReader.name,
  children: [
    it({
      name: 'reads the committed lockfile, not the working tree',
      fn: async () => {
        /**
         Repository.
         */
        await using repo = await makeGitRepo();
        /**
         Reader at HEAD.
         */
        const readBaseLock = await createGitBaseLockReader({ root: repo.root, base: 'HEAD', },);
        expect(await readBaseLock('package/cli/a',),).toBe('committed\n',);
      },
    },),
    it({
      name: 'reads a lockfile the base lacked as empty',
      fn: async () => {
        /**
         Repository.
         */
        await using repo = await makeGitRepo();
        /**
         Reader at HEAD.
         */
        const readBaseLock = await createGitBaseLockReader({ root: repo.root, base: 'HEAD', },);
        expect(await readBaseLock('package/cli/new',),).toBe('',);
      },
    },),
    it({
      name: 'rejects a revision that names no commit, before reading any lockfile',
      fn: async () => {
        /**
         Repository.
         */
        await using repo = await makeGitRepo();
        await expect(createGitBaseLockReader({ root: repo.root, base: 'no-such-revision', },),).rejects.toThrow(BaseRevisionError,);
        await expect(createGitBaseLockReader({ root: repo.root, base: '--output=x', },),).rejects.toThrow(BaseRevisionError,);
      },
    },),
  ],
},);
