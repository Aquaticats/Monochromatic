/**
 Tests for the `concurrency` accessor: reading, raising, lowering, and
 invalid setter input.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidConcurrencyError,
  pLimit,
} from '../dist/final/neutral/index.mjs';

import {
  createGate,
  type Gate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'concurrency accessor',
  children: [
    it({
      name: 'returns the configured bound',
      fn: async () => {
        const limit = pLimit(3,);
        expect(limit.concurrency,).toBe(3,);
      },
    },),

    it({
      name: 'raising the bound admits queued calls from a microtask',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Start order recorded by every call as its function runs.
         */
        const started: string[] = [];
        /**
         Gate held by the call occupying the single initial slot.
         */
        const gate = createGate();
        /**
         Gates held by the two calls queued behind the initial bound.
         */
        const queuedGates = [
          createGate(),
          createGate(),
        ];
        void limit({
          fn: async function blockedCall(): Promise<void> {
            started.push('blocker',);
            await gate.open;
          },
          args: [],
        },);
        void limit({
          fn: async function queuedOne(): Promise<void> {
            started.push('one',);
            await queuedGates[0]?.open;
          },
          args: [],
        },);
        void limit({
          fn: async function queuedTwo(): Promise<void> {
            started.push('two',);
            await queuedGates[1]?.open;
          },
          args: [],
        },);
        await yieldTurn();
        expect(started,).toEqual([
          'blocker',
        ],);

        limit.concurrency = 3;
        await yieldTurn();
        expect(started,).toEqual([
          'blocker',
          'one',
          'two',
        ],);
        expect(limit.activeCount,).toBe(3,);
        gate.release();
        for (const queuedGate of queuedGates)
          queuedGate.release();
        await yieldTurn();
        expect(limit.activeCount,).toBe(0,);
      },
    },),

    it({
      name: 'lowering the bound never interrupts running calls but caps new starts',
      fn: async () => {
        const limit = pLimit(3,);
        /**
         Gates held by the three calls running at once.
         */
        const gates: Gate[] = [
          createGate(),
          createGate(),
          createGate(),
        ];
        for (const gate of gates)
          void limit({
            fn: async function blockedCall(): Promise<void> {
              await gate.open;
            },
            args: [],
          },);
        /**
         Flag set when the call queued behind the lowered bound starts.
         */
        const state = { queuedRan: false, };
        void limit({
          fn: function queuedCall(): void {
            state.queuedRan = true;
          },
          args: [],
        },);
        await yieldTurn();
        expect(limit.activeCount,).toBe(3,);

        limit.concurrency = 1;
        await yieldTurn();
        expect(state.queuedRan,).toBe(false,);
        expect(limit.activeCount,).toBe(3,);

        gates[0]?.release();
        await yieldTurn();
        expect(state.queuedRan,).toBe(false,);

        gates[1]?.release();
        gates[2]?.release();
        await yieldTurn();
        expect(state.queuedRan,).toBe(true,);
        expect(limit.activeCount,).toBe(0,);
      },
    },),

    it({
      name: 'rejects an invalid bound and keeps the previous value',
      fn: async () => {
        const limit = pLimit(2,);
        let caught: unknown;
        try {
          limit.concurrency = 0;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
        expect(limit.concurrency,).toBe(2,);
      },
    },),

    it({
      name: 'starts every call without waiting when the bound is Number.POSITIVE_INFINITY',
      fn: async () => {
        const limit = pLimit(Number.POSITIVE_INFINITY,);
        for (const index of [
          0,
          1,
          2,
          3,
        ])
          void limit({
            fn: async function alwaysRunning(): Promise<void> {
              await createGate().open;
            },
            args: [],
          },);
        await yieldTurn();
        expect(limit.activeCount,).toBe(4,);
        expect(limit.pendingCount,).toBe(0,);
      },
    },),
  ],
},);
