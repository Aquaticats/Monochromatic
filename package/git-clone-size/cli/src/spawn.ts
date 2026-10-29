import nanoSpawn, { SubprocessError, } from 'nano-spawn';

import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

/**
 Result of a process invocation that did not throw operationally.
 `exitCode` 0 means success; non-zero means the process ran but failed,
 which probes tolerate (folding into a wider range) rather than crash on.
 */
export type SpawnResult = {
  readonly stdout: string;
  readonly stderr: string;
  readonly exitCode: number;
};

/**
 Fallback exit code when nano-spawn reports a failure without a numeric code
 (for example a missing executable).
 */
const FAILED_EXIT_CODE = 1;

/**
 Spawns a command and returns its captured streams plus exit code,
 NEVER throwing on a non-zero exit. Use this for probes that must tolerate
 a failed git/gh/glab invocation (unsupported filter, missing API, detached
 HEAD) by degrading the estimate rather than aborting.
 
 @param command - command name or path to execute
 
 @param args - argument vector for the command
 
 @param cwd - working directory for the child process
 
 @param stdin - optional string fed to the child's standard input
 
 @param env - optional environment overrides merged over the parent env
 
 @param signal - optional abort signal; aborting kills the child (budget kill)
 
 @returns captured stdout (trimmed), stderr (trimmed), and numeric exit code
 
 @example
 ```ts
 const { stdout, exitCode } = await spawnResult({ command: 'git', args: ['rev-list', '--count', 'HEAD'] });
 ```
 */
export async function spawnResult(
  {
    command,
    args,
    cwd,
    stdin,
    env,
    signal,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly cwd?: string;
    readonly stdin?: string;
    readonly env?: Readonly<Record<string, string>>;
    readonly signal?: AbortSignal;
  },
): Promise<SpawnResult> {
  /**
   Tagged logger so the debug line names the spawn call site.
   */
  const rl = tagged({
    tag: spawnResult.name,
    l: logger,
  },);
  rl.debug(`${command} ${args.join(' ',)}${cwd === undefined ? '' : ` (cwd=${cwd})`}`,);

  try {
    /**
     Subprocess result on success; stdout/stderr captured as strings.
     */
    const result = await nanoSpawn(
      command,
      [...args,],
      {
        ...cwd === undefined ? {} : { cwd, },
        ...stdin === undefined ? {} : { stdin: { string: stdin, }, },
        ...env === undefined ? {} : { env, },
        ...signal === undefined ? {} : { signal, },
      },
    );
    return {
      exitCode: 0,
      stderr: result.stderr
        .trim(),
      stdout: result.stdout
        .trim(),
    };
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError) {
      /**
       Exit code from the failed subprocess, defaulted when git/gh reports none.
       */
      const exitCode = error.exitCode ?? FAILED_EXIT_CODE;
      /**
       Diagnostic text: subprocess stderr, falling back to the error message.
       */
      const stderr = (error.stderr === '' ? error.message : error.stderr).trim();
      rl.debug(`exit ${String(exitCode,)}: ${stderr}`,);
      return {
        exitCode,
        stderr,
        stdout: error.stdout
          .trim(),
      };
    }
    /**
     Abort or other non-subprocess failure: report as a failed result so the
     probe degrades to a wider range rather than crashing.
     */
    const message = caughtValueText(error,);
    rl.debug(`aborted/failed: ${message}`,);
    return {
      exitCode: FAILED_EXIT_CODE,
      stderr: message,
      stdout: '',
    };
  }
}
