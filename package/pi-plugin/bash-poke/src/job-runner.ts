/**
 Background command execution owned by this package.

 @module
 */

import {
  spawn as spawnChildProcess,
  type ChildProcess,
} from 'node:child_process';
import { randomUUID, } from 'node:crypto';
import { once, } from 'node:events';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  CANCEL_SIGNAL,
  FORCE_SIGNAL,
} from './constants.ts';
import { bashPokeLogger, } from './logger.ts';
import {
  createOutputRecorder,
  type OutputRecorder,
} from './output-recorder.ts';
import { sanitizeOutput, } from './output-sanitize.ts';
import type { ShellInvocation, } from './shell-resolution.ts';
import {
  openSpoolWriter,
  type SpoolWriter,
} from './spool.ts';

//region Types

/**
 Values this package reads from Node's untyped child `close` payload.
 */
type CloseReading = {
  /**
   Exit code, absent when a signal ended the process.
   */
  readonly code?: number;

  /**
   Signal name, absent when the process exited on its own.
   */
  readonly signal?: string;
};

/**
 How one background job ended.
 */
type JobOutcome = {
  /**
   Reported exit code, absent when the job was signalled or never started.
   */
  readonly exitCode?: number;

  /**
   Whether the user cancelled the job.
   */
  readonly cancelled: boolean;

  /**
   Spawn failure text, present only when the command could not be started.
   */
  readonly spawnError?: string;
};

/**
 Cancellation state for one job, mutable because a cancel arrives after start.
 */
type CancellationState = {
  /**
   Whether the user asked for this job to stop.
   */
  cancelled: boolean;
};

/**
 Pending force-kill timers for one job, mutable because a cancel arms them.
 */
type KillTimers = {
  /**
   Armed force-kill timer, absent while no cancel is in flight.
   */
  forceKill?: NodeJS.Timeout;
};

/**
 A running background job plus the handles needed to report and cancel it.
 */
type RunningJob = {
  /**
   Identifier naming this job and its spool file.
   */
  readonly id: string;

  /**
   Command line as the user typed it.
   */
  readonly command: string;

  /**
   Start timestamp used for elapsed time in the progress widget.
   */
  readonly startedAt: number;

  /**
   Bounded recorder holding the head and tail of everything printed.
   */
  readonly recorder: OutputRecorder;

  /**
   Path holding complete output, absent when no spool file could be opened.
   */
  readonly spoolPath?: string;

  /**
   Stops the job's whole process group and suppresses its poke.
   */
  readonly cancel: () => void;

  /**
   Settles once the job exits, its spool file is closed, and its outcome is known.
   */
  readonly finished: Promise<JobOutcome>;
};

/**
 Everything needed to start one background job.
 */
type StartJobInput = {
  /**
   Command line to run.
   */
  readonly command: string;

  /**
   Working directory the command runs in.
   */
  readonly cwd: string;

  /**
   Resolved shell plus the flag carrying command text.
   */
  readonly shell: ShellInvocation;

  /**
   Characters kept from the start of output.
   */
  readonly headChars: number;

  /**
   Characters kept from the end of output.
   */
  readonly tailChars: number;

  /**
   Milliseconds between SIGTERM and SIGKILL on cancel.
   */
  readonly killGraceMs: number;

  /**
   Temp root for the spool file, injectable for disposable test directories.
   */
  readonly tmp?: string;

  /**
   Notified after each output chunk so progress display can refresh.
   */
  readonly onChunk?: () => void;

  /**
   Clock, injectable so elapsed time is deterministic in tests.
   */
  readonly now?: () => number;
};

//endregion Types

//region Close payload

/**
 Reads Node's untyped close payload into the values this package uses.
 
 Node documents the payload as an exit code and a signal name, either of which
 it reports as null, so each is narrowed by kind instead of being asserted into
 a tuple shape this package does not control.
 
 @param payload - value emitted with the child's close event
 
 @returns narrowed exit information
 
 @example
 ```ts
 readClosePayload({ payload: [0, null], },);
 ```
 */
function readClosePayload({ payload, }: { readonly payload: unknown; }, ): CloseReading {
  if (!Array.isArray(payload, ))
    return {};

  /**
   First emitted value, an exit code when the process exited normally.
   */
  const code: unknown = payload[0];

  /**
   Second emitted value, a signal name when the process was signalled.
   */
  const signal: unknown = payload[1];
  return {
    ...((typeof code) === 'number' ? { code, } : {}),
    ...((typeof signal) === 'string' ? { signal, } : {}),
  };
}

//endregion Close payload

//region Execution

/**
 Starts one command in the background and returns its handles.
 
 The child leads its own process group so cancellation can reach everything the
 command spawned, which is what makes a package-manager build cancellable
 rather than merely interruptible. Output is bounded in memory by the recorder
 and spooled in full to disk, with stream backpressure pausing the child when
 the disk falls behind.
 
 @param command - command line to run
 
 @param cwd - working directory
 
 @param shell - resolved shell invocation
 
 @param headChars - characters kept from the start of output
 
 @param tailChars - characters kept from the end of output
 
 @param killGraceMs - milliseconds between SIGTERM and SIGKILL on cancel
 
 @param tmp - temp root for the spool file
 
 @param onChunk - notified after each output chunk
 
 @param now - clock used for the start timestamp
 
 @returns running job handles
 
 @example
 ```ts
 const job = await startJob({
   command: 'printf hello',
   cwd: '/tmp',
   shell: { command: '/bin/bash', args: ['-c'], },
   headChars: 2000,
   tailChars: 6000,
   killGraceMs: 2000,
 },);
 await job.finished;
 ```
 */
async function startJob(
  {
    command,
    cwd,
    shell,
    headChars,
    tailChars,
    killGraceMs,
    tmp,
    onChunk,
    now = Date.now,
  }: StartJobInput,
): Promise<RunningJob> {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: startJob.name,
    l: bashPokeLogger,
  }, );

  /**
   Identifier naming this job and its spool file.
   */
  const id = randomUUID();

  /**
   Bounded recorder fed by every sanitized chunk.
   */
  const recorder = createOutputRecorder({
    headChars,
    tailChars,
  }, );

  /**
   Cancellation state read when the job exits and written by cancel.
   */
  const cancellation: CancellationState = { cancelled: false, };

  /**
   Pending force-kill timer holder, armed only by cancel.
   */
  const timers: KillTimers = {};

  /**
   Spool destination for complete output, discarding when it could not be opened.
   */
  const writer: SpoolWriter = await openSpoolWriter(
    tmp === undefined ? { jobId: id, } : {
      tmp,
      jobId: id,
    },
  );

  /**
   Detached child leading its own process group.
   */
  const child: ChildProcess = spawnChildProcess(
    shell.command,
    [
      ...shell.args,
      command,
    ],
    {
      cwd,
      detached: true,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      env: process.env,
    },
  );

  /**
   Stops reading from the child so a slow disk cannot grow memory without bound.
   */
  function pauseStreams(): void {
    child.stdout
      ?.pause();
    child.stderr
      ?.pause();
  }

  /**
   Resumes reading once buffered spool writes have reached disk.
   */
  function resumeStreams(): void {
    child.stdout
      ?.resume();
    child.stderr
      ?.resume();
  }

  /**
   Folds one chunk into the recorder, the spool file, and progress display.
   
   @param raw - chunk as delivered by the child's stream
   */
  function recordChunk(raw: string, ): void {
    /**
     Sanitized text, safe to render and to place inside a fence.
     */
    const text = sanitizeOutput({ text: raw, }, );
    if (text.length === 0)
      return;
    recorder.append(text, );

    /**
     Whether the spool stream accepted the write without buffering it.
     */
    const accepted = writer.write(text, );
    if (!accepted) {
      pauseStreams();
      writer.onceDrain(resumeStreams, );
    }
    if (onChunk !== undefined)
      onChunk();
  }

  if (child.stdout === null) {
    l.warn(`job ${id} has no stdout stream despite pipe stdio`, );
  }
  else {
    child.stdout
      .setEncoding('utf8', );
    child.stdout
      .on(
      'data',
      recordChunk,
      );
  }
  if (child.stderr === null) {
    l.warn(`job ${id} has no stderr stream despite pipe stdio`, );
  }
  else {
    child.stderr
      .setEncoding('utf8', );
    child.stderr
      .on(
      'data',
      recordChunk,
      );
  }

  /**
   Signals the whole process group, tolerating a group that already exited.
   
   @param signal - signal name to deliver
   */
  function signalGroup(signal: string, ): void {
    /**
     Group identifier, absent when the command could not be spawned at all.
     */
    const {pid} = child;
    if (pid === undefined) {
      l.debug(`job ${id} has no pid to signal with ${signal}`, );
      return;
    }
    try {
      // A negative pid targets the group the detached child leads, which is what
      // reaches grandchildren such as a package manager's own children.
      process.kill(
        -pid,
        signal,
      );
      l.debug(`sent ${signal} to job ${id} group ${String(pid)}`, );
    }
    catch (error: unknown) {
      // A group that already exited is the expected race when a command finishes
      // on its own while the user cancels, so the value is logged and dropped.
      l.debug(`signalling job ${id} with ${signal} failed: ${caughtValueText(error, )}`, );
    }
  }

  /**
   Releases any armed force-kill timer.
   */
  function disarmForceKill(): void {
    if (timers.forceKill === undefined)
      return;
    clearTimeout(timers.forceKill, );
    delete timers.forceKill;
  }

  /**
   Cancels the job, escalating to an uncatchable signal after the grace period.
   */
  function cancel(): void {
    if (cancellation.cancelled)
      return;
    cancellation.cancelled = true;
    l.debug(`cancelling job ${id}: ${command}`, );
    signalGroup(CANCEL_SIGNAL, );
    if (killGraceMs <= 0) {
      signalGroup(FORCE_SIGNAL, );
      return;
    }
    timers.forceKill = setTimeout(
      function forceKill(): void {
        signalGroup(FORCE_SIGNAL, );
      },
      killGraceMs,
    );
    // The grace timer must never be the reason a process stays alive.
    timers.forceKill
      .unref();
  }

  /**
   Waits for exit, closes the spool file, and reports the outcome.
   
   @returns how the job ended
   */
  async function awaitCompletion(): Promise<JobOutcome> {
    try {
      /**
       Raw close payload, or a rejection when the command could not be spawned.
       */
      const payload: unknown = await once(
        child,
        'close',
      );

      /**
       Exit information narrowed from that payload.
       */
      const reading = readClosePayload({ payload, }, );
      disarmForceKill();
      await writer.close();
      l.debug(`job ${id} closed: ${JSON.stringify(reading)}`, );
      return {
        cancelled: cancellation.cancelled,
        ...(reading.code === undefined ? {} : { exitCode: reading.code, }),
      };
    }
    catch (error: unknown) {
      // `once` rejects with the child's error event, which is how a missing
      // shell or an unexecutable command reaches the user as a poke.
      disarmForceKill();
      await writer.close();
      l.warn(`job ${id} failed: ${caughtValueText(error, )}`, );
      return {
        cancelled: cancellation.cancelled,
        spawnError: caughtValueText(error, ),
      };
    }
  }

  l.debug(`started job ${id} in ${cwd}: ${command}`, );
  return {
    id,
    command,
    startedAt: now(),
    recorder,
    ...(writer.path === undefined ? {} : { spoolPath: writer.path, }),
    cancel,
    finished: awaitCompletion(),
  };
}

//endregion Execution

export {
  readClosePayload,
  startJob,
};

export type {
  CloseReading,
  JobOutcome,
  KillTimers,
  RunningJob,
  StartJobInput,
};
