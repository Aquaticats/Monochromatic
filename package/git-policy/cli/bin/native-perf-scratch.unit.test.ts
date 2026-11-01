import { spawn } from 'node:child_process';
import {
  access,
  mkdir,
  mkdtemp,
  readdir,
  realpath,
  rm,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as pause } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findHolders,
  removeScratchRoot,
} from './native-perf-scratch.ts';

/**
 Private scratch parent shared with the benchmark itself, outside the repository.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 A scratch tree with a `pack` directory, like the benchmark's repositories.

 @returns the tree
 */
async function treeWithPack(): Promise<string> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  /**
   The new tree.
   */
  const root = await mkdtemp(join(
    scratchParent,
    'perf-scratch-test-',
  ));
  await mkdir(join(
    root,
    'pack',
  ));
  return root;
}

/**
 Start a detached process, like the one Git detaches, that keeps writing new files into `root/pack` for a while.

 @param root - tree to write in

 @returns the promise of the exit code (nonzero when a write failed), once the writer has written its first file; wrapped so the
   caller does not wait for the writer to end
 */
async function startDetachedWriter(root: string): Promise<{ readonly exited: Promise<number | null>; }> {
  /**
   Writer program: one new file every few milliseconds, exiting nonzero at the first failed write.
   */
  const program = `
    const { writeFileSync } = require('node:fs');
    try {
      for (let index = 0; index < 60; index += 1) {
        writeFileSync(process.argv[1] + '/pack/tmp_' + index, 'x');
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
      }
    } catch (error) {
      process.exit(1);
    }
  `;
  /**
   The writer, detached from this process's group and with its working directory in the tree.
   */
  const child = spawn(
    process.execPath,
    [
      '--eval',
      program,
      root,
    ],
    {
      cwd: root,
      detached: true,
      stdio: 'ignore',
    },
  );
  /**
   Settles with the exit code.
   */
  const exited = new Promise<number | null>(function waitForExit(resolve) {
    child.once(
      'exit',
      function onExit(code) {
        resolve(code);
      },
    );
  });
  for (let attempt = 0; attempt < 200; attempt += 1) {
    if ((await readdir(join(
      root,
      'pack',
    ))).length > 0)
      return { exited, };
    await pause(10);
  }
  throw new Error('the writer never wrote');
}

await describe({
  name: 'removeScratchRoot',
  children: [
    it({
      name: 'waits for a detached writer, then removes the tree whole',
      fn: async () => {
        /**
         Tree the writer works in.
         */
        const root = await treeWithPack();
        /**
         Exit of the writer, which fails if the tree vanished while it wrote.
         */
        const { exited, } = await startDetachedWriter(root);
        await removeScratchRoot({ root, });
        expect(await exited).toBe(0);
        await expect(access(root)).rejects.toThrow();
      },
    }),
    it({
      name: 'fails naming a process that still holds the tree at the deadline, and leaves the tree',
      fn: async () => {
        /**
         Tree the writer works in.
         */
        const root = await treeWithPack();
        /**
         Exit of the writer.
         */
        const { exited, } = await startDetachedWriter(root);
        /**
         Failure raised at the short deadline.
         */
        const failure = await removeScratchRoot({
          root,
          deadlineMilliseconds: 20,
        }).then(
          function unexpected() {
            return undefined;
          },
          function caught(error: unknown) {
            return error;
          },
        );
        expect(failure).toBeInstanceOf(Error);
        expect(String((failure as Error).message)).toContain(root);
        expect(String((failure as Error).message)).toContain('holds cwd');
        await exited;
        await removeScratchRoot({ root, });
      },
    }),
    it({
      name: 'finds no holder in a quiet tree',
      fn: async () => {
        /**
         Quiet tree.
         */
        const root = await treeWithPack();
        expect(await findHolders(await realpath(root))).toEqual([]);
        await rm(
          root,
          {
            recursive: true,
            force: true,
          },
        );
      },
    }),
  ],
});
