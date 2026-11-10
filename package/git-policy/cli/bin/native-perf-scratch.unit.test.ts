import { spawn } from 'node:child_process';
import {
  access,
  mkdir,
  mkdtempDisposable,
  readdir,
  realpath,
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

 @returns the tree, which removes itself when the caller's scope ends, so a failing assertion leaves no directory
   behind
 */
async function treeWithPack(): Promise<Readonly<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}>> {
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
  const directory = await mkdtempDisposable(join(
    scratchParent,
    'perf-scratch-test-',
  ));
  await mkdir(join(
    directory.path,
    'pack',
  ));
  return directory;
}

/**
 Wait until the writer has put its first file into `root/pack`.

 @param root - tree the writer works in

 @param attempts - how many 10 millisecond polls are left

 @throws Error when no file appears within the polls

 @example
 ```ts
 await waitForFirstFile({ root, attempts: 200 });
 ```
 */
async function waitForFirstFile({
  root,
  attempts,
}: {
  readonly root: string;
  readonly attempts: number;
}): Promise<void> {
  if (attempts <= 0)
    throw new Error('the writer never wrote');
  if ((await readdir(join(
    root,
    'pack',
  ))).length > 0)
    return;
  await pause(10);
  await waitForFirstFile({
    root,
    attempts: attempts - 1,
  });
}

/**
 Start a detached process, like the one Git detaches, that keeps writing new files into `root/pack` for a while.

 @param root - tree to write in

 @returns the promise of the exit code (nonzero when a write failed, -1 when a signal ended the writer), once the writer has written its first file; wrapped so the
   caller does not wait for the writer to end
 */
async function startDetachedWriter(root: string): Promise<{ readonly exited: Promise<number>; }> {
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
  const exited = new Promise<number>(function waitForExit(resolve) {
    child.once(
      'exit',
      function onExit(code) {
        // A writer that a signal ended has no exit code; -1 stands for it.
        resolve(code ?? (-1));
      },
    );
  });
  await waitForFirstFile({
    root,
    attempts: 200,
  });
  return { exited, };
}

await describe({
  name: removeScratchRoot.name,
  children: [
    it({
      name: 'waits for a detached writer, then removes the tree whole',
      fn: async () => {
        /**
         Tree the writer works in, removed when this test's scope ends.
         */
        await using tree = await treeWithPack();
        /**
         Path of the tree, which the function under test removes early.
         */
        const root = tree.path;
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
         Tree the writer works in, removed when this test's scope ends.
         */
        await using tree = await treeWithPack();
        /**
         Path of the tree, whose deadline the function under test runs out.
         */
        const root = tree.path;
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
        expect((failure as Error).message).toContain(root);
        expect((failure as Error).message).toContain('holds cwd');
        await exited;
        await removeScratchRoot({ root, });
      },
    }),
    it({
      name: 'finds no holder in a quiet tree',
      fn: async () => {
        /**
         Quiet tree, removed when this test's scope ends.
         */
        await using tree = await treeWithPack();
        expect(
          await findHolders(await realpath(tree.path)),
        ).toEqual([]);
      },
    }),
  ],
});
