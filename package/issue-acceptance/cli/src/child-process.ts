/**
 The one place this command starts a child process.

 Every call passes an exact argument vector,
 supplies no inherited standard input,
 and ends at a fixed deadline,
 so a wedged `gh` or `git` cannot hold the session.

 @module
 */

import spawn, { SubprocessError, } from 'nano-spawn';

import { ChildProcessError, } from './errors.ts';

/**
 Fixed deadline for one child process, in milliseconds.

 Long enough for `gh issue list` over a repository with thousands of issues
 and for `git show` of a large document,
 short enough that a wedged child cannot hold a session.
 */
export const CHILD_DEADLINE_MS: number = 120_000;

/**
 Longest child standard error echoed in a failure message.
 */
const STDERR_ECHO_LIMIT = 400;

/**
 One finished child, including the failures whose exit status carries meaning.
 */
export type ChildOutcome = {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 One child process to run.
 */
export type ChildRequest = {
  readonly file: string;
  readonly arguments: readonly string[];
  readonly description: string;
  readonly cwd?: string;
  readonly env?: Readonly<Record<string, string>>;
  readonly deadlineMs?: number;
};

/**
 Runs one child and reports its exit status instead of throwing on it,
 because callers read deliberate failures (a grep that finds nothing,
 a `cat-file` for a path that is absent).
 
 @param file - Executable name or path.
 
 @param arguments - Exact argument vector, passed without a shell.
 
 @param description - How a failure names the work this child was doing.
 
 @param cwd - Working directory, or the current one when absent.
 
 @param env - Variables to add to the inherited environment.
 
 @param deadlineMs - How long the child may run.
 
 @returns Exit status and captured output.
 
 @throws {@link ChildProcessError} when the child never starts or ends at its deadline.
 
 @example
 ```ts
 const outcome = await runChild({ file: 'git', arguments: ['status'], description: 'git status' });
 ```
 */
export async function runChild({
  file,
  arguments: childArguments,
  description,
  cwd,
  env,
  deadlineMs = CHILD_DEADLINE_MS,
}: ChildRequest,): Promise<ChildOutcome> {
  try {
    /**
     Finished child, when it exited on its own.
     */
    const result = await spawn(
      file,
      [...childArguments,],
      {
        ...(cwd === undefined ? {} : { cwd, }),
        stdin: 'ignore',
        stdout: 'pipe',
        stderr: 'pipe',
        ...(env === undefined ? {} : { env, }),
        signal: AbortSignal.timeout(deadlineMs,),
        killSignal: 'SIGKILL',
        windowsHide: true,
      },
    );
    return {
      status: 0,
      stdout: result.stdout,
      stderr: result.stderr,
    };
  }
  catch (error) {
    if (!(error instanceof SubprocessError)) {
      throw new ChildProcessError({
        message: `${description} could not run ${file}: ${String(error,)}`,
        stderr: '',
      },);
    }
    if ((!error.isCanceled) && (error.exitCode !== undefined)) {
      return {
        status: error.exitCode,
        stdout: error.stdout,
        stderr: error.stderr,
      };
    }
    throw new ChildProcessError({
      message: `${description} did not finish within ${String(deadlineMs,)} ms`,
      stderr: error.stderr
        .slice(
        0,
        STDERR_ECHO_LIMIT,
        ),
    },);
  }
}
