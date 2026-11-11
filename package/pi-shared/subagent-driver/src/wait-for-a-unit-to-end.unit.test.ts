/**
 Waiting for a unit to end without waiting forever: the looks at the unit's state, the two that end the wait, the bound that ends
 it when the manager answers nothing, and the waiter this launcher ends once its work is done.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CommandOutput,
  type RunCommand,
  NOT_RUNNING_LOOKS_TO_END,
  UNIT_STATE_INTERVAL_MILLISECONDS,
  unitStateOf,
  waitForUnitToEnd,
} from '../dist/final/node/index.mjs';

/**
 A systemd stand-in that answers the unit's state from a list, one look at a time.

 @param states - the states to answer, in order, with the last one repeated

 @returns the runner, and how many times it was asked

 @example
 ```ts
 const systemd = standInSystemd(['active', 'inactive']);
 ```
 */
function standInSystemd(states: readonly string[],): {
  readonly run: RunCommand;
  readonly asked: () => number;
} {
  /**
   How many times the state was asked for.
   */
  const asked: { count: number } = { count: 0, };
  return {
    asked: function count() {
      return asked.count;
    },
    run: async function systemd({ command, words, }: { readonly command: string; readonly words: readonly string[] },): Promise<CommandOutput> {
      asked.count += 1;
      /**
       The state this look is answered with.
       */
      const state = states[Math.min(asked.count, states.length) - 1] ?? '';
      return {
        status: command === 'systemctl' && words.includes('--property=ActiveState',) ? 0 : 1,
        stdout: `${state}\n`,
        stderr: '',
      };
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: unitStateOf.name,
      children: [
        it({
          name: 'reads the state the manager answers for one unit, and answers nothing when it did not answer',
          fn: async () => {
            expect(await unitStateOf({
              run: standInSystemd(['active',],).run,
              unitName: 'pi-a.service',
            },),).toBe('active',);
            /**
             A manager that refused the question.
             */
            const refusing: RunCommand = async function systemd() {
              return {
                status: 1,
                stdout: '',
                stderr: 'Failed to connect to bus',
              };
            };
            expect(await unitStateOf({
              run: refusing,
              unitName: 'pi-a.service',
            },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: waitForUnitToEnd.name,
      children: [
        it({
          name: 'ends the waiter once the unit has read as not running as many times in a row as the wait asks',
          fn: async () => {
            /**
             A unit that is running, then has ended.
             */
            const systemd = standInSystemd(['active', 'inactive',],);
            /**
             How many waits were taken.
             */
            const waits: number[] = [];
            /**
             How many times the waiter was ended.
             */
            const ended: number[] = [];
            expect(await waitForUnitToEnd({
              run: systemd.run,
              unitName: 'pi-a.service',
              deadline: Date.now() + 10_000_000,
              wait: async function wait(milliseconds,) {
                waits.push(milliseconds,);
              },
              now: Date.now,
              endWaiter: function endWaiter() {
                ended.push(1,);
              },
              waiterIsGone: function waiterIsGone() {
                return false;
              },
            },),).toEqual({ ended: 'the-unit-ended', },);
            expect(systemd.asked(),).toBe(NOT_RUNNING_LOOKS_TO_END + 1,);
            expect(ended,).toHaveLength(1,);
            expect(waits[0],).toBe(UNIT_STATE_INTERVAL_MILLISECONDS,);
          },
        },),
        it({
          name: 'takes a state it does not know for a unit that has ended, and a look that answered nothing for one that may still be running',
          fn: async () => {
            /**
             A unit whose state the manager answers with nothing, then with a state nobody expects.
             */
            const systemd = standInSystemd(['', 'failed',],);
            expect(await waitForUnitToEnd({
              run: systemd.run,
              unitName: 'pi-a.service',
              deadline: Date.now() + 10_000_000,
              wait: async function noWait() {
                // The looks follow one another without spending time.
              },
              now: Date.now,
              endWaiter: function endWaiter() {
                // The waiter is left for the launcher's own test.
              },
              waiterIsGone: function waiterIsGone() {
                return false;
              },
            },),).toEqual({ ended: 'the-unit-ended', },);
            expect(systemd.asked(),).toBe(NOT_RUNNING_LOOKS_TO_END + 1,);
          },
        },),
        it({
          name: 'ends the waiter at its own bound when the manager keeps saying the unit runs, and says so',
          fn: async () => {
            /**
             A unit that never stops running.
             */
            const systemd = standInSystemd(['active',],);
            /**
             A clock that passes the bound after the first look.
             */
            let now = 0;
            /**
             How many times the waiter was ended.
             */
            const ended: number[] = [];
            expect(await waitForUnitToEnd({
              run: systemd.run,
              unitName: 'pi-a.service',
              deadline: 0,
              wait: async function noWait() {
                now += 1;
              },
              now: function clock() {
                return now;
              },
              endWaiter: function endWaiter() {
                ended.push(1,);
              },
              waiterIsGone: function waiterIsGone() {
                return false;
              },
            },),).toEqual({ ended: 'the-wait-gave-up', },);
            expect(ended,).toHaveLength(1,);
          },
        },),
        it({
          name: 'ends at once when the waiter is already gone, without asking the manager anything',
          fn: async () => {
            /**
             A manager nobody should reach.
             */
            const systemd = standInSystemd(['active',],);
            expect(await waitForUnitToEnd({
              run: systemd.run,
              unitName: 'pi-a.service',
              deadline: Date.now() + 10_000_000,
              wait: async function noWait() {
                // The waiter is already gone, so no wait is taken.
              },
              now: Date.now,
              endWaiter: function endWaiter() {
                // Nothing is left to end.
              },
              waiterIsGone: function waiterIsGone() {
                return true;
              },
            },),).toEqual({ ended: 'the-unit-ended', },);
            expect(systemd.asked(),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
