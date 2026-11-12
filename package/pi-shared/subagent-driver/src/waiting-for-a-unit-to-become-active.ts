/**
 Waiting, bounded, until a systemd user unit is active: what a coordinator does after asking for a batch's unit, so the batch is
 known to be up before it is watched, and the wait ends on its own whatever the manager says.

 The wait never ends a unit and never signals anything: it asks the manager for the unit's state, at a period a caller sets, until
 the unit is active or its deadline passes.

 @module
 */

/**
 Milliseconds in a second, so a period a caller states in seconds is read in a millisecond.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 The state of a unit that is up and working.
 */
const ACTIVE_STATE = 'active';

/**
 The state a unit is in while it is being started, which is the one worth waiting out rather than reporting as failed.
 */
export const ACTIVATING_STATE = 'activating';

/**
 What waiting for a unit ended with.
 */
export type UnitWaitForActive = {
  /**
   The unit's state when the wait ended: `active` when it came up, or whatever it was at the deadline.
   */
  readonly state: string;

  /**
   Whether the unit was active.
   */
  readonly becameActive: boolean;

  /**
   How long the wait took, in milliseconds.
   */
  readonly waitedMilliseconds: number;

  /**
   How many times the manager was asked.
   */
  readonly looks: number;
};

/**
 Waits, bounded, until a unit is active.

 Every wait is the shorter of the period and what the deadline leaves, so this ends on its own even when the unit never comes up.

 @param stateOf - one look at the unit's state, which a test replaces

 @param deadline - when the wait must be over, as milliseconds since the epoch

 @param periodMilliseconds - how long to wait between looks

 @param wait - how the wait passes time

 @param now - where the wait reads the time

 @returns the state the wait ended on, how long it took, and how many looks it took

 @example
 ```ts
 const waited = await waitForUnitToBecomeActive({ stateOf, deadline: Date.now() + 60_000, periodMilliseconds: 1_000, wait, now: Date.now },);
 ```
 */
export async function waitForUnitToBecomeActive({
  stateOf,
  deadline,
  periodMilliseconds,
  wait,
  now,
}: {
  readonly stateOf: () => Promise<string>;
  readonly deadline: number;
  readonly periodMilliseconds: number;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly now: () => number;
},): Promise<UnitWaitForActive> {
  /**
   When the wait began.
   */
  const beganAt = now();
  /**
   How many times the manager has been asked.
   */
  const counted: { looks: number } = { looks: 0, };
  /* oxlint-disable eslint/no-await-in-loop -- a look, then a wait, then the next look: the command exists to keep looking while
     the unit is being started. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     The unit's state now.
     */
    const state = await stateOf();
    counted.looks += 1;
    /**
     When this look was answered.
     */
    const lookedAt = now();
    /**
     How much of the deadline is left.
     */
    const left = deadline - lookedAt;
    if ((state === ACTIVE_STATE) || (left <= 0)) {
      return {
        state,
        becameActive: state === ACTIVE_STATE,
        waitedMilliseconds: lookedAt - beganAt,
        looks: counted.looks,
      };
    }
    await wait(Math.min(
      periodMilliseconds,
      left,
    ),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}

/**
 Builds the line a finished wait prints.

 @param unitName - the unit's name as systemd prints it

 @param waited - what the wait ended with

 @returns the line, which says whether the unit came up and what it was doing if it did not

 @example
 ```ts
 const line = waitedLineOf({ unitName: 'launch-a.service', waited },);
 ```
 */
export function waitedLineOf({
  unitName,
  waited,
}: {
  readonly unitName: string;
  readonly waited: UnitWaitForActive;
},): string {
  /**
   How long the wait took, in whole seconds.
   */
  const seconds = Math.round(waited.waitedMilliseconds / MILLISECONDS_PER_SECOND,);
  return waited.becameActive
    ? `${unitName}: active after ${String(seconds,)}s, ${String(waited.looks,)} look${((waited.looks === 1) ? '' : 's')}`
    : `${unitName}: not active after ${String(seconds,)}s and ${String(waited.looks,)} look`
      + `${((waited.looks === 1) ? '' : 's')}, its state ${waited.state}`;
}
