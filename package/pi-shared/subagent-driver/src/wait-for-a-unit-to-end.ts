/**
 Waiting for a unit to end without waiting forever: the launcher follows the unit by asking the manager what state it is in, and
 ends its own waiter once the unit has read as not running, or once its own bound passes.

 A waiter started with `systemd-run --user --wait` follows the unit through the manager's signals on the session bus. While another
 manager held that bus name on 2026-11-11, sixteen launchers never heard their units end and waited about ten hours; their run
 names then read as already running, so new runs of those names quit at once and the old ones still counted against the pools.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import type { RunCommand, } from './run-command.ts';

/**
 The state a unit is in once it has ended, which every state other than `active` and `activating` answers.
 */
const RUNNING_STATES: ReadonlySet<string> = new Set<string>([
  'active',
  'activating',
]);

/**
 Minutes between two looks at the unit's state, which is one.
 */
const MINUTES_BETWEEN_LOOKS = 1;

/**
 Milliseconds between two looks at the unit's state. A minute is far below the wall clock of any run this starts, and it costs one
 `systemctl show` a minute per run.
 */
export const UNIT_STATE_INTERVAL_MILLISECONDS: number = MINUTES_BETWEEN_LOOKS * MILLISECONDS_PER_SECOND;

/**
 Seconds the wait may outlast the run's own wall clock before it ends whatever the manager says. The unit's `RuntimeMaxSec` is the
 wall clock plus the driver's teardown; fifteen minutes above that covers a manager that answers slowly or not at all, which is
 the failure this bound exists for.
 */
export const UNIT_WAIT_GRACE_SECONDS = 900;

/**
 How many looks in a row must read the unit as not running before the waiter is ended. Two, so a unit whose state is read between
 one ending and the next starting is not taken for a run that has ended.
 */
export const NOT_RUNNING_LOOKS_TO_END = 2;

/**
 What one wait answered.
 */
export type UnitWaitOutcome =
  | {
    /**
     The unit read as not running, so the waiter has nothing left to follow.
     */
    readonly ended: 'the-unit-ended';
  }
    | {
    /**
     The wait reached its own bound, so the unit may still be running.
     */
      readonly ended: 'the-wait-gave-up';
    };

/**
 Reads the state a unit is in.

 @param run - how a command is run, which a test replaces

 @param unitName - the unit's name as systemd prints it

 @returns the state, or empty text when the manager could not answer

 @example
 ```ts
 const state = await unitStateOf({ run: runCommandWithChildProcess, unitName: 'pi-a.service', },);
 ```
 */
export async function unitStateOf({
  run,
  unitName,
}: {
  readonly run: RunCommand;
  readonly unitName: string;
}): Promise<string> {
  /**
   What the manager said of the unit.
   */
  const shown = await run({
    command: 'systemctl',
    words: [
      '--user',
      'show',
      unitName,
      '--property=ActiveState',
      '--value',
    ],
  },);
  return (shown.status === 0) ? shown.stdout
    .trim() : '';
}

/**
 Waits for a unit to end, and ends the waiter this launcher started once it has.

 @param run - how a command is run, which a test replaces

 @param unitName - the unit's name as systemd prints it

 @param deadline - when the wait must be over whatever the manager says, as milliseconds since the epoch

 @param wait - how the wait passes time between looks

 @param now - where the wait reads the time

 @param endWaiter - ends the process that follows the unit, which is this launcher's own waiter

 @returns what ended the wait

 @example
 ```ts
 const outcome = await waitForUnitToEnd({ run, unitName: 'pi-a.service', deadline, wait, now: Date.now, endWaiter, },);
 ```
 */
export async function waitForUnitToEnd({
  run,
  unitName,
  deadline,
  wait,
  now,
  endWaiter,
  waiterIsGone,
}: {
  readonly run: RunCommand;
  readonly unitName: string;
  readonly deadline: number;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly now: () => number;
  readonly endWaiter: () => void;
  readonly waiterIsGone: () => boolean;
}): Promise<UnitWaitOutcome> {
  /**
   How many looks in a row have read the unit as not running, which two in a row end the wait.
   */
  const seen: { notRunningInARow: number } = { notRunningInARow: 0, };
  /* oxlint-disable eslint/no-await-in-loop -- a look, then a wait, then the next look: the look is what tells this wait the unit
     ended, and two looks must not be taken at once. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     How long is left before the wait must be over.
     */
    const left = deadline - now();
    if (left <= 0) {
      endWaiter();
      return { ended: 'the-wait-gave-up', };
    }
    if (waiterIsGone()) {
      return { ended: 'the-unit-ended', };
    }
    /**
     The state the unit is in.
     */
    const state = await unitStateOf({
      run,
      unitName,
    },);
    seen.notRunningInARow = ((state !== '') && (!RUNNING_STATES.has(state,)))
      ? (seen.notRunningInARow + 1)
      : 0;
    if ((seen.notRunningInARow >= NOT_RUNNING_LOOKS_TO_END) || waiterIsGone()) {
      endWaiter();
      return { ended: 'the-unit-ended', };
    }
    await wait(Math.min(
      UNIT_STATE_INTERVAL_MILLISECONDS,
      left,
    ),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
