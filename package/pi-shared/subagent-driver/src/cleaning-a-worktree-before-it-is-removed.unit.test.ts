/**
 The clean command a sweep runs inside one worktree before removing it: bounded, so a command that waits on a lock, a pipe or the
 network cannot hold the sweep, and said plainly in the line the sweep prints when it is given up.

 Every fixture here is a folder the test made under the scratch root, so no test reads or writes a real worktree.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  CLEAN_COMMAND_BOUND_SECONDS,
  cleanTheWorktreeIn,
  errorCodeOf,
} from '../dist/final/node/index.mjs';

/**
 The code the kernel gives a signal sent to a process that is not there, which is the answer this reads as "it has gone".
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 A folder for one test, removed when the test ends.

 @returns the folder's path, and the disposer that removes it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), 'pi-clean-of-a-worktree-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Says whether a process is still there.

 @param pid - the process number

 @returns whether the kernel still has that process

 @example
 ```ts
 const there = await processIsThere(4242);
 ```
 */
async function processIsThere(pid: number,): Promise<boolean> {
  try {
    process.kill(pid, 0,);
    return true;
  }
  catch (error) {
    // A process the kernel no longer has is exactly what this asks about; any other failure is one this cannot answer for.
    if (errorCodeOf(error,) !== NO_SUCH_PROCESS_CODE) {
      throw error;
    }
    return false;
  }
}

/**
 Waits between two looks at whether a process has gone.

 @param milliseconds - how long to wait

 @example
 ```ts
 await waitBetweenLooks(10);
 ```
 */
async function waitBetweenLooks(milliseconds: number,): Promise<void> {
  await new Promise<void>(function waitForIt(resolve,) {
    setTimeout(resolve, milliseconds,);
  },);
}

/**
 Waits, within a bound, until none of the processes named is a process any more.

 @param pids - the process numbers to wait for

 @param milliseconds - how long they may take to go

 @returns whether every one of them has gone

 @example
 ```ts
 const gone = await goneWithin({ pids: [4242,], milliseconds: 5_000, },);
 ```
 */
async function goneWithin({
  pids,
  milliseconds,
}: {
  readonly pids: readonly number[];
  readonly milliseconds: number;
},): Promise<boolean> {
  /**
   When this wait gives up on them.
   */
  const deadline = Date.now() + milliseconds;
  while (Date.now() < deadline) {
    /**
     The numbers that are still processes.
     */
    const alive: number[] = [];
    /* oxlint-disable eslint/no-await-in-loop -- one number at a time, each read before the next, so a failure names it; see
       doc/lint/sequential-awaits.md. */
    for (const pid of pids) {
      if ((pid > 0) && (await processIsThere(pid,))) {
        alive.push(pid,);
      }
    }
    /* oxlint-enable eslint/no-await-in-loop */
    if (alive.length === 0) {
      return true;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- one look, then another until the bound, so a killed process is not read as alive for ever
    await waitBetweenLooks(10,);
  }
  return false;
}

await describe({
  name: cleanTheWorktreeIn.name,
  children: [
    it({
      name: 'answers that there is no clean command when the caller named none, so the sweep says that rather than waiting for nothing',
      fn: async () => {
        /**
         The worktree a clean would run in, which the caller named none for.
         */
        await using folder = await makeFolder();
        expect(await cleanTheWorktreeIn({
          cleanWords: [],
          cleanFolder: '',
          worktreeFolder: folder.path,
        },),).toBe('no clean command',);
        expect(await cleanTheWorktreeIn({
          cleanWords: ['cargo', 'clean',],
          cleanFolder: '',
          worktreeFolder: folder.path,
        },),).toBe('no clean command',);
      },
    },),
    it({
      name: 'gives up a clean command that blocks at its bound, ends what it started, and says the sweep went on',
      fn: async () => {
        /**
         The worktree the clean runs in.
         */
        await using folder = await makeFolder();
        /**
         The folder inside the worktree the clean runs in.
         */
        const inside = join(
          folder.path,
          'a-build-folder',
        );
        await mkdir(inside, { recursive: true, },);
        /**
         Where the blocking command writes the numbers of itself and of its child.
         */
        const numbers = join(
          folder.path,
          'numbers.txt',
        );
        /**
         What the line said, which is the sweep's one line for this worktree.
         */
        const freed = await cleanTheWorktreeIn({
          cleanWords: [
            'sh',
            '-c',
            `sleep 30 & printf '%s %s\\n' "$$" "$!" > '${numbers}'; wait`,
          ],
          cleanFolder: 'a-build-folder',
          worktreeFolder: folder.path,
          boundMilliseconds: 300,
        },);
        expect(freed,).toContain('clean given up after',);
        expect(freed,).toContain('and the sweep went on',);
        /**
         The shell's number and its child's, as the shell wrote them before it blocked.
         */
        const [shell, child,] = (await readFile(
          numbers,
          'utf8',
        )).trim()
          .split(' ',)
          .map(Number,);
        // The group was killed, and the kernel reports a killed process as gone as soon as it is reaped: a machine under load takes a
        // moment to get there, so this waits for it within a bound rather than reading once.
        expect(await goneWithin({ pids: [shell ?? 0, child ?? 0,], milliseconds: 5_000, },),).toBe(true,);
      },
      timeout: 30_000,
    },),
    it({
      name: 'runs a clean command that ends inside its bound and says what it ended with',
      fn: async () => {
        /**
         The worktree the clean runs in.
         */
        await using folder = await makeFolder();
        await writeFile(join(
          folder.path,
          'freed.txt',
        ), 'x',);
        expect(await cleanTheWorktreeIn({
          cleanWords: ['sh', '-c', 'exit 3',],
          cleanFolder: '.',
          worktreeFolder: folder.path,
        },),).toBe('clean ended with status 3',);
      },
    },),
    it({
      name: 'says a clean that cannot be started at all, rather than reading it as a clean that failed',
      fn: async () => {
        /**
         The worktree the clean would run in.
         */
        await using folder = await makeFolder();
        expect(await cleanTheWorktreeIn({
          cleanWords: ['a-clean-command-that-is-not-here',],
          cleanFolder: '.',
          worktreeFolder: folder.path,
        },),).toContain('clean could not be started: ',);
      },
    },),
    it({
      name: 'bounds one clean at the seconds this package documents, which the sweep answers with the same figure',
      fn: async () => {
        expect(CLEAN_COMMAND_BOUND_SECONDS,).toBeGreaterThan(0,);
        expect(CLEAN_COMMAND_BOUND_SECONDS,).toBeLessThan(60,);
      },
    },),
  ],
},);
