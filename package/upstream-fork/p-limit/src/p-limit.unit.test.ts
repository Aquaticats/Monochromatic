/**
 Tests for the limiter's call semantics: scheduling, results, failures,
 concurrency bounds, and counters.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type LimitFunction,
  type LimitTask,
  pLimit,
} from '../dist/final/neutral/index.mjs';

import {
  createGate,
  type Gate,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: pLimit.name,
  children: [
    //region Results

    it({
      name: 'resolves with a synchronous function result',
      fn: async () => {
        const limit = pLimit(1,);
        const result = await limit({
          fn: function answer(): number {
            return 42;
          },
          args: [],
        },);
        expect(result,).toBe(42,);
      },
    },),

    it({
      name: 'resolves with a thenable function result',
      fn: async () => {
        const limit = pLimit(1,);
        const result = await limit({
          fn: async function delayedAnswer(): Promise<string> {
            return 'done';
          },
          args: [],
        },);
        expect(result,).toBe('done',);
      },
    },),

    it({
      name: 'spreads the args tuple into the function',
      fn: async () => {
        const limit = pLimit(1,);
        const result = await limit({
          fn: function add(left: number, right: number,): number {
            return left + right;
          },
          args: [
            2,
            3,
          ],
        },);
        expect(result,).toBe(5,);
      },
    },),

    //endregion Results

    //region Scheduling

    it({
      name: 'does not invoke the function during the limit call',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Invocation flag set the moment the scheduled function runs.
         */
        const state = { invoked: false, };
        const pending = limit({
          fn: function markInvoked(): string {
            state.invoked = true;
            return 'ran';
          },
          args: [],
        },);
        expect(state.invoked,).toBe(false,);
        await pending;
        expect(state.invoked,).toBe(true,);
      },
    },),

    it({
      name: 'starts at most concurrency calls at once',
      fn: async () => {
        const limit = pLimit(2,);
        /**
         Overlap counters shared by every scheduled call.
         */
        const counts = {
          running: 0,
          maxRunning: 0,
        };
        /**
         Gates for the six scheduled calls, released in creation order.
         */
        const gates: Gate[] = Array.from(
          { length: 6, },
          function makeGate(): Gate {
            return createGate();
          },
        );
        for (const gate of gates)
          void limit({
            fn: async function trackedCall(): Promise<void> {
              counts.running += 1;
              counts.maxRunning = Math.max(
                counts.maxRunning,
                counts.running,
              );
              await gate.open;
              counts.running -= 1;
            },
            args: [],
          },);

        await yieldTurn();
        expect(counts.running,).toBe(2,);
        expect(limit.activeCount,).toBe(2,);
        expect(limit.pendingCount,).toBe(4,);

        for (const gate of gates)
          gate.release();
        await yieldTurn();
        expect(counts.maxRunning,).toBe(2,);
        expect(counts.running,).toBe(0,);
        expect(limit.activeCount,).toBe(0,);
        expect(limit.pendingCount,).toBe(0,);
      },
    },),

    it({
      name: 'frees the slot when a call settles and starts the next queued call',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Gate held by the first call only.
         */
        const gate = createGate();
        /**
         Call order recorded as functions run.
         */
        const started: string[] = [];
        const first = limit({
          fn: async function blockedFirst(): Promise<string> {
            started.push('first',);
            await gate.open;
            return 'first';
          },
          args: [],
        },);
        const second = limit({
          fn: function queuedSecond(): string {
            started.push('second',);
            return 'second';
          },
          args: [],
        },);

        await yieldTurn();
        expect(started,).toEqual([
          'first',
        ],);
        gate.release();
        expect(await first,).toBe('first',);
        expect(await second,).toBe('second',);
        expect(started,).toEqual([
          'first',
          'second',
        ],);
      },
    },),

    //endregion Scheduling

    //region Failures

    it({
      name: 'rejects with the failure of a rejecting function',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Failure the scheduled function rejects with.
         */
        const failure = new Error('rejected',);
        let caught: unknown;
        try {
          await limit({
            fn: async function failingCall(): Promise<never> {
              throw failure;
            },
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
      name: 'rejects with a synchronous throw from the function',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Failure the scheduled function throws synchronously.
         */
        const failure = new Error('thrown',);
        let caught: unknown;
        try {
          await limit({
            fn: function throwingCall(): never {
              throw failure;
            },
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
      name: 'keeps scheduling after a failed call frees its slot',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Failure the first call throws synchronously.
         */
        const failure = new Error('first fails',);
        /**
         Flag set when the follow-up call runs.
         */
        const state = { followUpRan: false, };
        const failing = limit({
          fn: function failingCall(): never {
            throw failure;
          },
          args: [],
        },);
        const followUp = limit({
          fn: function followUpCall(): string {
            state.followUpRan = true;
            return 'ok';
          },
          args: [],
        },);
        let caught: unknown;
        try {
          await failing;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
        expect(await followUp,).toBe('ok',);
        expect(state.followUpRan,).toBe(true,);
      },
    },),

    //endregion Failures

    //region Counters

    it({
      name: 'reports running calls through activeCount and queued calls through pendingCount',
      fn: async () => {
        const limit = pLimit(1,);
        expect(limit.activeCount,).toBe(0,);
        expect(limit.pendingCount,).toBe(0,);

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
        void limit({
          fn: function queuedCall(): string {
            return 'queued';
          },
          args: [],
        },);
        await yieldTurn();
        expect(limit.activeCount,).toBe(1,);
        expect(limit.pendingCount,).toBe(1,);

        gate.release();
        await running;
        await yieldTurn();
        expect(limit.activeCount,).toBe(0,);
        expect(limit.pendingCount,).toBe(0,);
      },
    },),

    it({
      name: 'keeps limiter members non-enumerable, non-writable, and non-configurable like upstream p-limit',
      fn: async () => {
        const limit = pLimit(1,);
        expect(Object.keys(limit,),).toEqual([],);
        expect({
          ...limit,
        },).toEqual({},);

        /**
         Member descriptor contract shared by both implementations:
         accessors expose only a getter (plus a setter for `concurrency`),
         data members are frozen values, and none of them are enumerable or
         configurable.
         */
        const memberKinds: Readonly<Record<string, string>> = {
          activeCount: 'getter',
          pendingCount: 'getter',
          concurrency: 'getter-setter',
          clearQueue: 'frozen-value',
          map: 'frozen-value',
        };
        for (const [member, kind,] of Object.entries(memberKinds,)) {
          /**
           Own descriptor of this limiter member.
           */
          const descriptor = Object.getOwnPropertyDescriptor(
            limit,
            member,
          );
          expect(descriptor,).toBeDefined();
          expect(descriptor?.enumerable,).toBe(false,);
          expect(descriptor?.configurable,).toBe(false,);
          if (kind === 'frozen-value') {
            expect(descriptor?.writable,).toBe(false,);
            expect(descriptor?.value,).toBeDefined();
          }
          else {
            expect(typeof descriptor?.get,).toBe('function',);
            expect(typeof descriptor?.set,).toBe((kind === 'getter-setter')
              ? 'function'
              : 'undefined',);
          }
        }
      },
    },),

    //endregion Counters

    //region Types

    it({
      name: 'types the limiter as LimitFunction and calls as LimitTask',
      fn: async () => {
        /**
         Limiter whose public type must stay stable for consumers.
         */
        const limit = pLimit(1,);
        expectTypeOf(limit,).toEqualTypeOf<LimitFunction>();
        /**
         Call request whose args tuple must track the function's parameters.
         */
        const task: LimitTask<readonly [number, number], number> = {
          fn: function add(left: number, right: number,): number {
            return left + right;
          },
          args: [
            1,
            2,
          ],
        };
        expectTypeOf(task.args,).toEqualTypeOf<readonly [number, number]>();
      },
    },),

    //endregion Types
  ],
},);
