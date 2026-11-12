/**
 How this package asks git a question: which folder, never a prompt, and within a bound.

 A git that would ask for a password does not wait for one: a launcher runs unattended, and a git waiting on a terminal nobody is
 at is a run that never starts.

 @module
 */

import type { RunCommand, } from './run-command.ts';
import type { GitRunner, } from './worktree-of-a-run.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 Variables that turn every prompt git could ask into a failure, which a caller reads instead of waiting for a key nobody will
 press.
 */
export const GIT_ENVIRONMENT: Readonly<Record<string, string>> = {
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 Seconds one git question may take. A worktree add on this machine answered in seconds, and a status on the whole repository
 takes less than a minute; two minutes is the bound that ends a git waiting on an index lock or a filesystem that stopped
 answering, and it is a bound rather than a measurement.
 */
export const GIT_TIMEOUT_SECONDS = 120;

/**
 Builds the git runner from the seam that runs a command.

 @param run - how a command is run, which a test replaces

 @returns how one git question is asked

 @example
 ```ts
 const git = gitRunnerOf(runCommandWithChildProcess);
 ```
 */
export function gitRunnerOf(run: RunCommand,): GitRunner {
  return function gitIn({
    words,
    folder,
    timeoutMilliseconds = GIT_TIMEOUT_SECONDS * MILLISECONDS_PER_SECOND,
  }) {
    return run({
      command: 'git',
      words,
      folder,
      environment: GIT_ENVIRONMENT,
      timeoutMilliseconds,
    },);
  };
}
