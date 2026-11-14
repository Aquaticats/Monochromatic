/**
 One child process the proof runs, under a bound, and ended for certain when
 the bound passes.

 pi force-exits only on the second signal it receives:
 its bundled handler counts them and calls `process.exit` when the count passes
 one,
 so a single `SIGTERM` at the bound can leave pi alive inside a shutdown that
 never finishes.
 A live child also keeps its piped outputs open,
 which holds the proof's own process alive after every check has passed.
 So a run that reaches its bound is sent a second signal after a wait,
 then killed after another wait,
 and its streams and handle are released either way.

 Measured on this machine on 2026-11-14:
 a stand-in that traps `SIGTERM` was still running 3 s after a 1 s bound,
 and a proof process with its pipes attached did not exit until the stand-in
 was killed.

 @module
 */

import {
  spawn,
  type ChildProcessByStdio,
} from 'node:child_process';
import type { Readable, } from 'node:stream';
import { once, } from 'node:events';
import { setTimeout as sleepFor, } from 'node:timers/promises';

//region Constants

/**
 Text an abort shows up as, which is how a run reaches its bound.
 */
const ABORT_ERROR_NAME = 'AbortError';

/**
 Wait for the child to end after the bound's own signal,
 before the second signal is sent.
 */
const SECOND_SIGNAL_WAIT_MILLISECONDS = 2_000;

/**
 Wait for the child to end after the kill signal,
 before the proof stops waiting on it.
 */
const KILL_WAIT_MILLISECONDS = 2_000;

/**
 Wait for both output streams to close,
 so the output a run printed last is not lost to a return that outraces it.
 */
const STREAM_CLOSE_WAIT_MILLISECONDS = 2_000;

//endregion Constants

//region Types

/**
 The child this module starts:
 no standard input, both outputs piped,
 so neither output hangs on a reader that never comes.
 */
type PipedChild = ChildProcessByStdio<null, Readable, Readable>;

/**
 One child to run, and the bound it gets.
 */
export type BoundedChildRequest = {
  /**
   Program to run, by path.
   */
  readonly file: string;
  /**
   Arguments it is given.
   */
  readonly args: readonly string[];
  /**
   Folder it runs in.
   */
  readonly cwd: string;
  /**
   Environment it sees.
   */
  readonly env: NodeJS.ProcessEnv;
  /**
   Bound on the run, which is also when the first signal is sent.
   */
  readonly boundMilliseconds: number;
};

/**
 What one bounded child left behind.
 */
export type BoundedChildResult = {
  /**
   Everything it wrote to standard output,
   including what it printed before a bound ended it.
   */
  readonly stdout: string;
  /**
   Everything it wrote to standard error.
   */
  readonly stderr: string;
  /**
   Exit status it reported, absent when a signal ended it.
   */
  readonly exitCode?: number;
  /**
   Process id it ran under, absent when the start itself failed.
   */
  readonly pid?: number;
  /**
   Whether it reached its bound and had to be ended.
   */
  readonly timedOut: boolean;
};

//endregion Types

//region Ending one child

/**
 Whether the child has already ended, by an exit status or by a signal.

 @param child - child the run started

 @returns `true` when nothing is left to signal

 @example
 ```typescript
 if (hasEnded({ child })) {
   // nothing left to kill
 }
 ```
 */
function hasEnded({
  child,
}: {
  readonly child: PipedChild;
},): boolean {
  return (child.exitCode !== null) || (child.signalCode !== null);
}

/**
 Wait for the child to end, for no longer than one stated wait.

 @param child - child the run started

 @param milliseconds - longest this call waits

 @returns `true` when the child has ended

 @example
 ```typescript
 const ended = await endedWithin({ child, milliseconds: 2_000 });
 ```
 */
async function endedWithin({
  child,
  milliseconds,
}: {
  readonly child: PipedChild;
  readonly milliseconds: number;
},): Promise<boolean> {
  try {
    await Promise.race([
      once(
        child,
        'exit',
      ),
      sleepFor(milliseconds,),
    ],);
    return hasEnded({ child, },);
  } catch (error) {
    // An `error` event is what the abort itself emits, so it is expected here;
    // anything else is a failure of this wait rather than of the child.
    if ((Error.isError(error,)) && (error.name === ABORT_ERROR_NAME))
      return hasEnded({ child, },);
    throw error;
  }
}

/**
 End one child for certain:
 a second signal after a wait,
 then a kill after another wait,
 whatever it ignores.

 @param child - child the run started

 @example
 ```typescript
 await endForCertain({ child });
 ```
 */
async function endForCertain({
  child,
}: {
  readonly child: PipedChild;
},): Promise<void> {
  if (hasEnded({ child, },))
    return;
  child.kill('SIGTERM',);
  if (await endedWithin({
    child,
    milliseconds: SECOND_SIGNAL_WAIT_MILLISECONDS,
  },))
    return;
  child.kill('SIGKILL',);
  await endedWithin({
    child,
    milliseconds: KILL_WAIT_MILLISECONDS,
  },);
}

/**
 Read both output streams to their end, then release them and the child handle,
 so a process that outlived the run cannot keep this proof alive.

 @param child - child the run started

 @example
 ```typescript
 await closedStreams({ child });
 ```
 */
async function closedStreams({
  child,
}: {
  readonly child: PipedChild;
},): Promise<void> {
  try {
    await Promise.race([
      once(
        child,
        'close',
      ),
      sleepFor(STREAM_CLOSE_WAIT_MILLISECONDS,),
    ],);
  } catch (error) {
    // An `error` event is what the abort emits; the streams are released
    // either way, so only an unexpected failure is raised.
    if (!((Error.isError(error,)) && (error.name === ABORT_ERROR_NAME)))
      throw error;
  }
  child.stdout
    .destroy();
  child.stderr
    .destroy();
  child.unref();
}

/**
 How one child ended:
 its exit status when it ended by itself, or that its bound ended it.

 @param child - child the run started

 @returns the exit status, or that the run reached its bound

 @example
 ```typescript
 const ending = await endingOf({ child });
 ```
 */
async function endingOf({
  child,
}: {
  readonly child: PipedChild;
},): Promise<{
  readonly exitCode?: number;
  readonly timedOut: boolean;
}> {
  try {
    await once(
      child,
      'exit',
    );
    /**
     Exit status the child reported, which it leaves absent when a signal ended
     it.
     */
    const { exitCode, } = child;
    return {
      ...(exitCode === null ? {} : { exitCode, }),
      timedOut: false,
    };
  } catch (error) {
    return {
      timedOut: (Error.isError(error,))
        && (error.name === ABORT_ERROR_NAME),
    };
  }
}

//endregion Ending one child

//region Running one child

/**
 Run one child under a bound, and end it for certain when the bound passes.

 @param request - the program, its arguments, its folder, its environment and
 its bound

 @returns what it wrote, how it ended, and the process id it ran under

 @example
 ```typescript
 const child = await runBoundedChild({
   file: process.execPath,
   args: ['-e', 'setTimeout(() => {}, 60000)'],
   cwd: folder,
   env: process.env,
   boundMilliseconds: 300,
 });
 ```
 */
export async function runBoundedChild(
  request: BoundedChildRequest,
): Promise<BoundedChildResult> {
  /**
   Fields of the request this run reads.
   */
  const {
    file,
    args,
    cwd,
    env,
    boundMilliseconds,
  } = request;
  /**
   Child process for this run.
   */
  const child = spawn(
    file,
    [
      ...args,
    ],
    {
      cwd,
      // Standard input is refused rather than piped: a program that reads a
      // piped standard input waits for input nobody sends, and a pipe nobody
      // closes holds the run open.
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ] as const,
      env,
      // The bound's own signal, which a program may trap or ignore.
      signal: AbortSignal.timeout(boundMilliseconds,),
    },
  );
  /**
   Standard output the child prints, in order.
   */
  const stdoutChunks: Buffer[] = [];
  /**
   Standard error the child prints, in order.
   */
  const stderrChunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collectStdout(chunk: Buffer,) {
      stdoutChunks.push(chunk,);
    },
    );
  child.stderr
    .on(
    'data',
    function collectStderr(chunk: Buffer,) {
      stderrChunks.push(chunk,);
    },
    );
  /**
   How the child ended.
   */
  const ending = await endingOf({ child, },);
  if (ending.timedOut)
    await endForCertain({ child, },);
  await closedStreams({ child, },);
  return {
    stdout: Buffer.concat(stdoutChunks,)
      .toString('utf8',),
    stderr: Buffer.concat(stderrChunks,)
      .toString('utf8',),
    ...(ending.exitCode === undefined ? {} : { exitCode: ending.exitCode, }),
    ...(child.pid === undefined ? {} : { pid: child.pid, }),
    timedOut: ending.timedOut,
  };
}

//endregion Running one child
