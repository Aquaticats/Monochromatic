/**
 One Git command, run without any way to reach the desktop.

 A Git or `git-lfs` command that finds no credential falls back to `GIT_ASKPASS`,
 `core.askPass` and `SSH_ASKPASS` before it looks at `GIT_TERMINAL_PROMPT`, so an
 operator's machine can open a password dialog the command that started it cannot
 close. Every command here therefore runs with `GIT_ASKPASS` set but empty and
 `GIT_TERMINAL_PROMPT` at `0`, and a helper that would prompt fails instead.

 @module
 */

import spawn, { SubprocessError, } from 'nano-spawn';

/**
 A Git command that exited non-zero.

 The message carries the command and its standard error, never anything handed to
 the command on standard input, because a token travels that way.
 */
export class GitCommandError extends Error {
  /**
   Exit status the command reported, absent when a signal ended it or it never ran.
   */
  readonly status?: number;

  /**
   @param message - command, exit status and standard error, with no standard input

   @param status - exit status the command reported
   */
  constructor(
    message: string,
    status?: number,
  ) {
    super(message,);
    this.name = 'GitCommandError';
    if (status !== undefined) {
      this.status = status;
    }
  }
}

/**
 Parameters for {@link runGit}.
 */
export type RunGitParams = {
  /**
   Arguments after the program name.
   */
  readonly args: readonly string[];
  /**
   Directory the command runs in, so Git reads that directory's configuration.
   */
  readonly cwd: string;
  /**
   Text handed to standard input; absent when the command reads none.
   */
  readonly input?: string;
};

/**
 Run one Git command and answer its standard output.

 @param args - arguments after the program name

 @param cwd - directory the command runs in

 @param input - text handed to standard input

 @returns the command's standard output, with its trailing newline removed

 @throws {@link GitCommandError} when the command exits non-zero, naming its standard error

 @example
 ```ts
 await runGit({ args: ['rev-parse', '--show-toplevel'], cwd: repository });
 ```
 */
export async function runGit({
  args,
  cwd,
  input,
}: RunGitParams,): Promise<string> {
  try {
    return (await spawn(
      'git',
      [...args,],
      {
        cwd,
        env: {
          GIT_ASKPASS: '',
          GIT_TERMINAL_PROMPT: '0',
        },
        ...input === undefined ? {} : { stdin: { string: input, }, },
      },
    )).stdout;
  }
  catch (error) {
    throw new GitCommandError(
      describeFailure({
        args,
        error,
      },),
      error instanceof SubprocessError ? error.exitCode : undefined,
    );
  }
}

/**
 Parameters for {@link describeFailure}.
 */
type DescribeFailureParams = {
  /**
   Arguments the command ran with.
   */
  readonly args: readonly string[];
  /**
   Value the spawn call threw.
   */
  readonly error: unknown;
};

/**
 Spell a failed command and its standard error on one line.

 @param args - arguments the command ran with

 @param error - value the spawn call threw

 @returns message naming the command and what Git said
 */
function describeFailure({
  args,
  error,
}: DescribeFailureParams,): string {
  /**
   Standard error the command wrote, when the failure carried any.
   */
  const standardError = ((typeof error) === 'object')
    && (error !== null)
    && ('stderr' in error)
    && ((typeof error.stderr) === 'string')
    ? error.stderr
      .trim()
    : String(error,);
  return `git ${args.join(' ',)} failed: ${standardError}`;
}
