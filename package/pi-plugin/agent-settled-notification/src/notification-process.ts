/**
 Subprocess boundary that runs the desktop-notification executable.

 `notify-send` waits for the notification server's D-Bus reply, and that wait is
 GLib's 25 s method-call default unless the caller bounds it, so this module owns
 both bounds that keep a desktop refusal or stall from living longer than this
 package allows: a wall-clock deadline that kills the child, and a cap on how
 much of its standard error is kept for the log.

 The executable's own words are the whole diagnostic value here: Plasma refuses a
 repeated notification with `Failed to show notification:
 GDBus.Error:org.freedesktop.Notifications.Error.ExcessNotificationGeneration:
 Created too many similar notifications in quick succession`, and a package that
 discards that text can only report "exit code 1".
 @module
 */

import { spawn, } from 'node:child_process';

//region Constants

/**
 Bytes of the notification executable's standard error kept for a diagnostic.

 A Plasma refusal is about 200 bytes, so this holds any refusal whole, and a
 notifier that floods its error stream is cut at a bound instead of buffered
 without one.
 */
const NOTIFICATION_CAPTURE_LIMIT_BYTES = 4_096;

/**
 Text appended to a capture that reached the limit, so a reader knows the
 executable said more than is quoted.
 */
const STDERR_TRUNCATION_MARKER = ' [stderr truncated]';

//endregion Constants

//region Types

/**
 Command and arguments passed to the desktop-notification executable.
 */
type NotificationInvocation = {
  /**
   Executable resolved through `PATH`.
   */
  readonly command: string;

  /**
   Fixed arguments kept outside shell syntax.
   */
  readonly args: readonly string[];
};

/**
 One process run's request, including the deadline that bounds it.
 */
type NotificationProcessInput = NotificationInvocation & {
  /**
   Milliseconds after which the executable is killed.
   */
  readonly timeoutMs: number;
};

/**
 How one run of the notification executable ended.

 These are the ways `node:child_process` reports a finished child, kept as a closed
 set so no caller interprets a nullable pair and no caller mistakes "no answer" for
 "refused".
 */
type NotificationAttemptEnding =
  | {
    /**
     The process chose its own exit status.
     */
    readonly kind: 'exited';

    /**
     Status the process exited with.
     */
    readonly exitCode: number;
  }
    | {
    /**
     A signal ended the process.
     */
      readonly kind: 'ended-by-signal';

    /**
     Signal that ended the process.
     */
      readonly signalName: string;
    }
    | {
    /**
     This package's deadline killed the process.
     */
      readonly kind: 'killed-at-its-deadline';
    };

/**
 What one process run reported, whether or not the desktop accepted the notification.
 */
type NotificationAttemptOutcome = {
  /**
   How the process ended.
   */
  readonly ending: NotificationAttemptEnding;

  /**
   Standard error, trimmed and bounded by {@link NOTIFICATION_CAPTURE_LIMIT_BYTES}.
   */
  readonly stderr: string;
};

/**
 Injectable process boundary used to verify delivery without touching a desktop.
 */
type NotificationProcessRunner = (
  input: NotificationProcessInput,
) => Promise<NotificationAttemptOutcome>;

/**
 What the process boundary has collected about one child while it runs.
 */
type NotificationProcessRun = {
  /**
   Standard error captured so far, never longer than the capture limit.
   */
  captured: string;

  /**
   Whether the executable wrote more than the capture limit keeps.
   */
  truncated: boolean;

  /**
   Whether the deadline already killed the child.
   */
  timedOut: boolean;

  /**
   Whether this run's promise already settled.
   */
  reported: boolean;
};

//endregion Types

//region Spawn failure

/**
 Reports that the notification executable could not be started at all.

 A missing executable cannot be fixed by waiting, so this failure is distinct from
 a process that started and was refused: only the latter is worth retrying.
 */
class NotificationSpawnError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'NotificationSpawnError';

  /**
   Creates a start failure naming the executable that could not be started.

   @param command - executable the process boundary could not start

   @param cause - value the operating system reported for the failed start
   */
  constructor(
    {
      command,
      cause,
    }: {
      readonly command: string;
      readonly cause: unknown;
    },
  ) {
    super(
      `could not start ${command}`,
      { cause, },
    );
  }
}

//endregion Spawn failure

//region Process boundary

/**
 Runs the desktop-notification executable once, bounded in time and in captured output.

 The child gets no terminal input, its standard output is discarded, and its
 standard error is captured up to {@link NOTIFICATION_CAPTURE_LIMIT_BYTES}. The
 deadline kills it with `SIGKILL` rather than `SIGTERM`, because a bound a child
 can ignore is not a bound.

 @param command - executable resolved through `PATH`

 @param args - fixed values passed outside shell syntax

 @param timeoutMs - milliseconds after which the child is killed

 @returns how the process ended plus the standard error it wrote

 @throws NotificationSpawnError when the executable cannot be started

 @example
 ```ts
 const outcome = await runNotificationProcess({
   command: 'notify-send',
   args: ['--app-name=Pi', 'Pi agent finished', 'Idle in /work/agent-7'],
   timeoutMs: 6_000,
 });
 if (outcome.ending.kind !== 'exited' || outcome.ending.exitCode !== 0)
   logger.warn(`notify-send reported: ${outcome.stderr}`);
 ```
 */
function runNotificationProcess(
  {
    command,
    args,
    timeoutMs,
  }: NotificationProcessInput,
): Promise<NotificationAttemptOutcome> {
  /* oxlint-disable promise/avoid-new -- A child process reports through events, so a promise that settles exactly once is the only bridge (rule PP1). */
  return new Promise(function startBoundedProcess(
    resolve,
    reject,
  ) {
    /**
     Child running the notification executable.
     */
    const child = spawn(
      command,
      [...args,],
      {
        stdio: [
          'ignore',
          'ignore',
          'pipe',
        ],
      },
    );

    /**
     What this run has collected so far.
     */
    const run: NotificationProcessRun = {
      captured: '',
      truncated: false,
      timedOut: false,
      reported: false,
    };

    /**
     Timer that ends the child at the deadline.
     */
    const deadline = setTimeout(
      function killAtDeadline(): void {
        run.timedOut = true;
        child.kill('SIGKILL',);
      },
      timeoutMs,
    );

    /**
     Settles this run with the ending it reached and the output it captured, and
     marks the run reported so no second event settles it.

     @param ending - how the process ended
     */
    function finish(ending: NotificationAttemptEnding,): void {
      run.reported = true;
      clearTimeout(deadline,);
      resolve({
        ending,
        stderr: `${run.captured
          .trim()}${run.truncated ? STDERR_TRUNCATION_MARKER : ''}`,
      },);
    }

    child.stderr
      ?.setEncoding('utf8',);
    child.stderr
      ?.on(
      'data',
      function captureStderr(chunk: string,): void {
        if (run.captured
          .length
          >= NOTIFICATION_CAPTURE_LIMIT_BYTES) {
          run.truncated = true;
          return;
        }
        /**
         Bytes of this chunk that still fit inside the capture limit.
         */
        const room = NOTIFICATION_CAPTURE_LIMIT_BYTES
          - run.captured
            .length;
        if (chunk.length > room) {
          run.captured += chunk.slice(
            0,
            room,
          );
          run.truncated = true;
          return;
        }
        run.captured += chunk;
      },
      );

    child.on(
      'error',
      function reportStartFailure(error: Error,): void {
        if (run.reported)
          return;
        run.reported = true;
        clearTimeout(deadline,);
        reject(new NotificationSpawnError({
          command,
          cause: error,
        },),);
      },
    );

    child.on(
      'close',
      function reportExit(
        exitCode,
        signalName,
      ): void {
        if (run.reported)
          return;
        if (run.timedOut) {
          finish({ kind: 'killed-at-its-deadline', },);
          return;
        }
        if (signalName !== null) {
          finish({
            kind: 'ended-by-signal',
            signalName,
          },);
          return;
        }
        if (exitCode !== null) {
          finish({
            kind: 'exited',
            exitCode,
          },);
          return;
        }
        // `close` reports one of the two, so this state means Node changed shape.
        run.reported = true;
        clearTimeout(deadline,);
        reject(new Error(`child process ${command} closed without an exit code or a signal`,),);
      },
    );
  },);
  /* oxlint-enable promise/avoid-new */
}

//endregion Process boundary

export {
  NOTIFICATION_CAPTURE_LIMIT_BYTES,
  NotificationSpawnError,
  runNotificationProcess,
};

export type {
  NotificationAttemptEnding,
  NotificationAttemptOutcome,
  NotificationInvocation,
  NotificationProcessInput,
  NotificationProcessRunner,
};
