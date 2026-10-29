/**
 Captured-output pnpm invocations used by the diagnosis.

 The runner is a parameter everywhere it is used so tests substitute a fake
 that simulates pnpm's file writes without network access.

 @module
 */

import spawn, { SubprocessError, } from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for pnpm invocations.
 */
const l = tagged({ tag: 'deps-update/pnpm', },);

//region Types

/**
 Runs pnpm with arguments in a directory and returns its stdout.
 */
export type RunPnpm = (options: {
  readonly args: readonly string[];
  readonly cwd: string;
},) => Promise<string>;

//endregion Types

//region Errors

/**
 Thrown when a captured pnpm invocation exits unsuccessfully.
 */
export class PnpmCommandError extends Error {
  /**
   pnpm's stdout and stderr interleaved; pnpm prints `ERR_PNPM_*` diagnostics on either stream.
   */
  readonly output: string;

  /**
   Builds the error with the command line and pnpm's output.

   @param args - pnpm arguments that failed

   @param cwd - directory pnpm ran in

   @param output - pnpm's interleaved stdout and stderr

   @param cause - underlying subprocess error

   @example
   ```ts
   throw new PnpmCommandError({ args: ['why', 'a'], cwd: '/repo', output: '', cause });
   ```
   */
  constructor({
    args,
    cwd,
    output,
    cause,
  }: {
    readonly args: readonly string[];
    readonly cwd: string;
    readonly output: string;
    readonly cause: unknown;
  },) {
    super(
      `pnpm ${args.join(' ',)} failed in ${cwd}:\n${output}`,
      { cause, },
    );
    this.name = 'PnpmCommandError';
    this.output = output;
  }
}

//endregion Errors

//region Runner

/**
 Default {@link RunPnpm}: spawns the `pnpm` on `PATH` with piped output.

 @param args - pnpm arguments

 @param cwd - working directory

 @returns pnpm's stdout

 @throws PnpmCommandError when pnpm exits nonzero or cannot start

 @example
 ```ts
 await runPnpm({ args: ['config', 'get', 'registry'], cwd: process.cwd() });
 ```
 */
export async function runPnpm({
  args,
  cwd,
}: {
  readonly args: readonly string[];
  readonly cwd: string;
},): Promise<string> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: runPnpm.name,
    l,
  },);
  rl.debug(`pnpm ${args.join(' ',)} (cwd ${cwd})`,);
  try {
    /**
     Completed subprocess result.
     */
    const result = await spawn(
      'pnpm',
      [...args,],
      { cwd, },
    );
    return result.stdout;
  }
  catch (error) {
    if (!(error instanceof SubprocessError))
      throw error;
    rl.debug(`pnpm exited ${String(error.exitCode,)}`,);
    throw new PnpmCommandError({
      args,
      cwd,
      output: error.output,
      cause: error,
    },);
  }
}

//endregion Runner
