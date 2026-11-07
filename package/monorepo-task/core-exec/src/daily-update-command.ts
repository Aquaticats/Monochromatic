/**
 Runs one command of the daily update with a time bound: its output goes to the journal, its tail is kept, and the whole
 process group is stopped when the bound passes.

 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';

import type { CommandOutcome, } from './daily-update-run.ts';
import type { UpdateCommand, } from './daily-update-plan.ts';

/**
 How much of a command's output is kept for the notification, in characters.
 A notification shows a few lines; the journal has the full output.
 */
export const OUTPUT_TAIL_CHARACTERS = 1_500;

/**
 Milliseconds a stopped command gets to leave after the first signal before its process group is killed.
 A hung package manager answers a termination signal within seconds when it answers at all.
 */
export const KILL_GRACE_MS = 10_000;

/**
 Exit status reported for a command that could not be started, the shell convention for "command not found".
 */
export const START_FAILURE_EXIT_CODE = 127;

/**
 Exit status reported for a command that ended on a signal without an exit status of its own.
 */
export const SIGNAL_EXIT_CODE = 128;

/**
 A pending stop of a process group, cancelled when the scope ends.
 */
export type GroupStopper = {
  /**
   Whether the stop fired before the scope ended.
   */
  readonly fired: () => boolean;
  /**
   Cancels the pending stop.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Schedules the stop of a process group: a termination signal at the time bound, a kill after the grace period.

 @param pid - process id of the group leader, which is also the group id because the command was started detached

 @param timeoutMs - time bound in milliseconds

 @returns the pending stop, to be held with `using`

 @example
 ```ts
 using stopper = stopGroupAfter({ pid: child.pid, timeoutMs: 1000, },);
 ```
 */
export function stopGroupAfter({
  pid,
  timeoutMs,
}: {
  readonly pid: number;
  readonly timeoutMs: number;
},): GroupStopper {
  /**
   Whether the time bound passed.
   */
  const state = { fired: false, };
  // A process that never started has no group to stop; signalling group 0 would reach this process's own group.
  /**
   Process id to signal, zero when the process never started.
   */
  const startedPid = Math.max(
    pid,
    0,
  );
  /**
   The kill that follows the termination signal, absent until the bound passes.
   */
  const pending: NodeJS.Timeout[] = [];
  /**
   The termination signal at the time bound.
   */
  const stop = startedPid === 0 ? setTimeout(
    function nothingToStop() { /* no process */ },
    0,
  ) : setTimeout(
    function terminateGroup() {
      state.fired = true;
      process.kill(
        -startedPid,
        'SIGTERM',
      );
      pending.push(setTimeout(
        function killGroup() {
          try {
            process.kill(
              -startedPid,
              'SIGKILL',
            );
          }
          catch (error) {
        // The group left after the termination signal, which is what the signal was for.
            process.stderr
              .write(`daily-update: the group ${pid} was already gone: ${String(error,)}\n`,);
          }
        },
        KILL_GRACE_MS,
      ),);
    },
    timeoutMs,
  );
  return {
    fired: function wasFired() {
      return state.fired;
    },
    [Symbol.dispose]: function cancel() {
      clearTimeout(stop,);
      for (const timer of pending) {
        clearTimeout(timer,);
      }
    },
  };
}

/**
 How a child process ended.
 */
export type ChildEnd = {
  /**
   Exit status, or `127` when the process could not be started, or `128` when a signal ended it without a status.
   */
  readonly exitCode: number;
};

/**
 Bridges a child process's events to one promise that settles once, when the child closes or fails to start.

 @param child - the child process

 @returns how it ended

 @example
 ```ts
 const end = await endOf(spawn('node', ['--version',],),);
 ```
 */
export async function endOf(child: ChildProcess,): Promise<ChildEnd> {
  try {
    // `once` rejects when the child emits `error` (it could not be started) before it closes.
    await once(
      child,
      'close',
    );
  }
  catch (error) {
    process.stderr
      .write(`daily-update: the command could not be started: ${String(error,)}\n`,);
    return { exitCode: START_FAILURE_EXIT_CODE, };
  }
  return { exitCode: child.exitCode ?? SIGNAL_EXIT_CODE, };
}

/**
 Copies a child's output to the journal while keeping the tail of it.

 @param child - the child process, started with piped output

 @param tail - where the bounded tail is kept

 @example
 ```ts
 copyOutput({ child, tail: { text: '', }, },);
 ```
 */
export function copyOutput({
  child,
  tail,
}: {
  readonly child: ChildProcess;
  readonly tail: { text: string; };
},): void {
  for (const stream of [
    child.stdout,
    child.stderr,
  ]) {
    stream?.on(
      'data',
      function keepChunk(chunk: Buffer,) {
        process.stderr
          .write(chunk,);
        tail.text = `${tail.text}${chunk.toString('utf8',)}`.slice(-OUTPUT_TAIL_CHARACTERS,);
      },
    );
  }
}

/**
 Runs one command in the repository, bounded in time.

 @param repository - absolute repository path, the working directory

 @param command - the command

 @returns the outcome

 @example
 ```ts
 const outcome = await runCommandBounded({ repository: '/r', command, },);
 ```
 */
export async function runCommandBounded({
  repository,
  command,
}: {
  readonly repository: string;
  readonly command: UpdateCommand;
},): Promise<CommandOutcome> {
  /**
   Program and arguments of the command.
   */
  const [program = '', ...args] = command.argv;
  /**
   The command in its own process group, so one signal reaches everything it started.
   */
  const child = spawn(
    program,
    args,
    {
      cwd: repository,
      detached: true,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Output kept for the notification, bounded.
   */
  const tail = { text: '', };
  copyOutput({
    child,
    tail,
  },);
  /**
   The pending stop of the command's process group, cancelled when this function ends.
   */
  using stopper = stopGroupAfter({
    pid: child.pid ?? 0,
    timeoutMs: command.timeoutMs,
  },);
  /**
   How the command ended.
   */
  const end = await endOf(child,);
  return {
    exitCode: end.exitCode,
    timedOut: stopper.fired(),
    outputTail: tail.text,
  };
}
