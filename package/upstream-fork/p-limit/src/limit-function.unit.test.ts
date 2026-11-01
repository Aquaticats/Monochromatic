/**
 Tests for `limitFunction`: wrapping one function in its own
 concurrency-bounded call queue.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { limitFunction, } from '../dist/final/neutral/index.mjs';

import {
  createGate,
  type Gate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: limitFunction.name,
  children: [
    it({
      name: 'runs the wrapped function with the call args and resolves its result',
      fn: async () => {
        const limited = limitFunction({
          fn: function join(left: string, right: string,): string {
            return left + right;
          },
          options: {
            concurrency: 1,
          },
        },);
        const result = await limited({
          args: [
            'a',
            'b',
          ],
        },);
        expect(result,).toBe('ab',);
      },
    },),

    it({
      name: 'bounds concurrent executions of the wrapped function',
      fn: async () => {
        /**
         Overlap counters shared by every call of the wrapped function.
         */
        const counts = {
          running: 0,
          maxRunning: 0,
        };
        /**
         Wrapped function that blocks on the gate it is called with.
         */
        async function blockedWork(gate: Gate,): Promise<void> {
          counts.running += 1;
          counts.maxRunning = Math.max(
            counts.maxRunning,
            counts.running,
          );
          await gate.open;
          counts.running -= 1;
        }
        const limited = limitFunction({
          fn: blockedWork,
          options: {
            concurrency: 2,
          },
        },);
        /**
         Gates for the four calls, released in creation order.
         */
        const gates: Gate[] = Array.from(
          { length: 4, },
          function makeGate(): Gate {
            return createGate();
          },
        );
        for (const gate of gates)
          void limited({
            args: [gate],
          },);

        await yieldTurn();
        expect(counts.running,).toBe(2,);
        for (const gate of gates)
          gate.release();
        await yieldTurn();
        expect(counts.maxRunning,).toBe(2,);
        expect(counts.running,).toBe(0,);
      },
    },),

    it({
      name: 'propagates the wrapped function failure',
      fn: async () => {
        /**
         Failure the wrapped function rejects with.
         */
        const failure = new Error('wrapped failed',);
        const limited = limitFunction({
          fn: async function failingWork(): Promise<never> {
            throw failure;
          },
          options: {
            concurrency: 1,
          },
        },);
        let caught: unknown;
        try {
          await limited({
            args: [],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
      },
    },),

    it({
      name: 'exposes clearQueue that drops queued calls',
      fn: async () => {
        /**
         Flags recording which wrapped calls actually ran.
         */
        const state = {
          firstDone: false,
          droppedRan: false,
        };
        /**
         Gate held by the single running call.
         */
        const gate = createGate();
        /**
         Wrapped function that either blocks forever or records a dropped call.
         */
        async function work(mode: string,): Promise<string> {
          if (mode === 'block') {
            await gate.open;
            state.firstDone = true;
            return 'block';
          }
          state.droppedRan = true;
          return 'dropped';
        }
        const limited = limitFunction({
          fn: work,
          options: {
            concurrency: 1,
          },
        },);
        void limited({
          args: ['block'],
        },);
        const dropped = limited({
          args: ['drop'],
        },);
        limited.clearQueue();
        await yieldTurn();
        expect(state.droppedRan,).toBe(false,);
        gate.release();
        await yieldTurn();
        expect(state.firstDone,).toBe(true,);
        expect(state.droppedRan,).toBe(false,);
        void dropped;
      },
    },),
    it({
      name: 'attaches clearQueue as a non-enumerable, non-writable, non-configurable property like upstream p-limit',
      fn: async () => {
        const limited = limitFunction({
          fn: function identity(value: string,): string {
            return value;
          },
          options: {
            concurrency: 1,
          },
        },);
        /**
         Own descriptor of the attached `clearQueue` member.
         */
        const descriptor = Object.getOwnPropertyDescriptor(
          limited,
          'clearQueue',
        );
        expect(descriptor,).toBeDefined();
        expect(descriptor?.enumerable,).toBe(false,);
        expect(descriptor?.writable,).toBe(false,);
        expect(descriptor?.configurable,).toBe(false,);
        expect(typeof descriptor?.value,).toBe('function',);
        expect(Object.keys(limited,),).toEqual([],);
      },
    },),
  ],
},);
