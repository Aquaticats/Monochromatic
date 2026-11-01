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
  collect,
  countingAsyncSource,
  countingSyncSource,
  createGate,
  pullAt,
  rejectionOf,
  scriptedAsyncSource,
  yieldDone,
  yieldElement,
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
    describe({
      name: rejectionOf.name,
      children: [
        it({
          name: 'hands back the rejection reason',
          fn: async () => {
            /**
             Reason the promise rejects with.
             */
            const reason = new Error('boom',);
            /**
             Reason handed back by the helper.
             */
            const observed = await rejectionOf(Promise.reject(reason,),);
            expect(observed,).toBe(reason,);
          },
        },),

        it({
          name: 'throws when the promise resolves instead',
          fn: async () => {
            /**
             Promise that resolves where a rejection was expected.
             */
            const resolving = rejectionOf(Promise.resolve(1,),);
            await expect(resolving,).rejects.toThrow('expected the promise to reject',);
          },
        },),
      ],
    },),

    describe({
      name: collect.name,
      children: [
        it({
          name: 'drains an async iterable in order',
          fn: async () => {
            /**
             Async generator over three values.
             */
            async function* three(): AsyncGenerator<number> {
              yield 1;
              yield 2;
              yield 3;
              await Promise.resolve();
            }
            /**
             Values drained from the generator.
             */
            const drained = await collect(three(),);
            expect(drained,).toEqual([
              1,
              2,
              3,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'counting sources',
      children: [
        it({
          name: 'counts the pulls and the closes of a synchronous source',
          fn: async () => {
            const source = countingSyncSource();
            const iterator = source[Symbol.iterator]();
            /**
             Results of two pulls and a close.
             */
            const steps = [
              iterator.next(),
              iterator.next(),
              iterator.return?.(),
            ];
            expect(steps,).toEqual([
              {
                done: false,
                value: 0,
              },
              {
                done: false,
                value: 1,
              },
              {
                done: true,
                value: undefined,
              },
            ],);
            expect(source.counters.pulls,).toBe(2,);
            expect(source.counters.returnCalls,).toBe(1,);
            await yieldTurn();
          },
        },),

        it({
          name: 'counts the pulls and the closes of an asynchronous source',
          fn: async () => {
            const source = countingAsyncSource();
            const iterator = source[Symbol.asyncIterator]();
            /**
             Results of a pull and a close.
             */
            const steps = [
              await iterator.next(),
              await iterator.return?.(),
            ];
            expect(steps,).toEqual([
              {
                done: false,
                value: 0,
              },
              {
                done: true,
                value: undefined,
              },
            ],);
            expect(source.counters.pulls,).toBe(1,);
            expect(source.counters.returnCalls,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: scriptedAsyncSource.name,
      children: [
        it({
          name: 'hands out one pending pull per next call, in call order',
          fn: async () => {
            const source = scriptedAsyncSource<number>();
            const iterator = source.iterable[Symbol.asyncIterator]();
            /**
             Steps of the first two pulls, still pending.
             */
            const steps = [
              iterator.next(),
              iterator.next(),
            ];
            expect(source.pulls,).toHaveLength(2,);

            yieldElement({
              pull: pullAt({
                source,
                index: 1,
              },),
              value: 11,
            },);
            yieldDone(pullAt({
              source,
              index: 0,
            },),);

            expect(await Promise.all(steps,),).toEqual([
              {
                done: true,
                value: undefined,
              },
              {
                done: false,
                value: 11,
              },
            ],);
          },
        },),

        it({
          name: 'fails when the test reads a pull the run has not made',
          fn: async () => {
            const source = scriptedAsyncSource<number>();
            expect(function readMissingPull(): unknown {
              return pullAt({
                source,
                index: 0,
              },);
            },).toThrow();
          },
        },),
      ],
    },),
  ],
},);
