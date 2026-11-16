import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { executeHooks, } from '../dist/final/node/tunnel-util.mjs';

/**
 Longest a case waits for a program a hook left behind to reach its first write, in milliseconds.
 */
const BACKGROUND_WRITE_WAIT_MILLISECONDS = 3_000;

/**
 Process group of a program, as procfs holds it.

 @param pid - Program number to read.

 @returns The group number, or 0 when the program is gone.
 */
async function processGroupOf({ pid, }: { readonly pid: number; },): Promise<number> {
  try {
    /**
     Stat line of the process, whose group number follows the state letter.
     */
    const stat = await readFile(`/proc/${String(pid,)}/stat`, 'utf8',);
    /**
     Fields after the parenthesised name: state, ppid and the process group.
     */
    /**
     Fields after the parenthesised name, of which the third is the process group.
     */
    const fields = stat.slice(stat.lastIndexOf(')',) + 2,)
      .split(' ',);
    return Number(fields[2],);
  }
  catch (error: unknown) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT')
      return 0;
    throw error;
  }
}

/**
 Waits, bounded, for a file to appear.

 @param path - File to look for.

 @returns Whether it appeared within the bound.
 */
async function fileAppears({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   When the wait gives up.
   */
  const deadline = performance.now() + BACKGROUND_WRITE_WAIT_MILLISECONDS;
  while (performance.now() < deadline) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- The file is looked for again within one bound.
      await readFile(path,);
      return true;
    }
    catch (error: unknown) {
      // Not written yet; the bound above decides when to stop looking.
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT')
        throw error;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- The pause is part of the same bounded wait.
    await wait(50,);
  }
  return false;
}

/**
 Makes a scratch directory and binds its removal to the caller's scope.

 @returns The directory path.
 */
async function scratchDirectory(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   New directory under the harness scratch root.
   */
  const path = await mkdtemp(join(scratchRoot(), 'wg-hooks-',),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { force: true, recursive: true, },);
    },
  };
}

await describe({
  name: executeHooks.name,
  children: [
    it({
      name: 'runs a hook in the caller process group, so a terminal interrupt reaches the hook as it reaches the command',
      fn: async () => {
        await using directory = await scratchDirectory();
        /**
         File the hook writes its own process group into.
         */
        const groupFile = join(directory.path, 'hook-group.txt',);
        await executeHooks({
          hooks: [`ps -o pgid= -p $$ > '${groupFile}'`,],
          interfaceName: 'wghook',
        },);
        /**
         Group the hook ran in.
         */
        const hookGroup = Number((await readFile(groupFile, 'utf8',)).trim(),);
        // A terminal sends its interrupt to the foreground process group, so a hook outside it
        // never sees the Ctrl-C a person pressed for the command that runs the hook.
        expect(hookGroup,).toBe(await processGroupOf({ pid: process.pid, },));
      },
    },),
    it({
      name: 'leaves a program the hook started a standard output it can still write to after the call returned, instead of ending it with a broken pipe',
      fn: async () => {
        await using directory = await scratchDirectory();
        /**
         File the background program writes only if its write to the hook's standard output worked.
         */
        const marker = join(directory.path, 'wrote-after-return',);
        /**
         When the hook call started.
         */
        const started = performance.now();
        await executeHooks({
          // The program writes after the hook shell has exited and the call has returned.
          hooks: [`( sleep 0.5; echo late-log-line; touch '${marker}' ) &`,],
          interfaceName: 'wghook',
        },);
        /**
         How long the hook call took.
         */
        const elapsed = performance.now() - started;
        expect(elapsed,).toBeLessThan(1_000,);
        expect(await fileAppears({ path: marker, },),).toBe(true,);
      },
    },),
  ],
},);
