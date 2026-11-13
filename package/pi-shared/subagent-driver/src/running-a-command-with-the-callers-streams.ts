/**
 Running one command with the caller's own streams, so what it prints reaches the terminal as it prints it: with a bound the caller
 gives, which ends the command and everything it started, or with none at all for a command that bounds itself.

 A launcher is the shape the unbounded one serves: its own run reports progress while it happens and ends on its own terms. Anything
 that may wait on a lock, a pipe or the network is run with a bound, because nothing here may wait forever.

 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import { setTimeout as wait, } from 'node:timers/promises';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the kernel gives a signal sent to a process group that is already gone.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 Milliseconds a command is given to close after its bound is reached and its process group killed, before the caller is answered
 anyway.

 A process this cannot end is one in a file system call that cannot be interrupted, and waiting for it is the wait the bound exists to
 prevent.
 */
const GIVEN_UP_CLOSE_MILLISECONDS = 5_000;

/**
 The answer of a wait whose bound passed before the command closed.
 */
const NOT_YET: unique symbol = Symbol('script still going after the time it was allowed');

/**
 What a command that showed its own output ended with.
 */
export type ShownCommandEnding = {
  /**
   The status it exited with, or nothing when a signal ended it.
   */
  readonly status?: number;

  /**
   Why it could not be started, when it could not be.
   */
  readonly startProblem?: string;

  /**
   Whether a bound the caller gave is what ended the command, and the command and everything it started were ended with it.
   */
  readonly givenUp?: true;
};

/**
 Runs one command with the caller's own streams, so what it prints reaches the terminal as it prints it, and answers with how it
 ended.

 This is the shape a launcher needs: a run's progress must show while it runs, and its status is what the caller answers with.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param folder - the folder it runs in, which is this process's own when it names none

 @param environment - the environment it gets, which is this process's own when it names none

 @returns how it ended, which never throws for a failure of the command itself

 @example
 ```ts
 const ended = await runCommandShowingItsOutput({ command: 'systemd-run', words: ['--user', '--wait', '--unit=pi-a', 'true',], },);
 ```
 */
export async function runCommandShowingItsOutput({
  command,
  words,
  folder,
  environment,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly folder?: string;
  readonly environment?: NodeJS.ProcessEnv;
},): Promise<ShownCommandEnding> {
  /**
   The command's process.
   */
  const child = spawn(
    command,
    [...words,],
    {
      stdio: 'inherit',
      ...(folder === undefined ? {} : { cwd: folder, }),
      ...(environment === undefined ? {} : { env: environment, }),
    },
  );
  try {
    /**
     What the process reported when it closed, which is its status or the signal that ended it.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    return endingOf({ closed, },);
  }
  catch (error) {
    // `once` rejects when the process could not be started at all, and that is the one failure a caller reports rather than reads
    // as an exit.
    return { startProblem: String(error,) };
  }
}

/**
 Runs one command with the caller's own streams and a bound, ends the command and everything it started when the bound is reached,
 and answers with how it ended.

 The command is started in a process group of its own, because a shell or a build tool starts children of its own: ending only the
 process the caller named would leave those children holding whatever the command was waiting on. The group is signalled at the bound,
 and the caller is answered with `givenUp` whether or not the process then closes.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param folder - the folder it runs in, which is this process's own when it names none

 @param timeoutMilliseconds - how long it may take before it and everything it started is ended

 @returns how it ended, which never throws for a failure of the command itself

 @example
 ```ts
 const ended = await runCommandShowingItsOutputWithin({ command: 'cargo', words: ['clean',], folder: '/worktree', timeoutMilliseconds: 300_000, },);
 ```
 */
export async function runCommandShowingItsOutputWithin({
  command,
  words,
  folder,
  timeoutMilliseconds,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly folder?: string;
  readonly timeoutMilliseconds: number;
},): Promise<ShownCommandEnding> {
  /**
   The command's process, in a process group of its own.
   */
  const child = spawn(
    command,
    [...words,],
    {
      stdio: 'inherit',
      // A group of its own is what lets the bound reach what the command started, and it is its own from the moment it starts.
      detached: true,
      ...(folder === undefined ? {} : { cwd: folder, }),
    },
  );
  /**
   The promise of the process closing, which both waits below are answered by.
   */
  const closed = once(
    child,
    'close',
  );
  try {
    /**
     What the process reported within its bound, or that the bound passed first.
     */
    const inTime = await raceWithTheClock({
      closed,
      milliseconds: timeoutMilliseconds,
    },);
    if (inTime !== NOT_YET) {
      return endingOf({ closed: inTime, },);
    }
    // The bound is reached: the whole group is ended, and what it left behind is not waited for beyond the second bound.
    endProcessGroup({ child, },);
    /**
     What the process reported once its group was ended, if it closed at all.
     */
    const afterwards = await raceWithTheClock({
      closed,
      milliseconds: GIVEN_UP_CLOSE_MILLISECONDS,
    },);
    return {
      givenUp: true,
      ...((afterwards === NOT_YET) ? {} : endingOf({ closed: afterwards, },)),
    };
  }
  catch (error) {
    // `once` rejects when the process could not be started at all, and that is the one failure a caller reports rather than reads
    // as an exit.
    return { startProblem: String(error,) };
  }
}

/**
 Reads what a command that showed its own output ended with, from what its process reported when it closed.

 @param closed - the arguments the process's `close` carried: its status, then the signal that ended it

 @returns the status it exited with, or nothing when a signal ended it

 @example
 ```ts
 const ending = endingOf({ closed: [0, null,], },);
 ```
 */
function endingOf({ closed, }: { readonly closed: readonly unknown[] },): ShownCommandEnding {
  /**
   The status the process exited with, or nothing when a signal ended it.
   */
  const [status,] = closed;
  return ((typeof status) === 'number') ? { status, } : {};
}

/**
 Answers that a bound has passed, once it has.

 @param milliseconds - how long to wait

 @returns the answer a read of a command still running at its bound gets

 @example
 ```ts
 const passed = passWhenTheBoundDoes({ milliseconds: 300_000, },);
 ```
 */
async function passWhenTheBoundDoes({ milliseconds, }: { readonly milliseconds: number },): Promise<typeof NOT_YET> {
  // A timer that is not ref'd holds nothing open: a race this loses must not keep the caller alive for the rest of its bound, which is
  // what made a sweep print its summary and then sit for seconds before it exited.
  await wait(
    Math.max(
      milliseconds,
      0,
    ),
    undefined,
    { ref: false, },
  );
  return NOT_YET;
}

/**
 Waits for one command to close, and answers that its bound passed first when it did not close in time.

 @param closed - the promise of the command closing

 @param milliseconds - how long it may take

 @returns what the close carried, or that the bound passed first

 @example
 ```ts
 const inTime = await raceWithTheClock({ closed, milliseconds: 300_000, },);
 ```
 */
async function raceWithTheClock({
  closed,
  milliseconds,
}: {
  readonly closed: Promise<readonly unknown[]>;
  readonly milliseconds: number;
},): Promise<readonly unknown[] | typeof NOT_YET> {
  /**
   The wait that ends when the bound does, whatever the command does.
   */
  const boundPassed = passWhenTheBoundDoes({ milliseconds, },);
  return await Promise.race([
    closed,
    boundPassed,
  ],);
}

/**
 Ends one command's process group.

 @param child - the command's process, started detached so that its group is its own

 @example
 ```ts
 endProcessGroup({ child, },);
 ```
 */
function endProcessGroup({ child, }: { readonly child: ChildProcess },): void {
  if (child.pid === undefined) {
    return;
  }
  try {
    // The negative number names the whole group: the command, and every child it started before it blocked.
    process.kill(
      -child.pid,
      'SIGKILL',
    );
  }
  catch (error) {
    // A group whose command already closed is nothing to end, which is the answer a caller reads as done rather than as a failure. A
    // failure of any other kind is one this cannot answer for, so it is carried out.
    if (errorCodeOf(error,) !== NO_SUCH_PROCESS_CODE) {
      throw error;
    }
  }
}
