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

 Both events this module waits on are promised when the child starts,
 not when the wait begins:
 a listener attached after the event has fired waits for something that has
 already happened,
 and pays its stated wait for nothing.
 A child that ended at 100 ms was measured returning at 2 120 ms that way
 (review of 2026-11-13, `probe-bounded-child-timed.mjs early-exit-repeat`).
 The run reports which waits had to fall back
 ({@link BoundedChildResult.fallbackWaits}),
 so a caller can say the event ended a wait rather than that the wait was
 short.

 @module
 */

import {
  spawn,
  type ChildProcessByStdio,
} from 'node:child_process';
import type { Readable, } from 'node:stream';
import { setTimeout as sleepFor, } from 'node:timers/promises';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Text the bound's own abort arrives as,
 which is the one `error` event that does not mean the child never started.
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
 so the output a run printed last is not lost,
 and so a stream a grandchild keeps open cannot hold the proof.
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
 One wait this module can take,
 named for the event it waits on,
 so a reader of a result knows which wait fell back to its stated bound.
 */
export type BoundedChildWait = 'exit' | 'close';

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
  /**
   Which waits had to stop waiting for their event, in the order they did:
   `'exit'` for the child's own end, `'close'` for its output streams closing.
   An entry means the stated wait ended that wait rather than the event.
   */
  readonly fallbackWaits: readonly BoundedChildWait[];
};

/**
 What this run has observed about its child, filled by the listeners the run
 attaches when the child starts.
 */
type ChildProgress = {
  /**
   The child's own end, promised from the start.
   */
  readonly exited: Promise<void>;
  /**
   Both output streams closing, promised from the start.
   */
  readonly closed: Promise<void>;
  /**
   A start that failed, promised so nothing waits on a child that never ran.
   */
  readonly failed: Promise<void>;
  /**
   Waits that fell back to their stated bound, in the order they did.
   */
  readonly fallbackWaits: BoundedChildWait[];
};

//endregion Types

//region Waiting without missing an event

/**
 One promise's arrival, tagged so a race can say which side arrived.

 @param promise - promise to wait on

 @returns a promise that resolves when that one does

 @example
 ```typescript
 const winner = await Promise.race([arrived({ promise }), waitPassed({ milliseconds: 2_000 })]);
 ```
 */
async function arrived({
  promise,
}: {
  readonly promise: Promise<void>;
},): Promise<{ readonly arrived: true }> {
  await promise;
  return { arrived: true, };
}

/**
 One stated wait, tagged so a race can say it was the one that ended the wait.

 @param milliseconds - how long to wait

 @returns a promise that resolves after that wait

 @example
 ```typescript
 const winner = await waitPassed({ milliseconds: 2_000 });
 ```
 */
async function waitPassed({
  milliseconds,
}: {
  readonly milliseconds: number;
},): Promise<{ readonly waited: true }> {
  await sleepFor(milliseconds,);
  return { waited: true, };
}

/**
 A promise that resolves once one signal has aborted, or at once when it
 already has.

 @param signal - signal to watch

 @returns a promise that resolves when the signal aborts

 @example
 ```typescript
 const bound = await abortArrival({ signal: AbortSignal.timeout(1_000) });
 ```
 */
function abortArrival({
  signal,
}: {
  readonly signal: AbortSignal;
},): Promise<{ readonly aborted: true }> {
  /**
   The arrival, resolved by the signal's own abort.
   */
  const arrival = Promise.withResolvers<{ readonly aborted: true }>();
  if (signal.aborted) {
    arrival.resolve({ aborted: true, },);
    return arrival.promise;
  }
  signal.addEventListener(
    'abort',
    function onAbort() {
      arrival.resolve({ aborted: true, },);
    },
    { once: true, },
  );
  return arrival.promise;
}

//endregion Waiting without missing an event

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

 @param progress - what this run has observed, which records a fallback

 @param milliseconds - longest this call waits

 @returns `true` when the child has ended

 @example
 ```typescript
 const ended = await endedWithin({ child, progress, milliseconds: 2_000 });
 ```
 */
async function endedWithin({
  child,
  progress,
  milliseconds,
}: {
  readonly child: PipedChild;
  readonly progress: ChildProgress;
  readonly milliseconds: number;
},): Promise<boolean> {
  if (hasEnded({ child, },))
    return true;
  /**
   Which side ended this wait: the child's end, which was promised when it
   started, or the stated wait.
   */
  const winner = await Promise.race([
    arrived({ promise: progress.exited, }),
    waitPassed({ milliseconds, },),
  ],);
  if ('waited' in winner)
    progress.fallbackWaits
      .push('exit',);
  return hasEnded({ child, },);
}

/**
 End one child for certain:
 a second signal after a wait,
 then a kill after another wait,
 whatever it ignores.

 @param child - child the run started

 @param progress - what this run has observed

 @example
 ```typescript
 await endForCertain({ child, progress });
 ```
 */
async function endForCertain({
  child,
  progress,
}: {
  readonly child: PipedChild;
  readonly progress: ChildProgress;
},): Promise<void> {
  if (hasEnded({ child, },))
    return;
  child.kill('SIGTERM',);
  if (await endedWithin({
    child,
    progress,
    milliseconds: SECOND_SIGNAL_WAIT_MILLISECONDS,
  },))
    return;
  child.kill('SIGKILL',);
  await endedWithin({
    child,
    progress,
    milliseconds: KILL_WAIT_MILLISECONDS,
  },);
}

/**
 Read both output streams to their end, then release them and the child handle,
 so a process that outlived the run cannot keep this proof alive.

 @param child - child the run started

 @param progress - what this run has observed, which records a fallback

 @example
 ```typescript
 await closedStreams({ child, progress });
 ```
 */
async function closedStreams({
  child,
  progress,
}: {
  readonly child: PipedChild;
  readonly progress: ChildProgress;
},): Promise<void> {
  /**
   Which side ended this wait: both streams closing, promised when the child
   started, or the stated wait.
   */
  const winner = await Promise.race([
    arrived({ promise: progress.closed, }),
    waitPassed({ milliseconds: STREAM_CLOSE_WAIT_MILLISECONDS, },),
  ],);
  if ('waited' in winner)
    progress.fallbackWaits
      .push('close',);
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

 @param progress - what this run has observed

 @param bound - the signal the run's bound raises

 @returns the exit status, or that the run reached its bound

 @example
 ```typescript
 const ending = await endingOf({ child, progress, bound });
 ```
 */
async function endingOf({
  child,
  progress,
  bound,
}: {
  readonly child: PipedChild;
  readonly progress: ChildProgress;
  readonly bound: AbortSignal;
},): Promise<{
  readonly exitCode?: number;
  readonly timedOut: boolean;
}> {
  /**
   Which side ended this wait: the child's own end, a start that failed, or
   the bound's signal.
   */
  const winner = await Promise.race([
    arrived({ promise: progress.exited, }),
    arrived({ promise: progress.failed, }),
    abortArrival({ signal: bound, },),
  ],);
  if ('aborted' in winner)
    return { timedOut: true, };
  /**
   Exit status the child reported, which it leaves absent when a signal ended
   it.
   */
  const { exitCode, } = child;
  return {
    ...(exitCode === null ? {} : { exitCode, }),
    timedOut: false,
  };
}

//endregion Ending one child

//region Running one child

/**
 Run one child under a bound, and end it for certain when the bound passes.

 @param request - the program, its arguments, its folder, its environment and
 its bound

 @returns what it wrote, how it ended, the process id it ran under, and which
 waits had to fall back

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
   The run's bound, whose signal is the first one the child gets.
   */
  const bound = AbortSignal.timeout(boundMilliseconds,);
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
      signal: bound,
    },
  );
  /**
   The child's own end, promised now so a wait begun later cannot miss it.
   */
  const exited = Promise.withResolvers<void>();
  /**
   Both output streams closing, promised now for the same reason.
   */
  const closed = Promise.withResolvers<void>();
  child.once(
    'exit',
    function markExit() {
      exited.resolve();
    },
  );
  child.once(
    'close',
    function markClose() {
      closed.resolve();
    },
  );
  /**
   A start that failed, which ends both waits so nothing here waits on a child
   that never ran.
   */
  const failed = Promise.withResolvers<void>();
  child.once(
    'error',
    function markFailedStart(error: ForeignBorrowed<Error>,) {
      // The bound's own abort arrives here too, and it is not a start failure:
      // the child is running, and its own exit is what ends its wait.
      if (error.name === ABORT_ERROR_NAME)
        return;
      failed.resolve();
      exited.resolve();
      closed.resolve();
    },
  );
  /**
   What this run has observed about its child.
   */
  const progress: ChildProgress = {
    exited: exited.promise,
    closed: closed.promise,
    failed: failed.promise,
    fallbackWaits: [],
  };
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
  const ending = await endingOf({
    child,
    progress,
    bound,
  },);
  if (ending.timedOut)
    await endForCertain({
      child,
      progress,
    },);
  await closedStreams({
    child,
    progress,
  },);
  return {
    stdout: Buffer.concat(stdoutChunks,)
      .toString('utf8',),
    stderr: Buffer.concat(stderrChunks,)
      .toString('utf8',),
    ...(ending.exitCode === undefined ? {} : { exitCode: ending.exitCode, }),
    ...(child.pid === undefined ? {} : { pid: child.pid, }),
    timedOut: ending.timedOut,
    fallbackWaits: [
      ...progress.fallbackWaits,
    ],
  };
}

//endregion Running one child
