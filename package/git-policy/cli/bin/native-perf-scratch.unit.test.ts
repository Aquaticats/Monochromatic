import { spawn } from 'node:child_process';
import {
  access,
  mkdir,
  mkdtempDisposable,
  readdir,
  readlink,
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
  DEFAULT_HOLDER_DEADLINE_MILLISECONDS,
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
 Milliseconds a removal the test expects to fail is given, which is the wait's own length and never the test's window.

 The writer holds the tree until the test ends it, so the call's answer rests on that holder and not on this number: a slow
 host lengthens a scan of the process table, and no length of wait turns a holder into none.
 */
const failingRemovalDeadlineMilliseconds = 250;

/**
 Milliseconds the wait test watches a removal that must not answer, sized as several scans of the process table.

 Measured on 2026-11-27 at a load average of 14 to 30 (issue 6596): one scan of `findHolders` cost 107 to 135 ms on this
 host. A host that wakes this test's own thread late spends this window and never the code's promise, which is that a call
 with a live holder answers nothing.
 */
const unansweredWindowMilliseconds = 1_000;

/**
 Milliseconds a writer is given to hold the tree and to have written in it before the fixture gives up on it.

 Measured on 2026-11-27: the writer's working directory is the tree from its `exec`, and its first file was there 23 to 25
 ms after the spawn. This bound only stops a hang; no test judges anything by it.
 */
const writerReadinessMilliseconds = 10_000;

/**
 Milliseconds between two reads of the process table while the fixture waits for its own writer.

 A poll interval, not a bound: the bound is `writerReadinessMilliseconds`, which this interval divides into polls.
 */
const readinessPollMilliseconds = 10;

/**
 How many reads of the process table the fixture makes before it gives up on its own writer.

 Derived from the bound it waits for, so the two cannot disagree.
 */
const writerReadinessPolls = Math.ceil(writerReadinessMilliseconds / readinessPollMilliseconds);

/**
 Milliseconds the writer keeps writing before it gives up on being ended.

 The tests end their writer long before this, so an ending with its own code 0 is always the test's own act. A writer that
 a killed test process left behind ends itself at this bound instead of writing here for a day.
 */
const writerHoldMilliseconds = 30_000;

/**
 The writer's program: one new file in `pack` every few milliseconds, while this test's writer holds the tree.

 The writer's working directory is the tree it writes in. Its ending is the test's own act: `SIGTERM` takes the exit code 0,
 and the stated hold takes 2 for a writer nothing ended. A write that finds the tree gone takes 1, so a removal that ran
 while this writer was still writing is the value its end answers with.

 The writing hangs on a timer rather than on a loop that never turns, because a program answers a signal only when its own
 loop turns, and a loop that never turns would leave a writer no test could end.
 */
const writerProgram = `
  const { writeFileSync } = require('node:fs');

  let index = 0;

  const write = () => {
    try {
      writeFileSync(process.argv[1] + '/pack/tmp_' + index, 'x');
      index += 1;
    } catch (error) {
      process.exit(1);
    }
  };

  process.on('SIGTERM', () => { process.exit(0); });
  write();
  setInterval(write, 10);
  setTimeout(() => { process.exit(2); }, Number(process.argv[2]));
`;

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
 Where the kernel says one process's working directory is.

 @param pid - the process

 @param root - the tree this fixture is about, named in the failure so a writer that ended says which tree it left

 @returns the canonical path of that working directory

 @throws Error when the process has ended, since a process that is gone has no working directory to read

 @example
 ```ts
 await workingDirectoryOf({ pid: 4711, root: '/home/user/temp/agent/scratch' });
 ```
 */
async function workingDirectoryOf({
  pid,
  root,
}: {
  readonly pid: number;
  readonly root: string;
}): Promise<string> {
  try {
    return await readlink(join(
      '/proc',
      String(pid),
      'cwd',
    ));
  }
  catch (error) {
    throw new Error(
      `the writer ${String(pid)} ended before it held ${root}: ${String(error)}`,
      { cause: error, },
    );
  }
}

/**
 One read of the process table, and one more poll while the writer has not been seen holding the tree.

 @param pid - the writer's process number

 @param root - the tree the writer is meant to hold

 @param canonicalRoot - that tree as the process table spells it, since a link there holds the resolved path

 @param pollsLeft - how many reads are left before the fixture gives up

 @throws Error when the writer has ended, or when no read found it holding the tree

 @example
 ```ts
 await pollUntilTheWriterHoldsTheTree({ pid: 4711, root, canonicalRoot, pollsLeft: 1000 });
 ```
 */
async function pollUntilTheWriterHoldsTheTree({
  pid,
  root,
  canonicalRoot,
  pollsLeft,
}: {
  readonly pid: number;
  readonly root: string;
  readonly canonicalRoot: string;
  readonly pollsLeft: number;
}): Promise<void> {
  if (pollsLeft <= 0)
    throw new Error(
      `the writer ${String(pid)} neither held ${root} nor wrote in it within ${String(writerReadinessMilliseconds)} ms`,
    );
  /**
   Where the kernel says the writer's working directory is.
   */
  const cwd = await workingDirectoryOf({
    pid,
    root,
  });
  /**
   What the writer has written so far.
   */
  const written = await readdir(join(
    root,
    'pack',
  ));
  if ((cwd === canonicalRoot) && (written.length > 0))
    return;
  await pause(readinessPollMilliseconds);
  await pollUntilTheWriterHoldsTheTree({
    pid,
    root,
    canonicalRoot,
    pollsLeft: pollsLeft - 1,
  });
}

/**
 Wait until the writer holds the tree, in the two senses this file's tests rely on: the tree is its working directory, and
 it has written a file there.

 The test reads this fact from the process table itself, so a removal is never asked about a holder the test only hopes is
 there, and a writer that ended before it could hold anything fails here naming itself instead of making an assertion
 below pass or fail by luck.

 @param pid - the writer's process number, taken from the spawn this test made

 @param root - the tree the writer is meant to hold

 @throws Error when the writer has ended, or when it neither holds the tree nor writes in it before the bound

 @example
 ```ts
 await waitUntilTheWriterHoldsTheTree({ pid: 4711, root: '/home/user/temp/agent/scratch' });
 ```
 */
async function waitUntilTheWriterHoldsTheTree({
  pid,
  root,
}: {
  readonly pid: number;
  readonly root: string;
}): Promise<void> {
  /**
   The tree as the process table spells it, since a link there holds the resolved path.
   */
  const canonicalRoot = await realpath(root);
  await pollUntilTheWriterHoldsTheTree({
    pid,
    root,
    canonicalRoot,
    pollsLeft: writerReadinessPolls,
  });
}

/**
 Start a detached process, like the one Git detaches, that holds `root` as its working directory and keeps writing new
 files into `root/pack` until the caller ends it.

 @param root - tree to write in and to hold

 @returns the writer's process number, the end that stops it, and the owner that ends it when the caller's scope ends, so
   no path of a test leaves it running

 @throws Error when the writer cannot be started, or does not hold the tree

 @example
 ```ts
 await using writer = await startDetachedWriter('/home/user/temp/agent/scratch');
 ```
 */
async function startDetachedWriter(root: string): Promise<Readonly<{
  readonly pid: number;
  readonly end: () => Promise<number>;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}>> {
  /**
   The writer, detached from this process's group and with its working directory in the tree, holding nothing of this
   test's output.
   */
  const child = spawn(
    process.execPath,
    [
      '--eval',
      writerProgram,
      root,
      String(writerHoldMilliseconds),
    ],
    {
      cwd: root,
      detached: true,
      stdio: 'ignore',
    },
  );
  /**
   The writer's process number, which names it in the failure the test asks for and in the process table it is read from.
   */
  const { pid } = child;
  if (pid === undefined)
    throw new Error('the writer was started without a process number');
  /**
   Settles with the code the writer ended with, or -1 when a signal ended it before it answered that signal itself.
   */
  const exited = new Promise<number>(function waitForExit(resolve) {
    child.once(
      'exit',
      function onExit(code) {
        resolve(code ?? (-1));
      },
    );
  });

  /**
   End the writer, once it has been seen holding the tree, and answer the code it ended with.

   The signal goes to the process this fixture started, named by the number the spawn answered with, and to nothing else:
   the two fields Node keeps of an ending, `exitCode` and `signalCode`, are what says there is still a process to signal
   (`ChildProcess`, `subprocess.exitCode` and `subprocess.signalCode` in Node's `child_process` documentation).

   @returns the code the writer ended with, which is the test's witness that its writes found the tree

   @example
   ```ts
   await writer.end();
   ```
   */
  async function end(): Promise<number> {
    if ((child.exitCode === null) && (child.signalCode === null))
      child.kill('SIGTERM');
    return await exited;
  }

  try {
    await waitUntilTheWriterHoldsTheTree({
      pid,
      root,
    });
  }
  catch (error) {
    // A writer that never held the tree is still this test's to end, and it is ended before the failure is raised.
    await end();
    throw error;
  }
  return {
    pid,
    end,
    async [Symbol.asyncDispose]() {
      await end();
    },
  };
}

/**
 Whether a promise answered within a bound, without letting its own answer escape as an unhandled rejection.

 @param promise - what to watch

 @param milliseconds - how long to watch, sized by the caller as several bounds of the code's own promise

 @returns true when it answered inside the bound, false when the bound passed with it unanswered

 @example
 ```ts
 await answeredWithin({ promise: removeScratchRoot({ root, }), milliseconds: 1000 });
 ```
 */
async function answeredWithin({
  promise,
  milliseconds,
}: {
  readonly promise: Promise<void>;
  readonly milliseconds: number;
}): Promise<boolean> {
  /**
   The promise as an answer rather than a rejection: a call that stopped waiting with a failure has answered too, and
   `allSettled` is what keeps that answer from escaping as an unhandled rejection.
   */
  const answered = Promise.allSettled([promise])
    .then(() => true);
  /**
   The bound itself.
   */
  const bound = pause(milliseconds)
    .then(() => false);
  return await Promise.race([
    answered,
    bound,
  ]);
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
         The writer, which holds the tree until this test ends it, so the wait cannot end because a program timed itself.
         */
        await using writer = await startDetachedWriter(root);
        /**
         The removal, asked while the writer holds the tree.
         */
        const removal = removeScratchRoot({
          root,
          deadlineMilliseconds: DEFAULT_HOLDER_DEADLINE_MILLISECONDS,
        });
        expect(await answeredWithin({
          promise: removal,
          milliseconds: unansweredWindowMilliseconds,
        })).toBe(false);
        /**
         The code the writer ended with: 0 when this test's own end reached it and every write it made found the tree.
         */
        const writerExitCode = await writer.end();
        expect(writerExitCode).toBe(0);
        await removal;
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
         The writer, which holds the tree from here until this test ends it, whatever the host does with this process's
         own thread.
         */
        await using writer = await startDetachedWriter(root);
        /**
         Failure raised at the short deadline, and nothing when the call answered.
         */
        let failure: unknown;
        try {
          await removeScratchRoot({
            root,
            deadlineMilliseconds: failingRemovalDeadlineMilliseconds,
          });
        }
        catch (error: unknown) {
          failure = error;
        }
        expect(failure).toBeInstanceOf(Error);
        expect((failure as Error).message).toContain(root);
        expect((failure as Error).message).toContain('holds cwd');
        expect((failure as Error).message).toContain(`${String(writer.pid)} (`);
        // What the failure left: the tree the writer still holds.
        expect(await access(root)).toBeUndefined();
        // This test ends the writer, and only then is the tree the test's to remove.
        expect(await writer.end()).toBe(0);
        await removeScratchRoot({
          root,
          deadlineMilliseconds: failingRemovalDeadlineMilliseconds,
        });
        await expect(access(root)).rejects.toThrow();
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
