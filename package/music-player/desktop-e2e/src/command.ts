/**
 Running external programs without blocking, with a result that carries the exit status instead of throwing.
 
 @example
 ```ts
 import { runCommand, } from './command.ts';
 
 const { status, stdout, } = await runCommand({ command: 'pw-dump', args: [], },);
 ```
 
 @packageDocumentation
 */

import nanoSpawn, { SubprocessError, } from 'nano-spawn';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

/**
 Exit status reported when the program could not be started, was killed, or ran out of time.
 */
export const NO_EXIT_STATUS = -1;

/**
 What a program printed and how it ended. Each stream is without its final newline.
 */
export type CommandResult = {
  readonly status: number;
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Runs a program to the end and reports how it ended. A non-zero exit, a missing program and a timeout are all
 results, never exceptions: the callers decide what each means.
 
 @param command - program name or path
 
 @param args - argument vector
 
 @param cwd - directory the program runs in; the current directory when absent
 
 @param env - variables set on top of the current environment
 
 @param timeoutMs - milliseconds after which the program is killed; no limit when absent
 
 @returns exit status and the text printed on each stream
 
 @example
 ```ts
 const { status, } = await runCommand({ command: 'pgrep', args: ['--exact', 'music-player',], },);
 ```
 */
export async function runCommand(
  {
    command,
    args,
    cwd,
    env,
    timeoutMs,
  }: {
    readonly args: readonly string[];
    readonly command: string;
    readonly cwd?: string;
    readonly env?: Readonly<Record<string, string>>;
    readonly timeoutMs?: number;
  },
): Promise<CommandResult> {
  /**
   Tagged logger naming the command runner.
   */
  const rl = tagged({
    tag: runCommand.name,
    l: logger,
  },);
  rl.debug(`${command} ${args.join(' ',)}`,);
  try {
    /**
     Output of a program that exited with status zero.
     */
    const {
      stdout,
      stderr,
    } = await nanoSpawn(
      command,
      [...args,],
      {
        ...cwd === undefined ? {} : { cwd, },
        ...env === undefined ? {} : { env, },
        ...timeoutMs === undefined ? {} : { timeout: timeoutMs, },
      },
    );
    return {
      status: 0,
      stderr,
      stdout,
    };
  }
  catch (error: unknown) {
    rl.debug(`${command} did not exit with status zero: ${caughtValueText(error,)}`,);
    if (error instanceof SubprocessError)
      return {
        status: error.exitCode ?? NO_EXIT_STATUS,
        stderr: error.stderr === '' ? error.message : error.stderr,
        stdout: error.stdout,
      };
    return {
      status: NO_EXIT_STATUS,
      stderr: caughtValueText(error,),
      stdout: '',
    };
  }
}
