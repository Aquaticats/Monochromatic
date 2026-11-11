/**
 Asks systemd whether a user unit of a gate still runs, so a wait for a verdict ends when the run died instead of at the end
 of its deadline.

 The states are read from `systemctl show`, which is the only answer systemd gives once a transient unit is unloaded: a unit
 started with `--collect` leaves nothing behind, so a unit the manager does not know is one that stopped
 (`doc/troubleshooting/systemd-run-collect-reports-success-for-a-failed-unit.md`).
 */
import { inspectUnit } from './native-planted-controls-unit.ts';

/**
 Unit states in which the run can still write its log: systemd's own words for a unit that is up, starting, or stopping.
 */
const runningUnitStates: ReadonlySet<string> = new Set([
  'active',
  'activating',
  'deactivating',
]);

/**
 Tells whether one gate unit still runs.

 @param unit - the unit's name, without the `.service` suffix

 @returns true while systemd reports the unit in a running state

 @example
 ```ts
 await userUnitStillRunning({ unit: 'gpc-gate-a-20261110T180203-0500' });
 ```
 */
export async function userUnitStillRunning({ unit }: { readonly unit: string; }): Promise<boolean> {
  /**
   What systemd says about the unit.
   */
  const state = await inspectUnit(unit);
  return runningUnitStates.has(state.state);
}
