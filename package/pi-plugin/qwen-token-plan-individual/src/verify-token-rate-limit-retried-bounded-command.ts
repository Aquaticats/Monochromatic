/**
 Commands the unchanged-bundle builder runs, each under a bound of its own.

 A command that blocks holds nothing here:
 when its bound passes it is ended,
 and the builder fails with the command,
 its folder and the bound named.

 The bound ends the command with `SIGKILL` rather than `SIGTERM`,
 because `execFile` sends its kill signal once and then waits for the exit:
 a command that traps `SIGTERM` leaves the promise unsettled.
 Measured on this machine on 2026-11-14:
 with the default signal a trap-and-sleep command was still unsettled 2.5 s
 after a 300 ms bound, and the same command with `SIGKILL` was ended at 304 ms.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  promisify,
} from 'node:util';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Constants

/**
 Bound one builder command gets.

 Building the unchanged bundle takes about a second on this machine,
 and a bound this far above that fails the proof long before a person stops
 waiting.
 */
export const BUILDER_COMMAND_TIMEOUT_MILLISECONDS = 60_000;

/**
 Signal a command that passes its bound is ended with,
 which no command can trap.
 */
const BOUND_SIGNAL: NodeJS.Signals = 'SIGKILL';

/**
 Node's own promisify adapter, typed through its documented namespace:
 the generic `promisify(execFile)` form trips `strict-void-return`
 (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ =
  promisify;

/**
 Command runner every bounded command goes through,
 which Node's adapter turns into a promise of the command's output.
 */
const RUN_COMMAND = promisifyExecFile(execFile,);

//endregion Constants

//region Types

/**
 One command to run, and the bound it gets.
 */
export type BoundedCommandRequest = {
  /**
   Program to run, by name on `PATH` or by path.
   */
  readonly file: string;
  /**
   Arguments it is given.
   */
  readonly args: readonly string[];
  /**
   Folder it runs in, absent when the caller's own folder is right.
   */
  readonly cwd?: string;
  /**
   Bound this command gets, absent when the builder's own bound applies.
   */
  readonly boundMilliseconds?: number;
};

/**
 What a command that finished within its bound left behind.
 */
export type BoundedCommandResult = {
  /**
   Everything it wrote to standard output.
   */
  readonly stdout: string;
  /**
   Everything it wrote to standard error.
   */
  readonly stderr: string;
};

//endregion Types

//region Running a bounded command

/**
 Whether a command's failure is the bound ending it.

 @param error - value the runner caught

 @returns `true` when the runner killed the command at its bound

 @example
 ```typescript
 if (wasKilledAtItsBound({ error })) {
   // the command ran out of time rather than failing on its own
 }
 ```
 */
function wasKilledAtItsBound({
  error,
}: {
  readonly error: unknown;
},): boolean {
  return Error.isError(error,)
    && ('killed' in error)
    && (error.killed === true);
}

/**
 The error one failed command is reported as,
 naming the command, the folder it ran in and the bound it had.

 @param request - the command that failed

 @param error - value the runner caught

 @returns an error whose message a reader can act on without reading code

 @example
 ```typescript
 throw commandFailure({ request, error });
 ```
 */
function commandFailure({
  request,
  error,
}: {
  readonly request: BoundedCommandRequest;
  readonly error: unknown;
},): Error {
  /**
   Bound the command had, which the message states either way.
   */
  const boundMilliseconds = request.boundMilliseconds
    ?? BUILDER_COMMAND_TIMEOUT_MILLISECONDS;
  /**
   Folder the command ran in, as the message names it.
   */
  const folder = request.cwd ?? process.cwd();
  /**
   The command itself, as a reader would type it.
   */
  const command = [
    request.file,
    ...request.args,
  ].join(' ',);
  if (wasKilledAtItsBound({ error, },)) {
    return new Error(
      `${command} did not finish within ${String(boundMilliseconds,)} ms in ${folder}, so it was ended with ${BOUND_SIGNAL}`,
      { cause: error, },
    );
  }
  return new Error(
    `${command} failed in ${folder}: ${caughtValueText(error,)}`,
    { cause: error, },
  );
}

/**
 Run one command under a bound, and fail with the command named when it does not
 finish.

 @param request - the command, its arguments, its folder and its bound

 @returns what it wrote to both output streams

 @throws naming the command, its folder and its bound when it passes the bound,
 and naming the command and its failure text otherwise

 @example
 ```typescript
 const archived = await runBoundedCommand({
   file: 'git',
   args: ['archive', commitPath],
   cwd: repository,
 });
 ```
 */
export async function runBoundedCommand(
  request: BoundedCommandRequest,
): Promise<BoundedCommandResult> {
  /**
   Fields of the request this run reads.
   */
  const {
    file,
    args,
    cwd,
    boundMilliseconds = BUILDER_COMMAND_TIMEOUT_MILLISECONDS,
  } = request;
  /**
   Folder the command runs in,
   which the caller's own folder stands in for exactly as it would have.
   */
  const folder = cwd ?? process.cwd();
  try {
    /**
     What the command wrote, which the adapter reads as text.
     */
    const result = await RUN_COMMAND(
      file,
      [
        ...args,
      ],
      {
        cwd: folder,
        timeout: boundMilliseconds,
        killSignal: BOUND_SIGNAL,
      },
    );
    return {
      stdout: result.stdout,
      stderr: result.stderr,
    };
  } catch (error) {
    throw commandFailure({
      request: {
        ...request,
        cwd: folder,
        boundMilliseconds,
      },
      error,
    },);
  }
}

//endregion Running a bounded command
