/**
 Tests for limiter configuration parsing and validation, exercised through
 the public `pLimit` and `limitFunction` entry points.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidConcurrencyError,
  InvalidRejectOnClearError,
  limitFunction,
  pLimit,
} from '../dist/final/neutral/index.mjs';

import {
  createGate,
  yieldTurn,
} from './test-support.ts';

/**
 Concurrency values the limiter must reject, generated for parameterized
 coverage of every invalid shape.
 */
const INVALID_CONCURRENCIES: readonly unknown[] = [
  0,
  -1,
  -2.5,
  1.5,
  Number.NaN,
  Number.NEGATIVE_INFINITY,
  '3',
  true,
  {},
  [],
];

await describe({
  name: 'limiter option resolution',
  children: [
    //region Accepted shapes

    it({
      name: 'accepts a bare concurrency number',
      fn: async () => {
        const limit = pLimit(3,);
        expect(limit.concurrency,).toBe(3,);
      },
    },),

    it({
      name: 'accepts an options object',
      fn: async () => {
        const limit = pLimit({
          concurrency: 4,
          rejectOnClear: true,
        },);
        expect(limit.concurrency,).toBe(4,);
      },
    },),

    it({
      name: 'accepts Number.POSITIVE_INFINITY as an uncapped bound',
      fn: async () => {
        const limit = pLimit(Number.POSITIVE_INFINITY,);
        expect(limit.concurrency,).toBe(Number.POSITIVE_INFINITY,);
      },
    },),

    //endregion Accepted shapes

    //region Rejected shapes

    ...INVALID_CONCURRENCIES.map(function mapInvalidConcurrency(concurrency: unknown,) {
      return it({
        name: `rejects ${String(concurrency,)} with InvalidConcurrencyError`,
        fn: async () => {
          let caught: unknown;
          try {
            pLimit(concurrency as number,);
          }
          catch (error) {
            caught = error;
          }
          expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
        },
      },);
    },),

    it({
      name: 'rejects a non-boolean rejectOnClear with InvalidRejectOnClearError',
      fn: async () => {
        let caught: unknown;
        try {
          pLimit({
            concurrency: 1,
            rejectOnClear: 'yes',
          } as never,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidRejectOnClearError,);
      },
    },),

    it({
      name: 'validates concurrency before rejectOnClear, matching upstream precedence',
      fn: async () => {
        let caught: unknown;
        try {
          pLimit({
            concurrency: 0,
            rejectOnClear: 'yes',
          } as never,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
      },
    },),

    it({
      name: 'throws a plain TypeError for a non-object, non-number constructor input',
      fn: async () => {
        let caught: unknown;
        try {
          pLimit(null as never,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
      },
    },),

    it({
      name: 'applies the same validation to limitFunction options',
      fn: async () => {
        let caught: unknown;
        try {
          limitFunction({
            fn: function identity(value: string,): string {
              return value;
            },
            options: {
              concurrency: 0,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidConcurrencyError,);
      },
    },),

    //endregion Rejected shapes

    //region Defaults

    it({
      name: 'defaults rejectOnClear to false: clearQueue leaves dropped calls unsettled',
      fn: async () => {
        const limit = pLimit({
          concurrency: 1,
        },);
        const gate = createGate();
        /**
         Flags recording whether the dropped call's function ever ran.
         */
        const runState = { droppedRan: false, };
        void limit({
          fn: async function blockForever(): Promise<string> {
            await gate.open;
            return 'running';
          },
          args: [],
        },);
        const dropped = limit({
          fn: function markDropped(): string {
            runState.droppedRan = true;
            return 'dropped';
          },
          args: [],
        },);
        limit.clearQueue();
        await yieldTurn();
        expect(runState.droppedRan,).toBe(false,);
        expect(limit.pendingCount,).toBe(0,);

        /**
         Whether the dropped call's promise settled within the turn window.
         */
        const settleState = { settled: false, };
        /**
         Observes the dropped promise settling without awaiting it in the
         test body.
         */
        async function observeSettled(): Promise<void> {
          await dropped;
          settleState.settled = true;
        }
        void observeSettled();
        await yieldTurn();
        expect(settleState.settled,).toBe(false,);
        gate.release();
      },
    },),

    it({
      name: 'defaults rejectOnClear to false for the bare-number constructor form',
      fn: async () => {
        const limit = pLimit(1,);
        /**
         Gate held by the single running call.
         */
        const gate = createGate();
        /**
         Flags recording whether the dropped call's function ever ran.
         */
        const runState = { droppedRan: false, };
        void limit({
          fn: async function blockForever(): Promise<string> {
            await gate.open;
            return 'running';
          },
          args: [],
        },);
        const dropped = limit({
          fn: function markDropped(): string {
            runState.droppedRan = true;
            return 'dropped';
          },
          args: [],
        },);
        limit.clearQueue();
        await yieldTurn();
        expect(runState.droppedRan,).toBe(false,);

        /**
         Whether the dropped call's promise settled within the turn window.
         */
        const settleState = { settled: false, };
        /**
         Observes the dropped promise settling without awaiting it in the
         test body.
         */
        async function observeSettled(): Promise<void> {
          await dropped;
          settleState.settled = true;
        }
        void observeSettled();
        await yieldTurn();
        expect(settleState.settled,).toBe(false,);
        gate.release();
      },
    },),

    //endregion Defaults
  ],
},);
