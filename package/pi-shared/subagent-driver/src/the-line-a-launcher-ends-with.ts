/**
 The line a launcher prints as it ends: what is ending, which is the launcher, and what the run's unit is doing at that moment.

 A launcher's own exit line used to read `unit the-unit-ended with status 1`, and a reader took that for the run's end while the
 run it had started was alive and working in its unit. The unit is the manager's and outlives every launcher, so the line names the
 launcher's ending and the unit's own state, in words that cannot be read as the run's end.

 @module
 */

import type { RunCommand, } from './run-command.ts';
import type { UnitWaitOutcome, } from './wait-for-a-unit-to-end.ts';

/**
 The state a unit's state reads as when the manager says nothing about it.
 */
const STATE_UNKNOWN = 'unknown';

/**
 Reads what a unit is doing now, from the manager that holds it.

 `systemctl is-active` exits non-zero for every state but `active` and still prints the state, so the printed state is what this
 reads and the exit status only says whether the unit is active.

 @param run - how a command is run, which a test replaces

 @param unitName - the unit's name as systemd prints it

 @returns the unit's state, such as `active`, `inactive` or `failed`

 @example
 ```ts
 const state = await unitStateNow({ run: runCommandWithChildProcess, unitName: 'pi-a.service', },);
 ```
 */
export async function unitStateNow({
  run,
  unitName,
}: {
  readonly run: RunCommand;
  readonly unitName: string;
},): Promise<string> {
  /**
   What the manager answered.
   */
  const answered = await run({
    command: 'systemctl',
    words: [
      '--user',
      'is-active',
      unitName,
    ],
  },);
  /**
   The state it printed, which `is-active` prints whatever it exits with.
   */
  const state = answered.stdout
    .trim();
  return (state === '') ? STATE_UNKNOWN : state;
}

/**
 Reads the line a launcher prints as it ends.

 @param runName - the run's name

 @param unitName - the unit the run is started as

 @param unitState - what that unit is doing now

 @param ended - what ended the launcher's wait

 @param status - the status the launcher ends with

 @param runFolder - the run's own folder

 @param worktree - the worktree the run works in

 @returns the line, which says the launcher is ending and never that the unit did

 @example
 ```ts
 const line = launcherEndedLineOf({ runName: 'a', unitName: 'pi-a.service', unitState: 'active', ended: 'the-wait-gave-up', status: 1, runFolder: '/runs/a-1', worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export function launcherEndedLineOf({
  runName,
  unitName,
  unitState,
  ended,
  status,
  runFolder,
  worktree,
}: {
  readonly runName: string;
  readonly unitName: string;
  readonly unitState: string;
  readonly ended: UnitWaitOutcome['ended'];
  readonly status: number;
  readonly runFolder: string;
  readonly worktree: string;
},): string {
  /**
   What the wait achieved, in words that name the wait rather than the run.
   */
  const wait = (ended === 'the-unit-ended')
    ? 'its wait saw the unit end'
    : 'its wait gave up before the unit ended';
  return `${runName}: the launcher that started this run is ending with status ${String(status,)} (${wait}); unit ${unitName} is ${unitState} now, and a run that is still working keeps working. Run folder ${runFolder}; worktree ${worktree}`;
}
