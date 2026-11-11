/**
 Tests for the bounded, retried settled-agent delivery.

 The clock, the retry delay and the process boundary are all injected, so no case
 waits a real backoff or reaches a desktop.
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NOTIFICATION_ATTEMPT_TIMEOUT_MS,
  NOTIFICATION_DELAYS_BEFORE_RETRY_MS,
  NOTIFICATION_TOTAL_BOUND_MS,
  NotificationSpawnError,
  NotificationUnavailableError,
  deliverSettledNotification,
  notificationFailureReason,
  type NotificationAttemptOutcome,
  type NotificationInvocation,
  type NotificationProcessInput,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Notification the delivery cases send, with a folder so the payload is a real one.
 */
const INVOCATION: NotificationInvocation = {
  command: 'notify-send',
  args: [
    '--app-name=Pi',
    'Pi agent finished',
    'Idle in /work/agent-7',
  ],
};

/**
 Plasma's own words when it refuses one repeated notification.
 */
const PLASMA_REFUSAL = 'Failed to show notification: GDBus.Error:org.freedesktop.Notifications.Error.'
  + 'ExcessNotificationGeneration: Created too many similar notifications in quick succession';

/**
 An attempt the desktop accepted.
 */
const ACCEPTED: NotificationAttemptOutcome = {
  ending: {
    kind: 'exited',
    exitCode: 0,
  },
  stderr: '',
};

/**
 An attempt Plasma refused, as measured on this host.
 */
const REFUSED: NotificationAttemptOutcome = {
  ending: {
    kind: 'exited',
    exitCode: 1,
  },
  stderr: PLASMA_REFUSAL,
};

/**
 An attempt killed by this package's deadline.
 */
const KILLED_BY_DEADLINE: NotificationAttemptOutcome = {
  ending: { kind: 'killed-at-its-deadline', },
  stderr: '',
};

/**
 An attempt a signal ended before any deadline.
 */
const KILLED_BY_SIGNAL: NotificationAttemptOutcome = {
  ending: {
    kind: 'ended-by-signal',
    signalName: 'SIGTERM',
  },
  stderr: '',
};

/**
 Virtual milliseconds the first attempt spends in the bound-sharing case, leaving room for one retry.
 */
const SLOW_FIRST_ATTEMPT_MS = NOTIFICATION_TOTAL_BOUND_MS - 3_000;

/**
 Recorded run of one delivery: what it asked for and when it asked.
 */
type DeliveryReplay = {
  /**
   Requests the delivery handed to the process boundary, in order.
   */
  readonly requests: readonly NotificationProcessInput[];

  /**
   Retry delays the delivery asked for, in order.
   */
  readonly delays: readonly number[];

  /**
   Virtual milliseconds the delivery used, retries and waits included.
   */
  readonly elapsedMs: () => number;

  /**
   Delivery under test, running on the virtual clock.
   */
  readonly delivery: Promise<void>;
};

/**
 Builds a delivery that replays one outcome per attempt on a virtual clock.

 @param outcomes - outcomes each attempt returns, in order; the last one repeats

 @param attemptCostMs - virtual milliseconds each attempt costs

 @returns requests, delays, elapsed virtual time and the delivery itself

 @example
 ```ts
 const replay = deliveryReplay([REFUSED, ACCEPTED]);
 await replay.delivery;
 ```
 */
function deliveryReplay(
  outcomes: readonly NotificationAttemptOutcome[],
  attemptCostMs = 100,
): DeliveryReplay {
  /**
   Virtual milliseconds elapsed since the delivery started.
   */
  let virtualMs = 0;

  /**
   One entry per attempt, appended as the delivery makes it.
   */
  const requests: NotificationProcessInput[] = [];

  /**
   Retry delays the delivery asked for.
   */
  const delays: number[] = [];

  /**
   Attempts the delivery started so far.
   */
  let attemptIndex = 0;

  /**
   Record of this replay, built from the bindings above.
   */
  const replay: DeliveryReplay = {
    requests,
    delays,
    elapsedMs: () => virtualMs,
    delivery: deliverSettledNotification({
      invocation: INVOCATION,
      now: function readVirtualClock(): number {
        return virtualMs;
      },
      delay: async function chargeDelay(milliseconds: number,): Promise<void> {
        delays.push(milliseconds,);
        virtualMs += milliseconds;
      },
      run: async function replayAttempt(input: NotificationProcessInput,): Promise<NotificationAttemptOutcome> {
        requests.push(input,);
        const outcome = outcomes[Math.min(attemptIndex, outcomes.length - 1,)] ?? ACCEPTED;
        attemptIndex += 1;
        virtualMs += attemptCostMs;
        return outcome;
      },
    },),
  };
  return replay;
}

/**
 Reads the failure a delivery reported.

 @param delivery - delivery expected to fail

 @returns value the delivery threw, or `undefined` when it resolved

 @example
 ```ts
 const error = await failureOf(deliveryReplay([REFUSED]).delivery);
 ```
 */
async function failureOf(delivery: Promise<void>,): Promise<unknown> {
  try {
    await delivery;
  }
  catch (error) {
    return error;
  }
  return undefined;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: deliverSettledNotification.name,
      children: [
        it({
          name: 'returns after the first attempt the desktop accepted',
          fn: async () => {
            /** Delivery replay whose first attempt is accepted. */
            const replay = deliveryReplay([ACCEPTED,],);

            await replay.delivery;

            expect(replay.requests,).toHaveLength(1,);
            expect(replay.delays,).toEqual([],);
            expect(replay.requests[0]?.timeoutMs,).toBe(NOTIFICATION_ATTEMPT_TIMEOUT_MS,);
          },
        },),

        it({
          name: 'retries a refusal after the first measured delay and keeps the payload',
          fn: async () => {
            /** Delivery replay refused once, then accepted. */
            const replay = deliveryReplay(
              [
                REFUSED,
                ACCEPTED,
              ],
            );
            const [firstDelay,] = NOTIFICATION_DELAYS_BEFORE_RETRY_MS;

            await replay.delivery;

            expect(replay.requests,).toHaveLength(2,);
            expect(replay.delays,).toEqual([firstDelay,],);
            expect(replay.requests[1]?.args,).toEqual(INVOCATION.args,);
            expect(replay.requests[1]?.command,).toBe(INVOCATION.command,);
          },
        },),

        it({
          name: 'reports the executable own refusal text once every attempt failed',
          fn: async () => {
            /** Delivery replay refused by every attempt. */
            const replay = deliveryReplay([REFUSED,],);
            /** Failure the exhausted delivery reported. */
            const caught = await failureOf(replay.delivery,);

            expect(caught,).toBeInstanceOf(NotificationUnavailableError,);
            expect((caught as NotificationUnavailableError).attempts,).toBe(
              NOTIFICATION_DELAYS_BEFORE_RETRY_MS.length + 1,
            );
            expect((caught as Error).message,).toContain(PLASMA_REFUSAL,);
            expect((caught as Error).message,).toContain('exited with code 1',);
            expect(replay.delays,).toEqual([...NOTIFICATION_DELAYS_BEFORE_RETRY_MS,],);
          },
        },),

        it({
          name: 'gives a retry only the time the total bound leaves',
          fn: async () => {
            /** Delivery replay whose first attempt spends most of the total bound. */
            const replay = deliveryReplay(
              [
                REFUSED,
                ACCEPTED,
              ],
              SLOW_FIRST_ATTEMPT_MS,
            );
            /**
             Milliseconds left for the retry once the first attempt and its delay are paid.
             */
            const expectedRetryMs = NOTIFICATION_TOTAL_BOUND_MS
              - SLOW_FIRST_ATTEMPT_MS
              - (NOTIFICATION_DELAYS_BEFORE_RETRY_MS[0] ?? 0);

            await replay.delivery;

            expect(replay.requests,).toHaveLength(2,);
            expect(replay.requests[1]?.timeoutMs,).toBe(expectedRetryMs,);
          },
        },),

        it({
          name: 'stops retrying when the next delay would cross the total bound',
          fn: async () => {
            /** Delivery replay whose single attempt exhausts the total bound. */
            const replay = deliveryReplay([REFUSED,], NOTIFICATION_TOTAL_BOUND_MS - 1,);
            /** Failure the bound-bound delivery reported. */
            const caught = await failureOf(replay.delivery,);

            expect(replay.requests,).toHaveLength(1,);
            expect(replay.delays,).toEqual([],);
            expect((caught as NotificationUnavailableError).attempts,).toBe(1,);
            expect(replay.elapsedMs(),).toBeLessThanOrEqual(NOTIFICATION_TOTAL_BOUND_MS,);
          },
        },),

        it({
          name: 'never retries an executable that could not be started',
          fn: async () => {
            /**
             Start failure standing in for a missing notify-send.
             */
            const startFailure = new NotificationSpawnError({
              command: INVOCATION.command,
              cause: new Error('spawn notify-send ENOENT',),
            },);
            /** Attempts the unreachable delivery started. */
            let attempts = 0;
            /** Failure the unreachable delivery reported. */
            const caught = await failureOf(deliverSettledNotification({
              invocation: INVOCATION,
              run: async function failToStart(): Promise<NotificationAttemptOutcome> {
                attempts += 1;
                throw startFailure;
              },
              delay: async function neverDelay(): Promise<void> {
                throw new Error('a start failure must not reach a retry delay',);
              },
            },),);

            expect(attempts,).toBe(1,);
            expect(caught,).toBe(startFailure,);
          },
        },),

        it({
          name: 'reports a killed attempt as killed rather than as an exit code',
          fn: async () => {
            /** Delivery replay whose attempts are all killed by the deadline. */
            const replay = deliveryReplay([KILLED_BY_DEADLINE,],);
            /** Failure the exhausted delivery reported. */
            const caught = await failureOf(replay.delivery,);

            expect((caught as Error).message,).toContain(
              `no answer within ${String(NOTIFICATION_ATTEMPT_TIMEOUT_MS,)} ms`,
            );
          },
        },),
      ],
    },),

    describe({
      name: notificationFailureReason.name,
      children: [
        it({
          name: 'names the exit code and the standard error together',
          fn: async () => {
            expect(notificationFailureReason({ outcome: REFUSED, timeoutMs: NOTIFICATION_ATTEMPT_TIMEOUT_MS, },),).toBe(
              `exited with code 1; stderr: ${PLASMA_REFUSAL}`,
            );
          },
        },),

        it({
          name: 'names a signal that ended an attempt before its deadline',
          fn: async () => {
            expect(notificationFailureReason({ outcome: KILLED_BY_SIGNAL, timeoutMs: NOTIFICATION_ATTEMPT_TIMEOUT_MS, },),).toBe(
              'killed by SIGTERM',
            );
          },
        },),

        it({
          name: 'quotes the deadline an attempt ran out of',
          fn: async () => {
            /**
             Deadline this case reports, short enough to read in the expected text.
             */
            const pressedDeadlineMs = 1_500;

            expect(notificationFailureReason({ outcome: KILLED_BY_DEADLINE, timeoutMs: pressedDeadlineMs, },),).toBe(
              `no answer within ${String(pressedDeadlineMs,)} ms, so it was killed`,
            );
          },
        },),
      ],
    },),
  ],
},);
