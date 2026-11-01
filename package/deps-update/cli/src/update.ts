/**
 Runs the real `pnpm update --recursive --no-save`, showing pnpm's output
 live while keeping a copy of stderr to classify a failure.

 @module
 */

import spawn, { SubprocessError, } from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for the update run.
 */
const l = tagged({ tag: 'deps-update/update', },);

//region Types

/**
 Update outcome: success, or failure with pnpm's stderr for classification.
 */
export type UpdateOutcome =
  | {
    /**
     pnpm exited 0.
     */
    readonly ok: true;
  }
    | {
    /**
     pnpm exited nonzero or was killed.
     */
      readonly ok: false;
    /**
     pnpm's standard error, line-joined.
     */
      readonly stderr: string;
    /**
     Exit code; omitted when a signal ended pnpm or it never started.
     */
      readonly exitCode?: number;
    };

//endregion Types

//region Constants

/**
 Arguments of the update this task wraps.
 */
export const UPDATE_ARGS = [
  'update',
  '--recursive',
  '--no-save',
] as const;

/**
 pnpm diagnostic code for the strict-gate refusal under `--no-save`.
 */
export const STRICT_REQUIRES_SAVE_CODE = 'ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE';

//endregion Constants

//region Run

/**
 Runs the update with inherited stdin/stdout and tee'd stderr.

 @param cwd - workspace root

 @param command - executable to run; tests pass a fake pnpm

 @param commandArgs - leading arguments before {@link UPDATE_ARGS}; tests use it to run a script through `node`

 @returns success, or failure with captured stderr

 @throws Error when stderr iteration fails for a reason other than pnpm's exit

 @example
 ```ts
 const outcome = await runUpdate({ cwd: '/repo', command: 'pnpm', commandArgs: [] });
 ```
 */
export async function runUpdate({
  cwd,
  command,
  commandArgs,
}: {
  readonly cwd: string;
  readonly command: string;
  readonly commandArgs: readonly string[];
},): Promise<UpdateOutcome> {
  /**
   Logger tagged with this function.
   */
  const rl = tagged({
    tag: runUpdate.name,
    l,
  },);
  rl.info(`${command} ${[
    ...commandArgs,
    ...UPDATE_ARGS,
  ].join(' ',)} (cwd ${cwd})`,);
  /**
   Live subprocess; stderr stays piped so it can be both shown and kept.
   */
  const subprocess = spawn(
    command,
    [
      ...commandArgs,
      ...UPDATE_ARGS,
    ],
    {
      cwd,
      stdin: 'inherit',
      stdout: 'inherit',
    },
  );
  // Streaming lines arrive one at a time; an accumulator is the only way to
  // keep them while forwarding each immediately.
  /**
   Captured stderr lines.
   */
  const lines: string[] = [];
  try {
    for await (const line of subprocess.stderr) {
      // Raw console: this is pnpm's own output, forwarded verbatim.
      console.error(line,);
      lines.push(line,);
    }
    rl.info('pnpm update succeeded',);
    return { ok: true, };
  }
  catch (error) {
    if (!(error instanceof SubprocessError))
      throw error;
    rl.info(`pnpm update failed (exit ${String(error.exitCode,)})`,);
    return {
      ok: false,
      stderr: lines.join('\n',),
      ...(error.exitCode === undefined ? {} : { exitCode: error.exitCode, }),
    };
  }
}

//endregion Run
