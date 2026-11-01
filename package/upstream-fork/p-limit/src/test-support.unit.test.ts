/**
 Tests for the shared test-scheduling helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createGate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: '',
  children: [
    describe({
      name: createGate.name,
      children: [
        it({
          name: 'keeps the gate promise pending until released',
          fn: async () => {
            const gate = createGate();
            /**
             Whether the gate promise settled before the release.
             */
            const state = { settled: false, };
            /**
             Observes the gate promise settling without awaiting it in the
             test body.
             */
            async function observeSettled(): Promise<void> {
              await gate.open;
              state.settled = true;
            }
            void observeSettled();
            await yieldTurn();
            expect(state.settled,).toBe(false,);
          },
        },),

        it({
          name: 'settles the gate promise once released',
          fn: async () => {
            const gate = createGate();
            gate.release();
            /**
             Result of awaiting the released gate.
             */
            const result = await gate.open;
            expect(result,).toBeUndefined();
          },
        },),
      ],
    },),

    describe({
      name: yieldTurn.name,
      children: [
        it({
          name: 'resolves to undefined after a full turn',
          fn: async () => {
            const result = await yieldTurn();
            expect(result,).toBeUndefined();
          },
        },),

        it({
          name: 'spans a macrotask turn, letting earlier timers fire first',
          fn: async () => {
            /**
             Flag set by a timer scheduled before the turn under test.
             */
            const state = { timerFired: false, };
            setTimeout(function markFired(): void {
              state.timerFired = true;
            },
              0,
            );
            await yieldTurn();
            expect(state.timerFired,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
