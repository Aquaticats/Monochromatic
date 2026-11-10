/**
 Classification of one catalog-tighten run in the matrix, shared by the
 in-container entrypoint and its host-only unit tests.
 
 The entrypoint cannot be imported by a test: it reads a scenario from argv and
 runs a fixture install on import. Keeping the failure classification here keeps
 that logic testable, which matters because the difference between "the tool
 exited non-zero" and "the tool never started" decides whether a scenario may
 conclude anything at all.
 */

import {
  SubprocessError,
} from 'nano-spawn';

//region Types

/**
 Outcome of running the tool: whether it exited zero, and its combined output.
 */
export type ToolResult = {
  /**
   Whether the tool exited zero.
   */
  readonly ok: boolean;
  /**
   Combined stdout and stderr; the tighten line is on stdout, the MISS line on stderr.
   */
  readonly output: string;
};

/**
 Raised when the tool never produced an exit status, so the scenario learned
 nothing about catalog-tighten: the executable was missing or unrunnable, or a
 signal killed it. Reported as a scenario failure rather than as a tool
 failure, because an `expect: 'error'` scenario would otherwise pass without
 catalog-tighten ever reporting an error.
 */
export class ToolNotStartedError extends Error {
  /**
   @param message - why the tool produced no exit status, plus what it managed to emit
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'ToolNotStartedError';
  }
}

//endregion Types

//region Failure classification

/**
 Turns a rejection from `spawn` into a {@link ToolResult} when the tool ran to
 an exit status, and throws when it never got that far.
 
 Two failures look alike at the `spawn` call and mean opposite things: a tool
 that exited non-zero (its stdout and stderr are the evidence a scenario reports)
 and a tool that never started at all (nothing may be concluded). nano-spawn
 rejects with a `SubprocessError` in both cases and leaves `exitCode` undefined
 for the second, so the exit code is the discriminator.
 
 @param error - value the tool's `spawn` call rejected with
 
 @returns the tool's non-zero exit and its output, when it exited on its own
 
 @throws ToolNotStartedError when the tool produced no exit status
 
 @throws unknown the rejection itself, when it is not a subprocess failure at all
 
 @example
 ```ts
 toolResultFromSpawnError(error) // { ok: false, output: '...' }
 ```
 */
export function toolResultFromSpawnError(error: unknown,): ToolResult {
  if (!(error instanceof SubprocessError))
    throw error;

  if ((typeof error.exitCode) === 'number') {
    return {
      ok: false,
      output: `${error.stdout}\n${error.stderr}`,
    };
  }

  /**
   What ended the subprocess instead of an exit status of its own.
   */
  const ending = error.signalName === undefined
    ? 'the command never started'
    : `the command was killed by ${error.signalName}`;
  /**
   Why the tool has no exit status, and what it managed to emit before that.
   */
  const noExitStatus = [
    `catalog-tighten did not run to an exit status (${ending}),`,
    `so this scenario proves nothing about it: ${error.message}`,
    error.stdout,
    error.stderr,
  ].join(
    '\n',
  );
  throw new ToolNotStartedError(noExitStatus,);
}

//endregion Failure classification
