/**
 Runs a command whose output belongs to the person watching, and hands back how it ended.

 A unit started with `--wait` prints as it goes, so this one is not the seam that captures output: it lets the command write to
 this process's own streams and keeps only the status. A test hands the start a stand-in instead of a real `systemd-run`.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core';
import type { CommandOutput, } from './run-command.ts';

/**
 Runs one command to its end, with its output going where this process's output goes.
 */
export type RunToTheEnd = (request: {
  readonly command: string;
  readonly words: readonly string[];
}) => Promise<CommandOutput>;

/**
 Reads why the operating system refused to start the command.

 @param caught - what was caught

 @returns the reason, in the words the operating system gave

 @example
 ```ts
 const problem = problemOf({ code: 'ENOENT', },);
 ```
 */
function problemOf(caught: unknown,): string {
  return caughtValueText(caught,);
}

/**
 Runs one command to its end, letting it write to this process's own output.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @returns the status it exited with, and why it never did when it never did

 @example
 ```ts
 const ending = await runToTheEnd({ command: 'systemd-run', words: ['--user', '--wait',], });
 ```
 */
export async function runToTheEnd({
  command,
  words,
}: {
  readonly command: string;
  readonly words: readonly string[];
},): Promise<CommandOutput> {
  /**
   The command itself.
   */
  const child = spawn(
    command,
    [...words,],
    { stdio: 'inherit', },
  );
  try {
    /**
     The status the command exited with, which is null when a signal ended it.
     */
    const [code,] = await once(
      child,
      'exit',
    ) as [unknown];
    return {
      status: ((typeof code) === 'number') ? code : -1,
      stdout: '',
      stderr: '',
    };
  }
  catch (error) {
    // A program that is not on this machine never runs, and its own reason says which program it looked for.
    return {
      status: -1,
      stdout: '',
      stderr: '',
      problem: problemOf(error,),
    };
  }
}
