/**
 One run of the btrfs controls on this host: the package's own task, bounded by a deadline, with each passing test's name
 in the output.

 nextest prints a status line for a passing test only above a status level that a person's own nextest configuration can
 silence, so the run sets `NEXTEST_STATUS_LEVEL` and `NEXTEST_FINAL_STATUS_LEVEL`. The task keeps the flags it has, so
 nothing about the suite is spelled twice (`doc/troubleshooting/nextest-hides-a-passing-tests-output.md`).
 */
import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once } from 'node:events';

import {
  hostControlTask,
  millisecondsPerSecond,
} from './native-release-host-control-requirements.ts';

//region How a run ended

/**
 How a run ended: by itself, with the status it exited with, or by a signal that stopped it.
 */
export type HostControlEnding =
  | {
    readonly kind: 'exited';
    readonly status: number
  }
    | {
      readonly kind: 'stopped';
      readonly signal: string
    };

/**
 One finished run of the host controls, captured while it ran.
 */
export type HostControlRun = {
  /**
   How the run ended.
   */
  readonly ending: HostControlEnding;
  /**
   Everything the run printed, standard output and standard error in the order it arrived.
   */
  readonly output: string;
  /**
   Whether the deadline stopped the run instead of it ending by itself.
   */
  readonly timedOut: boolean;
};

/**
 Starts one run of the host controls and reports how it ended.
 */
export type HostControlRunner = (filters: readonly string[]) => Promise<HostControlRun>;

//endregion How a run ended

//region Reading the command

/**
 Write one child's stream to this process's own stream and keep the text.

 @param stream - the child's stream

 @param target - this process's stream of the same kind, so the run stays visible while it lasts

 @param keep - receives each decoded chunk, in arrival order
 */
async function teeAndKeep({
  stream,
  target,
  keep,
}: {
  readonly stream: NodeJS.ReadableStream;
  readonly target: NodeJS.WritableStream;
  readonly keep: (text: string) => void;
}): Promise<void> {
  for await (const chunk of stream) {
    /**
     The chunk as text, which nextest prints line by line.
     */
    const text = (typeof chunk) === 'string' ? chunk : String(chunk as Buffer);
    keep(text);
    target.write(text);
  }
}

/**
 How a finished command ended.

 @param child - the command that has closed

 @returns its exit status, or the signal that ended it
 */
function endingOf(child: ChildProcess): HostControlEnding {
  if (child.exitCode !== null) {
    return {
      kind: 'exited',
      status: child.exitCode,
    };
  }
  return {
    kind: 'stopped',
    signal: child.signalCode ?? 'an unknown signal',
  };
}

/**
 Whether a caught value is a system error, which carries the code that says why a call was refused.

 @param value - the caught value

 @returns true for an error with a `code` field

 @example
 ```ts
 isSystemError(new Error('not a system error'));
 ```
 */
function isSystemError(value: unknown): value is NodeJS.ErrnoException {
  return ((typeof value) === 'object') && (value !== null)
    && ('code' in value);
}

/**
 Stop a command and everything it started, so the deadline leaves no test behind.

 @param child - the started command, made a process group leader when it was spawned
 */
function stopGroup(child: ChildProcess): void {
  /**
   The command's own handle, whose process ID is also its group's because it was spawned detached.
   */
  const { pid } = child;
  if (pid === undefined) {
    child.kill('SIGKILL');
    return;
  }
  try {
    process.kill(
      -pid,
      'SIGKILL'
    );
  }
  catch (error) {
    // A group the kernel no longer has answers ESRCH; the command itself may still run, so it is stopped as well.
    if ((!isSystemError(error)) || (error.code !== 'ESRCH'))
      throw error;
    child.kill('SIGKILL');
  }
}

//endregion Reading the command

//region The run on this host

/**
 Run the host controls through the package's own task, bounded by a deadline.

 @param filters - test name filters for a narrowed run

 @param limitSeconds - deadline for the run, counted from its start

 @returns how the run ended, everything it printed, and whether the deadline stopped it

 @example
 ```ts
 await runHostControlsOnThisHost({ filters: ['subvolume::'], limitSeconds: hostControlLimitSeconds });
 ```
 */
export async function runHostControlsOnThisHost({
  filters,
  limitSeconds,
}: {
  readonly filters: readonly string[];
  readonly limitSeconds: number;
}): Promise<HostControlRun> {
  /**
   Everything the run printed, in arrival order.
   */
  const chunks: string[] = [];
  /**
   The run, in a process group of its own so the deadline can stop the tests it starts.
   */
  const child = spawn(
    'mise',
    [
      'run',
      hostControlTask,
      ...(filters.length === 0 ? [] : [
        '--',
        ...filters,
      ]),
    ],
    {
      env: {
        ...process.env,
        NEXTEST_STATUS_LEVEL: 'pass',
        NEXTEST_FINAL_STATUS_LEVEL: 'pass',
      },
      detached: true,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Whether the deadline stopped the run, so a stopped run is never read as a failed test.
   */
  const deadlineState = { timedOut: false };
  /**
   The deadline, which stops the whole group and releases the caller that waits here.
   */
  const deadline = setTimeout(
    function stopAtDeadline() {
      deadlineState.timedOut = true;
      stopGroup(child);
    },
    limitSeconds * millisecondsPerSecond
  );
  /**
   The streams, read together so the text keeps the order the run printed it in.
   */
  const streams = Promise.all([
    teeAndKeep({
      stream: child.stdout,
      target: process.stdout,
      keep: function keepStdout(text) {
        chunks.push(text);
      },
    }),
    teeAndKeep({
      stream: child.stderr,
      target: process.stderr,
      keep: function keepStderr(text) {
        chunks.push(text);
      },
    }),
  ]);
  await Promise.all([
    streams,
    once(
      child,
      'close',
    ),
  ]);
  clearTimeout(deadline);
  return {
    ending: endingOf(child),
    output: chunks.join(''),
    timedOut: deadlineState.timedOut,
  };
}

//endregion The run on this host
