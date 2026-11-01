/**
 Tests for `clearQueue`: discarding queued calls, the `rejectOnClear`
 hand-off, and the untouched running calls.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pLimit, } from '../dist/final/neutral/index.mjs';

import {
  createGate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'clearQueue',
  children: [
    it({
      name: 'returns undefined and drops queued calls without touching running ones',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Gate held by the single running call.
         */
        const gate = createGate();
        /**
         Flags recording which call functions actually ran.
         */
        const state = {
          runningDone: false,
          droppedRan: false,
        };
        const running = limit({
          fn: async function blockedCall(): Promise<string> {
            await gate.open;
            state.runningDone = true;
            return 'running';
          },
          args: [],
        },);
        const dropped = limit({
          fn: function droppedCall(): string {
            state.droppedRan = true;
            return 'dropped';
          },
          args: [],
        },);

        /**
         Return value of the clear under test.
         */
        const cleared = limit.clearQueue();
        expect(cleared,).toBeUndefined();
        await yieldTurn();
        expect(state.droppedRan,).toBe(false,);
        expect(limit.pendingCount,).toBe(0,);

        gate.release();
        expect(await running,).toBe('running',);
        expect(state.runningDone,).toBe(true,);
        expect(state.droppedRan,).toBe(false,);
        void dropped;
      },
    },),

    it({
      name: 'leaves dropped call promises pending when rejectOnClear is off',
      fn: async () => {
        const limit = pLimit({
          concurrency: 1,
        },);
        /**
         Gate held by the single running call.
         */
        const gate = createGate();
        void limit({
          fn: async function blockedCall(): Promise<void> {
            await gate.open;
          },
          args: [],
        },);
        const dropped = limit({
          fn: function droppedCall(): string {
            return 'never';
          },
          args: [],
        },);
        limit.clearQueue();

        /**
         Whether the dropped call's promise settled within the turn window.
         */
        const state = { settled: false, };
        /**
         Observes the dropped promise settling without awaiting it in the
         test body.
         */
        async function observeSettled(): Promise<void> {
          await dropped;
          state.settled = true;
        }
        void observeSettled();
        await yieldTurn();
        await yieldTurn();
        expect(state.settled,).toBe(false,);
        gate.release();
      },
    },),

    it({
      name: 'rejects dropped call promises with the abort reason when rejectOnClear is on',
      fn: async () => {
        const limit = pLimit({
          concurrency: 1,
          rejectOnClear: true,
        },);
        /**
         Gate held by the single running call.
         */
        const gate = createGate();
        const running = limit({
          fn: async function blockedCall(): Promise<string> {
            await gate.open;
            return 'running';
          },
          args: [],
        },);
        const dropped = limit({
          fn: function droppedCall(): string {
            return 'dropped';
          },
          args: [],
        },);
        limit.clearQueue();

        let caught: unknown;
        try {
          await dropped;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(DOMException,);
        expect((caught as DOMException).name,).toBe('AbortError',);
        expect((caught as DOMException).message,).toBe('This operation was aborted',);
        expect(limit.pendingCount,).toBe(0,);

        gate.release();
        expect(await running,).toBe('running',);
      },
    },),

    it({
      name: 'rejects every dropped call promise in FIFO order with the same abort reason',
      fn: async () => {
        const limit = pLimit({
          concurrency: 1,
          rejectOnClear: true,
        },);
        /**
         Gate held by the single running call.
         */
        const gate = createGate();
        void limit({
          fn: async function blockedCall(): Promise<void> {
            await gate.open;
          },
          args: [],
        },);
        /**
         Promises of the three queued calls, enqueued in order.
         */
        const dropped = [
          'first',
          'second',
          'third',
        ].map(function scheduleDropped(id: string,): Promise<string> {
          return limit({
            fn: function droppedCall(): string {
              return id;
            },
            args: [],
          },);
        },);
        limit.clearQueue();

        /**
         Outcomes of awaiting each dropped call, in order.
         */
        const outcomes = await Promise.allSettled(dropped,);
        expect(outcomes.map(function toOutcomeKind(outcome: PromiseSettledResult<string>,): string {
          return outcome.status;
        },),).toEqual([
          'rejected',
          'rejected',
          'rejected',
        ],);
        for (const outcome of outcomes)
          if (outcome.status === 'rejected') {
            expect(outcome.reason,).toBeInstanceOf(DOMException,);
            expect((outcome.reason as DOMException).name,).toBe('AbortError',);
          }
        gate.release();
      },
    },),
  ],
},);
