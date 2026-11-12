/**
 Bounds and retries a settled-agent desktop notification.

 Three numbers here come from measurement on this host, at load average 81 to 96,
 with twenty real deliveries of distinct payloads timed by `notify-send`:

 - Attempts took 20, 24, 52, 81, 89, 96, 107, 107, 120, 137, 141, 178, 198, 296,
   305, 347, 515, 632, 792 and 3 060 ms. The worst needs about 3.1 s, so
   {@link NOTIFICATION_ATTEMPT_TIMEOUT_MS} is roughly twice that.
 - A refusal is fast: Plasma answered with
   `org.freedesktop.Notifications.Error.ExcessNotificationGeneration` in 8 to
   133 ms, so retrying costs one wait, not one timeout.
 - The same payload stayed refused for 1 183 ms after the previous acceptance,
   which matches the server's own 1000 ms rule, so the first retry waits longer
   than that.

 Nothing here runs on Pi's settlement path: the registration starts a delivery and
 returns, and this work finishes in the background whichever way it ends.
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  runNotificationProcess,
  type NotificationAttemptOutcome,
  type NotificationInvocation,
  type NotificationProcessRunner,
} from './notification-process.ts';

//region Bounds

/**
 Longest one `notify-send` attempt may run.

 Twice the worst measured attempt (3 060 ms at load average 81 to 96), and far
 below the 25 s GLib method-call default `notify-send` itself would wait for.
 */
const NOTIFICATION_ATTEMPT_TIMEOUT_MS = 6_000;

/**
 Wait before the first retry.

 Longer than the 1 183 ms the notification server was measured to keep refusing one
 repeated payload, so the retry lands after the server's own 1000 ms window.
 */
const RETRY_DELAY_AFTER_MEASURED_WINDOW_MS = 1_500;

/**
 Wait before the second retry.

 Covers a second agent settling from the same working folder inside the first
 window, where this package's payload is identical again.
 */
const RETRY_DELAY_AFTER_REPEAT_COLLISION_MS = 3_000;

/**
 Waits tried after a failed attempt, in order.
 */
const NOTIFICATION_DELAYS_BEFORE_RETRY_MS: readonly number[] = [
  RETRY_DELAY_AFTER_MEASURED_WINDOW_MS,
  RETRY_DELAY_AFTER_REPEAT_COLLISION_MS,
];

/**
 Longest a whole delivery may take, retries and their waits included.

 Two attempts and the wait between them need at most 13 500 ms, so this bound
 leaves room for the third attempt only when the earlier ones answered quickly,
 and no delivery outlives it either way.
 */
const NOTIFICATION_TOTAL_BOUND_MS = 15_000;

//endregion Bounds

//region Types

/**
 Injectable delay, so a retry can be verified without waiting it out.
 */
type NotificationDelay = (milliseconds: number,) => Promise<unknown>;

/**
 Injectable monotonic clock in milliseconds.
 */
type NotificationClock = () => number;

/**
 Everything one delivery needs, with each boundary replaceable for tests.
 */
type SettledNotificationDelivery = {
  /**
   Executable and payload the notification carries.
   */
  readonly invocation: NotificationInvocation;

  /**
   Replacement process boundary for tests.
   */
  readonly run?: NotificationProcessRunner;

  /**
   Replacement retry delay for tests.
   */
  readonly delay?: NotificationDelay;

  /**
   Replacement clock for tests.
   */
  readonly now?: NotificationClock;
};

//endregion Types

//region Delivery failure

/**
 Reports that every attempt to deliver the settled-agent notification failed.

 The message carries what the desktop executable itself said, because that text is
 the only thing that distinguishes a refusal the next attempt can clear from a
 desktop that is not there at all.
 */
class NotificationUnavailableError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'NotificationUnavailableError';

  /**
   Attempts this delivery spent before giving up.
   */
  readonly attempts: number;

  /**
   Creates a delivery failure naming the executable, the attempt count and the reason.

   @param command - executable whose delivery attempts failed

   @param attempts - attempts spent before the delivery gave up

   @param reason - what the executable reported, or why no report came

   @param cause - value the process boundary threw, when it threw instead of reporting
   */
  constructor(
    {
      command,
      attempts,
      reason,
      cause,
    }: {
      readonly command: string;
      readonly attempts: number;
      readonly reason: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      `desktop notification through ${command} failed after ${String(attempts,)}`
        + ` ${attempts === 1 ? 'attempt' : 'attempts'}: ${reason}`,
      { cause, },
    );
    this.attempts = attempts;
  }
}

/**
 Says whether one process run delivered the notification.

 @param outcome - what the attempt reported

 @returns whether the desktop accepted the notification

 @example
 ```ts
 isAcceptedAttempt({ ending: { kind: 'exited', exitCode: 0 }, stderr: '', }); // true
 ```
 */
function isAcceptedAttempt(outcome: NotificationAttemptOutcome,): boolean {
  return (outcome.ending
    .kind
    === 'exited') && (outcome.ending
      .exitCode
      === 0);
}

/**
 Describes one unsuccessful attempt in the executable's own terms.

 @param outcome - what the attempt reported

 @param timeoutMs - deadline that bounded the attempt

 @returns one clause naming how the attempt ended, plus its standard error when it wrote one

 @example
 ```ts
 notificationFailureReason({
   outcome: {
     ending: { kind: 'exited', exitCode: 1 },
     stderr: 'Failed to show notification: GDBus.Error:...ExcessNotificationGeneration: ...',
   },
   timeoutMs: 6_000,
 });
 ```
 */
function notificationFailureReason(
  {
    outcome,
    timeoutMs,
  }: {
    readonly outcome: NotificationAttemptOutcome;
    readonly timeoutMs: number;
  },
): string {
  /**
   How the process ended, in the terms the process itself reported.
   */
  const ending = outcome.ending
    .kind
    === 'killed-at-its-deadline'
    ? `no answer within ${String(timeoutMs,)} ms, so it was killed`
    : outcome.ending
      .kind
      === 'ended-by-signal'
      ? `killed by ${outcome.ending
        .signalName}`
      : `exited with code ${String(outcome.ending
        .exitCode,)}`;
  if (outcome.stderr
    .length
    === 0)
    return ending;
  return `${ending}; stderr: ${outcome.stderr}`;
}

//endregion Delivery failure

//region Delivery

/**
 Default retry delay, which is the repository's own sleep.
 */
const delayAfterFailure: NotificationDelay = wait;

/**
 Default monotonic clock, immune to wall-clock steps mid-delivery.

 @returns milliseconds from an arbitrary fixed origin
 */
function monotonicNow(): number {
  return performance.now();
}

/**
 Delivers one settled-agent notification, retrying a refusal inside a total bound.

 A delivery that fails reports the executable's own standard error, retries after
 the delays in {@link NOTIFICATION_DELAYS_BEFORE_RETRY_MS} while the whole delivery
 stays inside {@link NOTIFICATION_TOTAL_BOUND_MS}, and never retries a process that
 could not start: a missing executable does not appear by waiting.

 @param invocation - executable and payload the notification carries

 @param run - replacement process boundary for tests

 @param delay - replacement retry delay for tests

 @param now - replacement clock for tests

 @throws NotificationUnavailableError when every attempt failed, carrying the last
 attempt's reason and the count spent

 @throws NotificationSpawnError when the executable could not be started at all

 @example
 ```ts
 await deliverSettledNotification({
   invocation: {
     command: 'notify-send',
     args: ['--app-name=Pi', 'Pi agent finished', 'Idle in /work/agent-7'],
   },
 });
 ```
 */
async function deliverSettledNotification(
  {
    invocation,
    run = runNotificationProcess,
    delay = delayAfterFailure,
    now = monotonicNow,
  }: SettledNotificationDelivery,
): Promise<void> {
  /**
   Instant this delivery must have finished by, whatever the retries did.
   */
  const deadline = now() + NOTIFICATION_TOTAL_BOUND_MS;

  /**
   Attempts this delivery may make: the first try plus one per measured retry delay.
   */
  const attemptLimit = NOTIFICATION_DELAYS_BEFORE_RETRY_MS.length + 1;

  /* oxlint-disable no-await-in-loop -- Retries must run one after another: the notification server refuses a request that repeats the previous one, so a concurrent retry would race the attempt it is meant to replace. */
  for (let attemptIndex = 0; attemptIndex < attemptLimit; attemptIndex += 1) {
    /**
     Deadline for this attempt: its own bound, or whatever the total bound still leaves.
     */
    const timeoutMs = Math.min(
      NOTIFICATION_ATTEMPT_TIMEOUT_MS,
      deadline - now(),
    );
    /**
     What this attempt reported.
     */
    const outcome = await run({
      ...invocation,
      timeoutMs,
    },);
    if (isAcceptedAttempt(outcome,))
      return;

    /**
     Wait this failure calls for before the next attempt, if another one fits.
     */
    const waitMs = NOTIFICATION_DELAYS_BEFORE_RETRY_MS[attemptIndex];
    if ((waitMs === undefined) || ((now() + waitMs) >= deadline)) {
      throw new NotificationUnavailableError({
        command: invocation.command,
        attempts: attemptIndex + 1,
        reason: notificationFailureReason({
          outcome,
          timeoutMs,
        },),
      },);
    }
    await delay(waitMs,);
  }
  /* oxlint-enable no-await-in-loop */
}

//endregion Delivery

export {
  NOTIFICATION_ATTEMPT_TIMEOUT_MS,
  NOTIFICATION_DELAYS_BEFORE_RETRY_MS,
  NOTIFICATION_TOTAL_BOUND_MS,
  NotificationUnavailableError,
  deliverSettledNotification,
  isAcceptedAttempt,
  notificationFailureReason,
};

export type {
  NotificationClock,
  NotificationDelay,
  SettledNotificationDelivery,
};
