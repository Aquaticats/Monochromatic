/**
 How the 2026-11-21 host measurement runs a command and walks a list of them.

 A measurement that runs two adds at once measures this host's contention rather than the arms, so every list here is
 walked one item at a time; and a failed command has to name itself and what it printed, or a phase that ends early says
 nothing about why.
 */
import {
  access,
} from 'node:fs/promises';

import { isMissingPathError, } from '../bin/native-gate-paths.ts';
import { runCommand, } from '../bin/native-verification-process.ts';

/**
 How much of a failed command's standard error the thrown message carries.
 */
export const stderrTailInFailureMessage = 800;

/**
 Whether a path is there.

 @param path - the path to look for

 @returns true when the path can be reached

 @example
 ```ts
 await isThere({ path: '/home/user/temp/agent' }); // true
 ```
 */
export async function isThere({ path, }: { readonly path: string; }): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (isMissingPathError({ error, },)) return false;
    throw error;
  }
}

/**
 Runs one command and answers what it printed, failing loudly unless the caller reads the status itself.

 @param args - the program and its arguments, each passed as one argument with no shell interpretation

 @param cwd - the directory to run it in, which defaults to this process's

 @param env - the environment to run it with, which defaults to this process's

 @returns the command's standard output

 @throws Error when the command ended nonzero, naming what it printed

 @example
 ```ts
 await run({ args: ['git', '--version'] });
 ```
 */
export async function run({
  args,
  cwd,
  env,
}: {
  readonly args: readonly string[];
  readonly cwd?: string;
  readonly env?: NodeJS.ProcessEnv;
}): Promise<string> {
  /**
   The program to run, which the arguments' first entry always is.
   */
  const [command] = args;
  if (command === undefined) throw new Error('a command needs a program to run');
  /**
   What the command answered.
   */
  const outcome = await runCommand({
    command,
    args: args.slice(1,),
    capture: true,
    allowFailure: true,
    ...(cwd === undefined ? {} : { cwd, }),
    ...(env === undefined ? {} : { env, }),
  },);
  if ((outcome.status ?? 1) !== 0) {
    throw new Error(`${args.join(' ',)} in ${cwd ?? process.cwd()} failed: ${outcome.stderr
      .slice(-stderrTailInFailureMessage,)}`);
  }
  return outcome.stdout;
}

/**
 Runs one step for each item of a list, one item at a time.

 Why:
  Two adds at once would share this host's page cache, disk queue and CPU, and every phase here compares arms on the time
  their adds took, so an add waits for the one before it. The lint refuses an `await` inside a loop, so the list is walked
  by recursion: one item, then the rest. Every list here is a phase's own adds, whose length the run names.

 @param items - the items left to handle

 @param step - what to do with one item

 @typeParam Item - what one item of the list is

 @example
 ```ts
 await oneAtATime({ items: [1, 2], step: async function handle(item) { console.log(item); } });
 ```
 */
export async function oneAtATime<Item>({
  items,
  step,
}: {
  readonly items: readonly Item[];
  readonly step: (item: Item) => Promise<void>;
}): Promise<void> {
  /**
   The item to handle now, and the items after it.
   */
  const [next, ...rest] = items;
  if (next === undefined) return;
  await step(next,);
  return oneAtATime({
    items: rest,
    step,
  },);
}
