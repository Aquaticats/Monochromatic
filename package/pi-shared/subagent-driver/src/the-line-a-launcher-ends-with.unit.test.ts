/**
 The line a launcher prints as it ends: what is ending, the unit's state at that moment, and the words that keep a reader from
 reading it as the run's end. Nothing here starts a unit.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type RunCommand,
  launcherEndedLineOf,
  unitStateNow,
} from '../dist/final/node/index.mjs';

/**
 The run a launcher started.

 @returns the run's name

 @example
 ```ts
 const name = aRunName();
 ```
 */
function aRunName(): string {
  return 'copy-route';
}

/**
 The unit a run of that name is started as.

 @returns the unit's name

 @example
 ```ts
 const unit = aUnitName();
 ```
 */
function aUnitName(): string {
  return `pi-${aRunName()}.service`;
}

/**
 A manager that prints one state, whichever state that is.

 @param state - what it prints

 @param status - the status `is-active` exits with for that state

 @returns how a command is run for this read

 @example
 ```ts
 const run = aManagerSaying({ state: 'active', status: 0, },);
 ```
 */
function aManagerSaying({
  state,
  status,
}: {
  readonly state: string;
  readonly status: number;
},): RunCommand {
  return async function answered() {
    return {
      status,
      stdout: (state === '') ? '' : `${state}\n`,
      stderr: '',
    };
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: unitStateNow.name,
      children: [
        it({
          name: 'reads the state the manager prints, whatever status it exits with, and answers unknown when it prints none',
          fn: async () => {
            expect(await unitStateNow({
              run: aManagerSaying({
                state: 'active',
                status: 0,
              },),
              unitName: aUnitName(),
            },),).toBe('active',);
            // `is-active` exits non-zero for every state but active and still prints the state.
            expect(await unitStateNow({
              run: aManagerSaying({
                state: 'inactive',
                status: 3,
              },),
              unitName: aUnitName(),
            },),).toBe('inactive',);
            expect(await unitStateNow({
              run: aManagerSaying({
                state: '',
                status: 4,
              },),
              unitName: aUnitName(),
            },),).toBe('unknown',);
          },
        },),
      ],
    },),
    describe({
      name: launcherEndedLineOf.name,
      children: [
        it({
          name: 'says the launcher is ending, names the unit and its state now, and never reads as the run ending',
          fn: async () => {
            /**
             The line a launcher prints when its wait gave up while the run still works.
             */
            const line = launcherEndedLineOf({
              runName: aRunName(),
              unitName: aUnitName(),
              unitState: 'active',
              ended: 'the-wait-gave-up',
              status: 1,
              runFolder: '/runs/copy-route-1',
              worktree: `/repo/.claude/worktrees/pi-${aRunName()}`,
            },);
            expect(line,).toContain('the launcher that started this run is ending',);
            expect(line,).toContain(`unit ${aUnitName()} is active now`,);
            expect(line,).toContain('a run that is still working keeps working',);
            expect(line,).toContain('its wait gave up before the unit ended',);
            expect(line,).toContain('/runs/copy-route-1',);
            // The line a reader took for the run's end is gone: no phrasing of "the unit ended with status" is printed.
            expect(line,).not.toContain('unit the-unit-ended',);
            expect(line,).not.toContain('unit the-wait-gave-up',);
          },
        },),
        it({
          name: 'says the wait saw the unit end when it did, so a reader knows which of the two happened',
          fn: async () => {
            /**
             The line a launcher prints once its unit has ended.
             */
            const line = launcherEndedLineOf({
              runName: aRunName(),
              unitName: aUnitName(),
              unitState: 'inactive',
              ended: 'the-unit-ended',
              status: 0,
              runFolder: '/runs/copy-route-1',
              worktree: `/repo/.claude/worktrees/pi-${aRunName()}`,
            },);
            expect(line,).toContain('its wait saw the unit end',);
            expect(line,).toContain(`unit ${aUnitName()} is inactive now`,);
            expect(line,).toContain('ending with status 0',);
          },
        },),
      ],
    },),
  ],
},);
